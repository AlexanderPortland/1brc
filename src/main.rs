use core::f64;
use std::io::BufRead;

use itertools::Itertools;

const FILE_NAME: &str = "measurements.txt";

fn main() {
    // 1. Parse the file into a reader
    let file = std::fs::File::open(FILE_NAME).expect("file could not be opened");
    // TODO: play w internal buffer capacity
    let mut reader = std::io::BufReader::new(file);

    let mut name_buf = Vec::new();
    let mut temp_buf = Vec::with_capacity(5);

    let mut info = FinalInfo::default();

    loop {
        name_buf.clear();
        temp_buf.clear();
        let name_len = reader.read_until(b';', &mut name_buf).unwrap();
        if name_len == 0 {
            break;
        }
        let name = &name_buf[..(name_len - 1)];

        let temp_len = reader.read_until(b'\n', &mut temp_buf).unwrap();
        debug_assert_ne!(temp_len, 0);
        let temp = &temp_buf[..(temp_len - 1)];

        let measure = Measurement::from_bytes(name, temp);

        // Add to info map.
        // println!(
        //     "station name is {:?} ({})",
        //     measure.station_name,
        //     measure.station_name.len()
        // );
        if let Some(existing) = info.get_mut(measure.station_name) {
            existing.add_measure(measure.measurement);
        } else {
            let mut new_record = Record::default();
            new_record.add_measure(measure.measurement);
            info.insert(measure.station_name.into(), new_record);
        }
    }

    // 3. Print out the final information
    print_info(info);
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
