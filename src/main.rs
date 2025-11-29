use core::f64;
use std::collections::HashMap;

use itertools::Itertools;

mod input;

const FILE_NAME: &str = "measurements.txt";

fn main() {
    // 1. Parse the file into a reader
    let lines = input::input_csv_lines(FILE_NAME);
    let measurements = input::measurements_from_lines(lines);

    // 2. Loop through the file's lines, accumulating information about it
    let info = get_info(measurements).unwrap();

    // 3. Print out the final information
    print_info(info);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub station_name: String,
    // TODO: data oriented design!! make this smaller or use a usize tbh.
    pub measurement: Temperature,
}

impl Measurement {
    fn from_string(mut s: String) -> Self {
        let semi_colon_i = s.find(';').expect("no semicolon!!");

        let measure_str = &s[(semi_colon_i + 1)..];
        let measurement = Temperature::from_str(measure_str);
        s.truncate(semi_colon_i);

        Measurement {
            station_name: s,
            measurement,
        }
    }
}

// TODO: intern strings?
type FinalInfo = HashMap<String, Record>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// We can exploit the fact that 'floats' in this problem always have just one decimal for
/// faster parsing and storage
pub struct Temperature(i16);

impl Temperature {
    const MAX: Self = Self::from_f64(99.9);
    const MIN: Self = Self::from_f64(-99.9);

    fn from_str(s: &str) -> Self {
        let mut bytes = s.as_bytes();
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
    // println!("byte to num {c}");
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

fn get_info(
    measures: impl Iterator<Item = Result<Measurement, std::io::Error>>,
) -> Result<FinalInfo, std::io::Error> {
    let mut info = FinalInfo::default();

    for measure in measures {
        match measure {
            Ok(measure) => {
                info.entry(measure.station_name)
                    .or_default()
                    .add_measure(measure.measurement);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(info)
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
