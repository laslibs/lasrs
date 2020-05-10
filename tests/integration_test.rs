use lasrs::Las;

#[test]
fn version_test() {
    let las = Las::new("./sample/example1.las");
    assert_eq!(las.version(), 2.0);
}
#[test]
fn wrap_test() {
    let las = Las::new("./sample/example1.las");
    assert_eq!(las.wrap(), false);
}
#[test]
fn headers_test() {
    let las = Las::new("./sample/example1.las");
    assert_eq!(
        vec!["DEPT", "DT", "RHOB", "NPHI", "SFLU", "SFLA", "ILM", "ILD"],
        las.headers()
    );
    let las = Las::new("./sample/A10.las");
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
    let las = Las::new("./sample/example1.las");
    let expected: Vec<Vec<f64>> = vec![
        vec![1670.0, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
        vec![1669.875, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
        vec![1669.75, 123.45, 2550.0, 0.45, 123.45, 123.45, 110.2, 105.6],
        vec![
            1669.745, 123.45, 2550.0, -999.25, 123.45, 123.45, 110.2, 105.6,
        ],
    ];
    assert_eq!(expected, las.data());
    let las = Las::new("./sample/A10.las");
    let expected: Vec<Vec<f64>> = vec![
        vec![1499.879, -999.25, -999.25, -999.25, -999.25, 0.0],
        vec![1500.129, -999.25, -999.25, -999.25, -999.25, 0.0],
        vec![1500.629, -999.25, -999.25, -999.25, -999.25, 0.0],
        vec![1501.129, -999.25, -999.25, 0.270646, 0.0, 0.0],
        vec![1501.629, 124.5799, 78.869453, 0.267428, 0.0, 0.0],
    ];
    assert_eq!(expected, &las.data()[0..5]);
}

#[test]
fn test_column() {
    let las = Las::new("./sample/example1.las");
    assert_eq!(
        vec![1670.0, 1669.875, 1669.75, 1669.745],
        las.column("DEPT")
    );
}

#[test]
fn test_counts() {
    let las = Las::new("./sample/example1.las");
    assert_eq!(las.column_count(), 8);
    assert_eq!(las.row_count(), 4);
}
#[test]
fn test_header_and_desc() {
    let las = Las::new("./sample/example1.las");
    let mut expected = vec![
        ("DEPT", "DEPTH"),
        ("DT", "SONIC TRANSIT TIME"),
        ("RHOB", "BULK DENSITY"),
        ("NPHI", "NEUTRON POROSITY"),
        ("SFLU", "SHALLOW RESISTIVITY"),
        ("SFLA", "SHALLOW RESISTIVITY"),
        ("ILM", "MEDIUM RESISTIVITY"),
        ("ILD", "DEEP RESISTIVITY"),
    ]
    .into_iter()
    .map(|a| (a.0.to_string(), a.1.to_string()))
    .collect::<Vec<_>>();
    assert_eq!(
        expected.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()),
        las.headers_and_desc()
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    );
}

#[test]
fn other_test() {
    let las = Las::new("./sample/example1.las");
    let expected = [
        "Note: The logging tools became stuck at 625 metres causing the data",
        "between 625 metres and 615 metres to be invalid.",
    ];
    assert_eq!(las.other(), expected.join("\n").to_string());

    let las = Las::new("./sample/A10.las");
    let expected = "".to_string();
    assert_eq!(las.other(), expected);
}

#[test]
fn csv_test() {
    use std::fs;
    let las = Las::new("./sample/example1.las");
    las.to_csv("example");
    let result = fs::read_to_string("example.csv").expect("File was not created");
    let expected = [
        "DEPT,DT,RHOB,NPHI,SFLU,SFLA,ILM,ILD",
        "1670,123.45,2550,0.45,123.45,123.45,110.2,105.6",
        "1669.875,123.45,2550,0.45,123.45,123.45,110.2,105.6",
        "1669.75,123.45,2550,0.45,123.45,123.45,110.2,105.6",
        "1669.745,123.45,2550,-999.25,123.45,123.45,110.2,105.6",
    ];
    assert_eq!(expected.join("\n").to_string(), result);
    fs::remove_file("example.csv").expect("Could not clean up after test");
}
