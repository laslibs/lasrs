use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref DOT_IN_SPACES: Regex = Regex::new("\\s*[.]\\s+").unwrap();
    pub static ref SPACES_AND_DOT: Regex = Regex::new("\\s+[.]").unwrap();
    static ref DOT_OR_SPACES: Regex = Regex::new("[.]|\\s+").unwrap();
    static ref LETTERS_AND_DOT_IN_SPACES: Regex = Regex::new("^\\w+\\s*[.]*s*").unwrap();
    static ref DIGITS_AND_SPACES: Regex = Regex::new("\\d+\\s*").unwrap();
    static ref LETTERS_IN_SPACES: Regex = Regex::new("\\s{2,}\\w*\\s{2,}").unwrap();
    pub static ref SPACES: Regex = Regex::new("\\s+").unwrap();
}
#[derive(Debug, PartialEq)]
pub struct WellProps {
    unit: String,
    description: String,
    value: String,
}

impl WellProps {
    fn new(unit: &str, description: &str, value: &str) -> Self {
        Self {
            unit: unit.to_string(),
            description: description.to_string(),
            value: value.to_string(),
        }
    }
}

pub(crate) fn remove_comment(raw_str: &str) -> Vec<&'_ str> {
    raw_str
        .lines()
        .filter_map(|x| {
            if x.trim().starts_with("#") || x.trim().len() < 1 {
                None
            } else {
                Some(x.trim())
            }
        })
        .collect()
}

#[test]
fn test_remove_comment() {
    let test = "#remove me
    #    still remove me
    retain me
      retain me but trimmed  
    123 retain";
    let expected = vec!["retain me", "retain me but trimmed", "123 retain"];
    assert_eq!(expected, remove_comment(test))
}

pub(crate) fn metadata(raw_str: &str) -> (Option<f64>, bool) {
    lazy_static! {
        static ref SPACEMATCH: Regex = Regex::new(r"\s+|\s*:").unwrap();
    }
    let m = raw_str
        .split('~')
        .nth(1)
        .map(|x| remove_comment(x))
        .unwrap()
        .into_iter()
        .skip(1)
        .take(2)
        .map(|x| SPACEMATCH.splitn(&x, 3).nth(1).unwrap_or("").to_string())
        .collect::<Vec<_>>();
    (m[0].parse::<f64>().ok(), m[1].to_lowercase() == "yes")
}

#[test]
pub(crate) fn test_metatdata() {
    let test = "~VERSION INFORMATION
    VERS.                          2.0 :   CWLS LOG ASCII STANDARD -VERSION 2.0
    WRAP.                          NO  :   ONE LINE PER DEPTH STEP
    ~WELL INFORMATION";
    assert_eq!((Some(2.0), false), metadata(test));

    let test1 = "~VERSION INFORMATION
    VERS.                           :   CWLS LOG ASCII STANDARD -VERSION 2.0
    WRAP.                           :   ONE LINE PER DEPTH STEP
    ~WELL INFORMATION";
    assert_eq!((None, false), metadata(test1));

    let test2 = "# LAS format log file from PETREL
    # Project units are specified as depth units
    #==================================================================
    ~Version Information
    VERS.   2.0:
    WRAP.   NO:
    #==================================================================";
    assert_eq!((Some(2.0), false), metadata(test2));
}

pub(crate) fn property(raw_str: &str, key: &str) -> HashMap<String, WellProps> {
    let lines = raw_str
        .split(key)
        .nth(1)
        .unwrap()
        .split('~')
        .nth(0)
        .map(|x| remove_comment(x))
        .unwrap()
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();

    let mut prop_hash: HashMap<String, WellProps> = HashMap::new();

    lines.into_iter().for_each(|line| {
        let root = DOT_IN_SPACES.replace_all(line, "   none   ");
        let title = DOT_OR_SPACES
            .splitn(&root, 2)
            .nth(0)
            .unwrap_or("UNKNOWN")
            .trim();
        let unit = SPACES
            .splitn(
                LETTERS_AND_DOT_IN_SPACES
                    .splitn(&root, 2)
                    .nth(1)
                    .unwrap_or(""),
                2,
            )
            .nth(0)
            .map(|x| if x.trim() == "none" { "" } else { x })
            .unwrap_or("");
        let description = root.split(':').nth(1).unwrap_or("").trim();
        let description = DIGITS_AND_SPACES.replace_all(description, "");

        let value = LETTERS_IN_SPACES
            .split(root.split(":").nth(0).unwrap_or(""))
            .collect::<Vec<_>>();
        let value = {
            if value.len() > 2 {
                value[value.len() - 2].trim()
            } else {
                value[value.len() - 1].trim()
            }
        };
        prop_hash.insert(
            title.to_string(),
            WellProps {
                unit: unit.to_string(),
                description: description.to_string(),
                value: value.to_string(),
            },
        );
    });
    prop_hash
}

#[test]
fn test_property() {
    let test = "~Well
    STRT .m       1499.879000 :
    STOP .m       2416.379000 :
    STEP .m     0.000000 :
    NULL .        -999.250000 :
    COMP.           : COMPANY
    WELL.  A10   : WELL
    FLD.            : FIELD
    LOC.            : LOCATION
    SRVC.           : SERVICE COMPANY
    DATE.  Tuesday, July 02 2002 10:57:24   : DATE
    PROV.           : PROVINCE
    UWI.   02c62c82-552d-444d-bf6b-69cd07376368   : UNIQUE WELL ID
    API.            : API NUMBER
    #==================================================================
    ~Curve
    DEPT .m                   : DEPTH
    Perm .m                   :
    Gamma .m                  :
    Porosity .m               :
    Fluvialfacies .m          :
    NetGross .m               :
    ~Parameter
    #==================================================================
    ~Ascii";
    let result = property(test, "~W");
    assert_eq!(
        &WellProps::new("m", "", "1499.879000"),
        result.get("STRT").unwrap()
    );
    assert_eq!(
        &WellProps::new("", "", "-999.250000"),
        result.get("NULL").unwrap()
    );
    let result = property(test, "~C");
    assert_eq!(
        &WellProps::new("m", "DEPTH", ""),
        result.get("DEPT").unwrap()
    );
    assert_eq!(&WellProps::new("m", "", ""), result.get("Gamma").unwrap());
}
