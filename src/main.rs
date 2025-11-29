use core::f64;
use std::{collections::BTreeMap, str::FromStr};

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
    pub measurement: f64,
}

impl FromStr for Measurement {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, measure) = s.split_at(s.find(';').expect("no semicolon!!"));
        Ok(Measurement {
            station_name: name.to_string(),
            measurement: measure[1..].parse().expect("can't parse!!"),
        })
    }
}

type FinalInfo = BTreeMap<String, Record>;

#[derive(Debug)]
struct Record {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        }
    }
}

impl Record {
    fn add_measure(&mut self, measure: f64) {
        if measure < self.min {
            self.min = measure;
        }
        if measure > self.max {
            self.max = measure;
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

fn print_info(info: FinalInfo) {
    let s = info
        .into_iter()
        .map(|(station, data)| {
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
