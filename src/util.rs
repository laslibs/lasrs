use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref DOT_IN_SPACES: Regex = Regex::new("\\s*[.]\\s+").unwrap();
    pub(crate) static ref SPACES_AND_DOT: Regex = Regex::new("\\s+[.]").unwrap();
    static ref DOT_OR_SPACES: Regex = Regex::new("[.]|\\s+").unwrap();
    static ref LETTERS_AND_DOT_IN_SPACES: Regex = Regex::new("^\\w+\\s*[.]*s*").unwrap();
    static ref DIGITS_AND_SPACES: Regex = Regex::new("\\d+\\s*").unwrap();
    static ref LETTERS_IN_SPACES: Regex = Regex::new("\\s{2,}\\w*\\s{2,}").unwrap();
    pub(crate) static ref SPACES: Regex = Regex::new("\\s+").unwrap();
}

/// Wellprop represents an entry in every sections
/// excluding ~O, ~A and ~V (other, data and version sections respectively)
#[derive(Debug, PartialEq)]
pub struct WellProp {
    /// unit of measurement
    pub unit: String,
    /// entry description
    pub description: String,
    /// entry value
    pub value: String,
}

impl WellProp {
    /// Returns a Wellprop
    ///
    /// ## Arguments
    ///
    /// * `unit` - string slice
    /// * `description` - string slice
    /// * `value` - string slice
    ///
    /// ## Example
    /// ```
    /// use lasrs::WellProp;
    /// let well_prop = WellProp::new("DEGC", "BOTTOM HOLE TEMPERATURE", "35.5000");
    /// assert_eq!(well_prop.unit, "DEGC".to_owned());
    /// ```
    pub fn new(unit: &str, description: &str, value: &str) -> Self {
        Self {
            unit: unit.to_string(),
            description: description.to_string(),
            value: value.to_string(),
        }
    }
}

// Removes lines that starts with `#`, returns Vec<&str> of uncommented lines
pub(crate) fn remove_comment(raw_str: &str) -> Vec<&str> {
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

// Extracts version number and wrap mode
// Refers to whether a wrap around mode was used in the data section. If the wrap mode is
// false, there is no limit to the line length. If wrap mode is used, the depth value will be on its
// own line and all lines of data will be no longer than 80 characters (including carriage return
// and line feed).
pub(crate) fn metadata(raw_str: &str) -> Option<(Option<f64>, bool)> {
    lazy_static! {
        static ref SPACEMATCH: Regex = Regex::new(r"\s+|\s*:").unwrap();
    }
    let m = raw_str
        .split('~')
        .nth(1)
        .map(|x| remove_comment(x))?
        .into_iter()
        .skip(1)
        .take(2)
        .map(|x| SPACEMATCH.splitn(&x, 3).nth(1).unwrap_or("").to_string())
        .collect::<Vec<_>>();
    Some((m[0].parse::<f64>().ok(), m[1].to_lowercase() == "yes"))
}

// Returns all the WellProp in a section
// key - section signature, raw_str - string to extract them from
pub(crate) fn property(raw_str: &str, key: &str) -> Option<HashMap<String, WellProp>> {
    let lines = raw_str
        .split(key)
        .nth(1)?
        .split('~')
        .nth(0)
        .map(|x| remove_comment(x))?
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();

    let mut prop_hash: HashMap<String, WellProp> = HashMap::new();

    lines.into_iter().for_each(|line| {
        let root = DOT_IN_SPACES.replace(line, "   none   ");
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
        prop_hash.insert(title.to_string(), WellProp::new(unit, &description, value));
    });
    Some(prop_hash)
}

#[cfg(test)]
mod test {
    use super::*;

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
    #[test]
    fn test_metatdata() {
        let test = "~VERSION INFORMATION
        VERS.                          2.0 :   CWLS LOG ASCII STANDARD -VERSION 2.0
        WRAP.                          NO  :   ONE LINE PER DEPTH STEP
        ~WELL INFORMATION";
        assert_eq!(Some((Some(2.0), false)), metadata(test));
        let test1 = "~VERSION INFORMATION
        VERS.                           :   CWLS LOG ASCII STANDARD -VERSION 2.0
        WRAP.                           :   ONE LINE PER DEPTH STEP
        ~WELL INFORMATION";
        assert_eq!(Some((None, false)), metadata(test1));
        let test2 = "# LAS format log file from PETREL
        # Project units are specified as depth units
        #==================================================================
        ~Version Information
        VERS.   2.0:
        WRAP.   NO:
        #==================================================================";
        assert_eq!(Some((Some(2.0), false)), metadata(test2));
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
        let result = property(test, "~W").unwrap();
        assert_eq!(
            &WellProp::new("m", "", "1499.879000"),
            result.get("STRT").unwrap()
        );
        assert_eq!(
            &WellProp::new("", "", "-999.250000"),
            result.get("NULL").unwrap()
        );
        let result = property(test, "~C").unwrap();
        assert_eq!(
            &WellProp::new("m", "DEPTH", ""),
            result.get("DEPT").unwrap()
        );
        assert_eq!(&WellProp::new("m", "", ""), result.get("Gamma").unwrap());
    }

    #[test]
    fn test_() {
        let test = "~Well
	STRT    .M              1670.0000                :START DEPTH
	STOP    .M              1669.7500                :STOP DEPTH
	STEP    .M              -0.1250                  :STEP
	NULL    .               -999.25                  :NULL VALUE
	COMP    .       ANY OIL COMPANY INC.             :COMPANY
	WELL    .       ANY ET AL 12-34-12-34            :WELL
	FLD     .       WILDCAT                          :FIELD
	LOC     .       12-34-12-34W5M                   :LOCATION
	PROV    .       ALBERTA                          :PROVINCE
	SRVC    .       ANY LOGGING COMPANY INC.         :SERVICE COMPANY
	DATE    .       13-DEC-86                        :LOG DATE
	UWI     .       100123401234W500                 :UNIQUE WELL ID
    ";
        let result = property(test, "~W").unwrap();
        assert_eq!(
            &WellProp::new("", "COMPANY", "ANY OIL COMPANY INC."),
            result.get("COMP").unwrap()
        );
        assert_eq!(
            &WellProp::new("", "SERVICE COMPANY", "ANY LOGGING COMPANY INC."),
            result.get("SRVC").unwrap()
        );
    }
}
