use std::fs::File;
use std::io;
use std::path::Path;
use std::fmt;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BitpackingSpec {
    pub data_name: String,
    pub bit_start: usize,
    pub bit_length: usize,
    pub bit_subslice_start: Option<usize>,
    pub bit_subslice_length: Option<usize>
}

impl fmt::Display for BitpackingSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let subslice_start = match self.bit_subslice_start {
            Some(start) => format!("{}", start),
            None => String::new(),
        };
        let subslice_length = match self.bit_subslice_length {
            Some(length) => format!("{}", length),
            None => String::new(),
        };
        write!(
            f,
            "{} {} {} {} {}",
            self.data_name, self.bit_start, self.bit_length, subslice_start, subslice_length
        )
    }
}

impl FromStr for BitpackingSpec {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(());
        }
        let data_name = parts[0].to_string();
        let bit_start = parts[1].parse().map_err(|_| ())?;
        let bit_length = parts[2].parse().map_err(|_| ())?;
        let bit_subslice_start = if parts.len() > 3 {
            Some(parts[3].parse().map_err(|_| ())?)
        } else {
            None
        };
        let bit_subslice_length = if parts.len() > 4 {
            Some(parts[4].parse().map_err(|_| ())?)
        } else {
            None
        };
        Ok(BitpackingSpec {
            data_name,
            bit_start,
            bit_length,
            bit_subslice_start,
            bit_subslice_length,
        })
    }
}