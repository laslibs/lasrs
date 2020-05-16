//! #Lasrs
//!
//! lasrs is a crate used to parse geophysical well log files `.las`.
//! Provides utilities for extracting strongly typed information from the files.
//! Supports Las Version by [Canadian Well Logging Society](http://www.cwls.org)
//! [Specification](https://www.cwls.org/wp-content/uploads/2017/02/Las2_Update_Feb2017.pdf)

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{collections::HashMap, path::Path};

mod util;
use util::{metadata, property, remove_comment, SPACES, SPACES_AND_DOT};

pub use util::WellProp;

pub struct Las {
    blob: String,
}

impl Las {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let mut blob = String::new();
        let f = File::open(path.as_ref()).expect("Invalid path, verify existence of file");
        let mut br = BufReader::new(f);
        br.read_to_string(&mut blob).expect("Unable to read file");
        Self { blob }
    }

    pub fn version(&self) -> f64 {
        let (res, _) = metadata(&self.blob);
        match res {
            Some(v) => v,
            None => panic!("Invalid version"),
        }
    }

    pub fn wrap(&self) -> bool {
        let (_, v) = metadata(&self.blob);
        v
    }

    pub fn headers(&self) -> Vec<String> {
        self.blob
            .splitn(2, "~C")
            .nth(1)
            .unwrap_or("")
            .splitn(2, "~")
            .nth(0)
            .map(|x| remove_comment(x))
            .unwrap_or(vec![])
            .into_iter()
            .skip(1)
            .filter_map(|x| {
                SPACES_AND_DOT
                    .splitn(x.trim(), 2)
                    .next()
                    .map(|x| x.to_string())
            })
            .collect()
    }

    pub fn data(&self) -> Vec<Vec<f64>> {
        self.blob
            .splitn(2, "~A")
            .nth(1)
            .unwrap_or("")
            .lines()
            .skip(1)
            .flat_map(|x| {
                SPACES
                    .split(x.trim())
                    .map(|v| v.trim().parse::<f64>().unwrap_or(0.0))
            })
            .collect::<Vec<f64>>()
            .chunks(self.headers().len())
            .map(|ch| Vec::from(ch))
            .collect()
    }

    pub fn column(self, col: &str) -> Vec<f64> {
        let index = self
            .headers()
            .into_iter()
            .position(|x| x == col.to_owned())
            .expect("msg");
        self.data().into_iter().map(|x| x[index]).collect()
    }

    pub fn column_count(&self) -> usize {
        self.headers().len()
    }

    pub fn row_count(&self) -> usize {
        self.data().len()
    }

    pub fn headers_and_desc(&self) -> Vec<(String, String)> {
        property(self.blob.as_str(), "~C")
            .into_iter()
            .map(|(title, body)| (title, body.description))
            .collect()
    }

    pub fn curve_params(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~C")
    }

    pub fn well_info(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~W")
    }

    pub fn log_params(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~P")
    }

    pub fn other(&self) -> String {
        self.blob
            .splitn(2, "~O")
            .nth(1)
            .unwrap_or("")
            .splitn(2, "~")
            .nth(0)
            .map(|x| remove_comment(x))
            .unwrap_or(vec![])
            .into_iter()
            .skip(1)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_csv(&self, filename: &str) {
        let f = File::create(format!("{}.csv", filename)).expect("Unable to create csv file");
        let mut f = BufWriter::new(f);
        let mut headers = self.headers().join(",");
        headers.push_str("\n");
        f.write(headers.as_bytes())
            .expect("Unable to write headers");
        let data = self
            .data()
            .into_iter()
            .map(|x| x.into_iter().map(|d| d.to_string()))
            .map(|x| x.collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join("\n");
        f.write_all(data.as_bytes())
            .expect("Unable to write data to file");
    }
}
