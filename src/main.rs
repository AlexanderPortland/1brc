#![allow(internal_features)]
#![feature(slice_internals)]
use core::{f64, slice::memchr::memchr};

use itertools::Itertools;

mod input;

const FILE_NAME: &str = "measurements.txt";
const NUM_THREADS: u64 = 12;

fn main() {
    let (file_mem, file_len) = input::initialize_file(FILE_NAME);
    let per_thread = file_len / NUM_THREADS;

    let threads = (0..NUM_THREADS)
        .map(|thread_id| {
            std::thread::spawn(move || {
                let start_offset = thread_id * per_thread;
                read_file(
                    usize::try_from(start_offset).unwrap(),
                    usize::try_from(per_thread).unwrap(),
                    file_mem,
                )
            })
        })
        .collect::<Vec<_>>();

    let infos = threads
        .into_iter()
        .map(|a| a.join().unwrap())
        .collect::<Vec<_>>();
    let info = join_infos(infos);
    print_info(info);
}

const NEWLINE: u8 = b'\n';
const NAME_SEP: u8 = b';';

fn read_file(start_offset: usize, bytes: usize, file_mem: &[u8]) -> FinalInfo {
    // 1. Parse the file into a reader
    let mut offset = start_offset;

    if offset != 0 {
        // check if the byte right before us is a '\n'
        if file_mem[offset - 1] != NEWLINE {
            let to_next_nl =
                memchr(NEWLINE, &file_mem[offset..]).expect("should handle this properly");
            offset += to_next_nl + 1;
        }
    }

    let mut info = FinalInfo::default();

    loop {
        let Some(name_len) = memchr(NAME_SEP, &file_mem[offset..]) else {
            break;
        };
        let name = &file_mem[offset..(offset + name_len)];

        let temp_start = offset + name_len + 1;
        let temp_len = memchr(NEWLINE, &file_mem[temp_start..])
            .expect("should always have corresponding temp");
        let temp = &file_mem[temp_start..(temp_start + temp_len)];

        let measure = Measurement::from_bytes(name, temp);

        // Add to info map.
        if let Some(existing) = info.get_mut(measure.station_name) {
            existing.add_measure(measure.measurement);
        } else {
            let mut new_record = Record::default();
            new_record.add_measure(measure.measurement);
            info.insert(measure.station_name.into(), new_record);
        }

        offset = temp_start + temp_len + 1;

        if offset >= (start_offset + bytes) {
            break;
        }
    }

    info
}

#[derive(Debug, Clone, PartialEq)]
pub struct Measurement<'a> {
    pub station_name: &'a [u8],
    // TODO: data oriented design!! make this smaller or use a usize tbh.
    pub measurement: Temperature,
}

impl<'a> Measurement<'a> {
    fn from_bytes(name: &'a [u8], temp: &[u8]) -> Self {
        let measurement = Temperature::from_bytes(temp);

        Measurement {
            station_name: name,
            measurement,
        }
    }
}

// TODO: intern strings?
type FinalInfo = rustc_data_structures::fx::FxHashMap<Box<[u8]>, Record>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// We can exploit the fact that 'floats' in this problem always have just one decimal for
/// faster parsing and storage
pub struct Temperature(i16);

impl Temperature {
    const MAX: Self = Self::from_f64(99.9);
    const MIN: Self = Self::from_f64(-99.9);

    fn from_bytes(mut bytes: &[u8]) -> Self {
        // The length of this string should never be more than 5 (1 negative, 3 digits, 1 decimal).
        debug_assert!(bytes.len() < 5);
        let neg = bytes[0] == b'-';
        if neg {
            bytes = &bytes[1..];
        }

        debug_assert!(bytes.len() < 4);

        let mut total = 0;

        // TODO: don't love this code pattern, so should fix it,
        // but for now i think it should be optimized away nicely anyway...
        total += byte_to_num(bytes[bytes.len() - 1]);

        if bytes.len() >= 3 {
            total += 10 * byte_to_num(bytes[bytes.len() - 3]);
        }

        if bytes.len() >= 4 {
            total += 100 * byte_to_num(bytes[bytes.len() - 4]);
        }

        if neg {
            total *= -1;
        }

        Temperature(total)
    }

    #[allow(clippy::cast_possible_truncation)]
    const fn from_f64(f: f64) -> Self {
        Temperature((f * 10.0) as i16)
    }

    fn to_f64(self) -> f64 {
        f64::from(self.0) / 10.0
    }
}

fn byte_to_num(c: u8) -> i16 {
    (c - 48).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Using a different big data struct for our accumulators so that the actual data struct
/// can be very tiny...
struct BigTemperature(i32);

impl BigTemperature {
    #[allow(clippy::cast_precision_loss)]
    const fn to_f64(self) -> f64 {
        self.0 as f64 / 10.0
    }
}

impl std::ops::AddAssign<Temperature> for BigTemperature {
    fn add_assign(&mut self, rhs: Temperature) {
        self.0 += i32::from(rhs.0);
    }
}

#[derive(Debug)]
struct Record {
    min: Temperature,
    /// Boolean flag set to mark if the min of this record cannot possibly change anymore.
    minned: bool,
    max: Temperature,
    /// Boolean flag set to mark if the max of this record cannot possibly change anymore.
    maxed: bool,
    sum: BigTemperature,
    count: usize,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            min: Temperature::MAX,
            minned: false,
            max: Temperature::MIN,
            maxed: false,
            sum: BigTemperature(0),
            count: 0,
        }
    }
}

impl std::ops::AddAssign<Record> for Record {
    fn add_assign(&mut self, rhs: Record) {
        self.count += rhs.count;
        self.sum.0 += rhs.sum.0;
        self.max = std::cmp::max(self.max, rhs.max);
        self.maxed = std::cmp::max(self.maxed, rhs.maxed);
        self.min = std::cmp::min(self.min, rhs.min);
        self.minned = std::cmp::max(self.minned, rhs.minned);
    }
}

impl Record {
    fn add_measure(&mut self, measure: Temperature) {
        if !self.minned && measure < self.min {
            self.min = measure;
            if self.min == Temperature::MAX {
                self.minned = true;
            }
        }
        if !self.maxed && measure > self.max {
            self.max = measure;
            if self.max == Temperature::MIN {
                self.maxed = true;
            }
        }
        self.sum += measure;
        self.count += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    fn mean(&self) -> f64 {
        self.sum.to_f64() / (self.count as f64)
    }
}

fn join_infos(mut infos: Vec<FinalInfo>) -> FinalInfo {
    let mut first = infos.pop().unwrap();
    while let Some(next) = infos.pop() {
        for (s, record) in next {
            *first.entry(s).or_default() += record;
        }
    }
    first
}

#[allow(clippy::needless_pass_by_value)]
fn print_info(info: FinalInfo) {
    let mut keys = info.keys().collect::<Box<[_]>>();
    // Sort to ensure station names are presented in order.
    keys.sort_unstable();

    let s = keys
        .into_iter()
        .map(|station| {
            let data = &info[station];
            let station = std::str::from_utf8(station).unwrap();
            format!(
                "{station}={:.1}/{:.1}/{:.1}",
                data.min.to_f64(),
                data.mean(),
                data.max.to_f64()
            )
        })
        .join(", ");
    print!("{{{s}}}");
}
