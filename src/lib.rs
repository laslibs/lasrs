#[macro_use]
extern crate lazy_static;
use crate::util::{metadata, remove_comment, SPACES_AND_DOT};
use std::path::Path;

mod util;

pub struct Lasrs {
    blob: String,
}

impl Lasrs {
    pub fn new(path: &str) -> Self {
        Self {
            blob: std::fs::read_to_string(Path::new(path))
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
}
