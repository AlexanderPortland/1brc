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
    pub measurement: f64,
}

impl Measurement {
    fn from_string(mut s: String) -> Self {
        let semi_colon_i = s.find(';').expect("no semicolon!!");

        let measure_str = &s[(semi_colon_i + 1)..];
        let measurement = measure_str.parse::<f64>().expect("can't parse!!");
        s.truncate(semi_colon_i);

        Measurement {
            station_name: s,
            measurement,
        }
    }
}

// TODO: intern strings?
type FinalInfo = HashMap<String, Record>;

#[derive(Debug)]
struct Record {
    min: f64,
    /// Boolean flag set to mark if the min of this record cannot possibly change anymore.
    minned: bool,
    max: f64,
    /// Boolean flag set to mark if the max of this record cannot possibly change anymore.
    maxed: bool,
    sum: f64,
    count: usize,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            min: f64::MAX,
            minned: false,
            max: f64::MIN,
            maxed: false,
            sum: 0.0,
            count: 0,
        }
    }
}

const MAX_MEASURE: f64 = 99.9;
const MIN_MEASURE: f64 = -99.9;

impl Record {
    fn add_measure(&mut self, measure: f64) {
        if !self.minned && measure < self.min {
            self.min = measure;
            if self.min == MIN_MEASURE {
                self.minned = true;
            }
        }
        if !self.maxed && measure > self.max {
            self.max = measure;
            if self.max == MAX_MEASURE {
                self.maxed = true;
            }
        }
        self.sum += measure;
        self.count += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    fn mean(&self) -> f64 {
        self.sum / (self.count as f64)
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
                data.min,
                data.mean(),
                data.max
            )
        })
        .join(", ");
    print!("{{{s}}}");
}
