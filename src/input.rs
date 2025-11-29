use std::{
    io::{self, BufRead},
    path::Path,
    str::FromStr,
};

use crate::Measurement;

/// Gets an iterator over the lines in this file that can then be lazily pulled in for more data.
pub fn input_csv_lines(
    path: impl AsRef<Path>,
) -> impl Iterator<Item = Result<String, std::io::Error>> {
    let file = std::fs::File::open(path).expect("file could not be opened");

    io::BufReader::new(file).lines().skip(2)
}

pub fn measurements_from_lines(
    lines: impl Iterator<Item = Result<String, std::io::Error>>,
) -> impl Iterator<Item = Result<Measurement, std::io::Error>> {
    lines.map(|line| line.map(|string| FromStr::from_str(&string).unwrap()))
}
