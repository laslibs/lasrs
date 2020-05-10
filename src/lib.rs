#[macro_use]
extern crate lazy_static;
use crate::util::{metadata, remove_comment, SPACES, SPACES_AND_DOT};
use std::path::Path;

mod util;

pub struct Lasrs {
    blob: String,
}

impl Lasrs {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            blob: std::fs::read_to_string(path.as_ref())
                .expect("Invalid path, verify existence of file"),
        }
    }

    pub fn version(self) -> f64 {
        let (res, _) = metadata(&self.blob);
        match res {
            Some(v) => v,
            None => panic!("Invalid version"),
        }
    }

    pub fn wrap(self) -> bool {
        let (_, v) = metadata(&self.blob);
        v
    }

    pub fn headers(self) -> Vec<String> {
        self.blob
            .splitn(3, "~C")
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

    pub fn data(self) -> Vec<Vec<f64>> {
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
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_test() {
        let las = Lasrs::new("./sample/example1.las");
        assert_eq!(las.version(), 2.0);
    }
    #[test]
    fn wrap_test() {
        let las = Lasrs::new("./sample/example1.las");
        assert_eq!(las.wrap(), false);
    }
    #[test]
    fn headers_test() {
        let las = Lasrs::new("./sample/example1.las");
        assert_eq!(
            vec!["DEPT", "DT", "RHOB", "NPHI", "SFLU", "SFLA", "ILM", "ILD"],
            las.headers()
        );
        let las = Lasrs::new("./sample/A10.las");
        assert_eq!(
            vec![
                "DEPT",
                "Perm",
                "Gamma",
                "Porosity",
                "Fluvialfacies",
                "NetGross"
            ],
            las.headers()
        );
    }

    #[test]
    fn data_test() {
        let las = Lasrs::new("./sample/example1.las");
        let expected: Vec<Vec<f64>> = vec![
            vec![1670.0, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
            vec![1669.875, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
            vec![1669.75, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
            vec![
                1669.745, 123.45, 2550.0, -999.25, 123.45, 123.45, 110.2, 105.6,
            ],
        ];
        assert_eq!(expected, las.data());
        let las = Lasrs::new("./sample/A10.las");
        let expected: Vec<Vec<f64>> = vec![
            vec![1499.879, -999.25, -999.25, -999.25, -999.25, 0.0],
            vec![1500.129, -999.25, -999.25, -999.25, -999.25, 0.0],
            vec![1500.629, -999.25, -999.25, -999.25, -999.25, 0.0],
            vec![1501.129, -999.25, -999.25, 0.270646, 0.0, 0.0],
            vec![1501.629, 124.5799, 78.869453, 0.267428, 0.0, 0.0],
        ];
        assert_eq!(expected, &las.data()[0..5]);
    }
}
