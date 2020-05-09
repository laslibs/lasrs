use regex::Regex;

pub fn remove_comment(raw_str: &str) -> Vec<String> {
    raw_str
        .lines()
        .filter_map(|x| {
            if x.trim().starts_with("#") {
                None
            } else {
                Some(x.trim().to_string())
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

pub fn metadata(raw_str: &str) -> (Option<f64>, bool) {
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
fn test_metatdata() {
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

pub fn property(key: &str) {}
