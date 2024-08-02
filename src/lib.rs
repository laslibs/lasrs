//! #Lasrs
//!
//! lasrs is a crate used to parse geophysical well log files `.las`.
//! Provides utilities for extracting strongly typed information from the files.
//! Supports Las Version 2.0 by [Canadian Well Logging Society](http://www.cwls.org) -
//! [Specification](https://www.cwls.org/wp-content/uploads/2017/02/Las2_Update_Feb2017.pdf)

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{collections::HashMap, path::Path};

mod util;
use util::{metadata, property, remove_comment, SPACES, SPACES_AND_DOT};

pub use util::WellProp;

/// Represents a parsed well log file
pub struct Las {
    /// blob holds the String data read from the file
    /// ## Note
    /// There's no need to access the blob field, only exposed for debugging
    pub blob: String,
}

impl Las {
    /// Returns a `Las` read from a las file with the given path
    ///
    /// ## Arguments
    ///
    /// `path` - Path to well log file
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(&log.blob[..=7], "~VERSION");
    /// ```
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let mut blob = String::new();
        let f = File::open(path.as_ref()).expect("Invalid path, verify existence of file");
        let mut br = BufReader::new(f);
        br.read_to_string(&mut blob).expect("Unable to read file");
        Self { blob }
    }

    /// Returns `f64` representing the version of Las specification
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(log.version(), 2.0);
    /// ```
    pub fn version(&self) -> f64 {
        metadata(&self.blob)
            .and_then(|(v, _)| v)
            .expect("Invalid version")
    }

    /// Returns a `bool` denoting the wrap mode
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(log.wrap(), false);
    /// ```
    pub fn wrap(&self) -> bool {
        metadata(&self.blob).map(|(_, w)| w).unwrap_or_default()
    }

    /// Returns `Vec<String>` representing the titles of the curves (~C),
    /// Which can be mapped to a row in ~A (data) section
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/A10.las");
    /// assert_eq!(
    ///     log.headers(),
    ///     vec!["DEPT", "Perm", "Gamma", "Porosity", "Fluvialfacies", "NetGross"],
    /// );
    /// ```
    pub fn headers(&self) -> Vec<String> {
        self.blob
            .splitn(2, "~C")
            .nth(1)
            .unwrap_or("")
            .splitn(2, "~")
            .nth(0)
            .map(|x| remove_comment(x))
            .unwrap_or_default()
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

    /// Returns `Vec<Vec<f64>>` where every Vec<f64> represents a row in ~A (data) section,
    /// and every f64 represents an entry in a column/curve
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/A10.las");
    /// let expected: Vec<Vec<f64>> = vec![
    ///                     vec![1501.129, -999.25, -999.25, 0.270646, 0.0, 0.0],
    ///                     vec![1501.629, 124.5799, 78.869453, 0.267428, 0.0, 0.0],
    ///                   ];
    /// assert_eq!(expected, &log.data()[3..5]);
    /// ```
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

    /// Returns `Vec<f64>` - all reading for a curve/column
    ///
    /// ## Arguments
    ///
    /// `col` - string slice representing the title of the column
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(
    ///     vec![1670.0, 1669.875, 1669.75, 1669.745],
    ///     log.column("DEPT")
    /// );
    /// ```
    pub fn column(self, col: &str) -> Vec<f64> {
        let index = self
            .headers()
            .into_iter()
            .position(|x| x == col.to_owned())
            .expect("msg");
        self.data().into_iter().map(|x| x[index]).collect()
    }

    /// Returns `usize` representing the total number of columns/curves
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(8, log.column_count());
    /// ```
    pub fn column_count(&self) -> usize {
        self.headers().len()
    }

    /// Returns `usize` representing the total number of entry in ~A (data) section
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// assert_eq!(4, log.row_count());
    /// ```
    pub fn row_count(&self) -> usize {
        self.data().len()
    }

    /// Returns `Vec<(String, String)>` where the first item in the tuple is the title of curve
    /// and the second is the full description of the curve
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// let mut expected = vec![
    ///     ("DEPT".to_owned(), "DEPTH".to_owned()),
    ///     ("DT".to_owned(), "SONIC TRANSIT TIME".to_owned()),
    ///     ("ILD".to_owned(), "DEEP RESISTIVITY".to_owned()),
    /// ];
    /// let mut result = log.headers_and_desc();
    /// result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    /// assert_eq!(expected, &result[..3]);
    /// ```
    pub fn headers_and_desc(&self) -> Vec<(String, String)> {
        property(self.blob.as_str(), "~C")
            .unwrap_or_default()
            .into_iter()
            .map(|(title, body)| (title, body.description))
            .collect()
    }

    /// Returns `HashMap<String, WellProp>` containing all the `WellProp`(s) in a ~C (curve) section
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::{Las, WellProp};
    /// let log = Las::new("./sample/example.las");
    /// let curve_section = log.curve_params();
    /// assert_eq!(
    ///     &WellProp::new("OHMM", "SHALLOW RESISTIVITY", "07 220 04 00"),
    ///     curve_section.get("SFLU").unwrap()
    /// );
    /// ```
    pub fn curve_params(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~C").unwrap_or_default()
    }

    /// Returns `HashMap<String, WellProp>` containing all the `WellProp`(s) in a ~W (well) section
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::{Las, WellProp};
    /// let log = Las::new("./sample/example.las");
    /// let well_section = log.well_info();
    /// assert_eq!(
    ///     &WellProp::new("M", "STOP DEPTH", "1669.7500"),
    ///     well_section.get("STOP").unwrap()
    /// );
    /// ```
    pub fn well_info(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~W").unwrap_or_default()
    }

    /// Returns `HashMap<String, WellProp>` containing all the `WellProp`(s) in a ~P (parameter) section
    ///
    /// ## Example
    ///
    /// ```
    /// use lasrs::{Las, WellProp};
    /// let log = Las::new("./sample/example.las");
    /// let params = log.log_params();
    /// assert_eq!(
    ///     &WellProp::new("", "MUD TYPE", "GEL CHEM"),
    ///     params.get("MUD").unwrap()
    /// );
    pub fn log_params(&self) -> HashMap<String, WellProp> {
        property(&self.blob, "~P").unwrap_or_default()
    }

    /// Returns a `String` representing extra information in ~O (other) section
    ///
    /// ## Example
    /// ```
    /// use lasrs::Las;
    /// let log = Las::new("./sample/example.las");
    /// let expected = [
    ///     "Note: The logging tools became stuck at 625 metres causing the data",
    ///     "between 625 metres and 615 metres to be invalid.",
    /// ];
    /// assert_eq!(log.other(), expected.join("\n").to_string());
    /// ```
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

    /// Converts file to csv and saves it to the current directory
    /// ## Arguments
    ///
    /// `filename` - string slice, the name used to save the csv file
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
