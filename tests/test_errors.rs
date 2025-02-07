use nzb_rs::{InvalidNzbError, Nzb};
use std::path::{Path, PathBuf};

fn get_file(name: &str) -> PathBuf {
    Path::new(file!())
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap()
        .join("nzbs")
        .join(name)
        .to_path_buf()
}

#[test]
fn test_invalid_xml() {
    let invalid_xml = r#"
    <?xml version="1.0" encoding="iso-8859-1" ?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <head>
            <meta type="title">Your File!</meta>
            <meta type="password">secret</meta>
            <meta type="tag">HD</meta>
            <meta type="category">TV</meta>
        </head>
        <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
            <groups>
                <group>alt.binaries.newzbin</group>
                <group>alt.binaries.mojo</group>
            </groups>
            <segments>
                <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
            </segments>
        </file>
    "#;

    let nzb = Nzb::parse(invalid_xml);
    assert!(nzb.is_err());
}

#[test]
fn test_valid_xml_but_invalid_nzb() {
    let valid_xml_but_invalid_nzb = r#"
    <?xml version="1.0" encoding="iso-8859-1" ?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <head>
            <meta type="title">Your File!</meta>
        </head>
        <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
            <groups>
                <group>alt.binaries.newzbin</group>
                <group>alt.binaries.mojo</group>
            </groups>
        </file>
    </nzb>
    "#;

    let nzb = Nzb::parse(valid_xml_but_invalid_nzb);
    assert!(nzb.is_err());
}

#[test]
fn test_malformed_files() {
    let file = get_file("malformed_files.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());
    assert_eq!(
        nzb.unwrap_err(),
        InvalidNzbError::new("Missing or malformed <file>...</file>!")
    );
}

#[test]
fn test_malformed_files2() {
    let file = get_file("malformed_files2.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());
    assert_eq!(
        nzb.unwrap_err(),
        InvalidNzbError::new("Missing or malformed <groups>...</groups>!")
    );
}

#[test]
fn test_malformed_groups() {
    let file = get_file("malformed_groups.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());
    assert_eq!(
        nzb.unwrap_err(),
        InvalidNzbError::new("Missing or malformed <groups>...</groups>!")
    );
}

#[test]
fn test_malformed_segments() {
    let file = get_file("malformed_segments.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());
    assert_eq!(
        nzb.unwrap_err(),
        InvalidNzbError::new("Missing or malformed <segments>...</segments>!")
    );
}


#[test]
fn test_bad_gzip_file() {
    let file = get_file("invalid_gzipped_nzb.nzb.gz");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());
    assert_eq!(
        nzb.unwrap_err(),
        InvalidNzbError::new("invalid gzip header")
    );
}
