use nzb_rs::{Nzb, ParseNzbError, ParseNzbFileError};
use pretty_assertions::assert_eq;
use std::io;
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
    assert_eq!(
        nzb.unwrap_err(),
        ParseNzbError::XmlSyntax {
            message: "the root node was opened but never closed".to_string()
        }
    )
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
    assert_eq!(nzb.unwrap_err(), ParseNzbError::SegmentsElement)
}

#[test]
fn test_malformed_files() {
    let file = get_file("malformed_files.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Parse { source } => {
            assert_eq!(source, ParseNzbError::FileElement)
        }
        _ => panic!(),
    }
}

#[test]
fn test_malformed_files2() {
    let file = get_file("malformed_files2.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Parse { source } => {
            assert_eq!(source, ParseNzbError::GroupsElement)
        }
        _ => panic!(),
    }
}

#[test]
fn test_malformed_groups() {
    let file = get_file("malformed_groups.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Parse { source } => {
            assert_eq!(source, ParseNzbError::GroupsElement)
        }
        _ => panic!(),
    }
}

#[test]
fn test_malformed_segments() {
    let file = get_file("malformed_segments.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Parse { source } => {
            assert_eq!(source, ParseNzbError::SegmentsElement)
        }
        _ => panic!(),
    }
}

#[test]
fn test_bad_gzip_file() {
    let file = get_file("invalid_gzipped_nzb.nzb.gz");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Gzip { source, file } => {
            assert_eq!(source.to_string(), "invalid gzip header".to_string());
            assert_eq!(file, get_file("invalid_gzipped_nzb.nzb.gz"));
        }
        _ => panic!(),
    }
}

#[test]
fn test_non_existent_file() {
    let nzb = Nzb::parse_file("i dont exist");
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Io { source, file } => {
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
            assert_eq!(file, Path::new("i dont exist"));
        }
        _ => panic!(),
    }
}

#[test]
fn test_file_with_missing_poster() {
    let no_poster = r#"
    <?xml version="1.0" encoding="iso-8859-1" ?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <head>
            <meta type="title">Your File!</meta>
            <meta type="password">secret</meta>
            <meta type="tag">HD</meta>
            <meta type="category">TV</meta>
        </head>
        <file date="1071674882" subject="Here's your file!  abc-mr2a.r01 (1/2)">
            <groups>
                <group>alt.binaries.newzbin</group>
                <group>alt.binaries.mojo</group>
            </groups>
            <segments>
                <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
            </segments>
        </file>
    </nzb>
    "#
    .trim();

    let nzb = Nzb::parse(no_poster);
    assert!(nzb.is_err_and(|e| e
        == ParseNzbError::FileAttribute {
            attribute: "poster".to_string()
        }))
}

#[test]
fn test_file_with_bad_date() {
    let no_poster = r#"
    <?xml version="1.0" encoding="iso-8859-1" ?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <head>
            <meta type="title">Your File!</meta>
            <meta type="password">secret</meta>
            <meta type="tag">HD</meta>
            <meta type="category">TV</meta>
        </head>
        <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="blah" subject="Here's your file!  abc-mr2a.r01 (1/2)">
            <groups>
                <group>alt.binaries.newzbin</group>
                <group>alt.binaries.mojo</group>
            </groups>
            <segments>
                <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
            </segments>
        </file>
    </nzb>
    "#.trim();

    let nzb = Nzb::parse(no_poster);
    assert!(nzb.is_err_and(|e| e
        == ParseNzbError::FileAttribute {
            attribute: "date".to_string()
        }))
}

#[test]
fn test_file_with_missing_subject() {
    let no_poster = r#"
    <?xml version="1.0" encoding="iso-8859-1" ?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <head>
            <meta type="title">Your File!</meta>
            <meta type="password">secret</meta>
            <meta type="tag">HD</meta>
            <meta type="category">TV</meta>
        </head>
        <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1071674882">
            <groups>
                <group>alt.binaries.newzbin</group>
                <group>alt.binaries.mojo</group>
            </groups>
            <segments>
                <segment bytes="102394" number="1">123456789abcdef@news.newzbin.com</segment>
                <segment bytes="4501" number="2">987654321fedbca@news.newzbin.com</segment>
            </segments>
        </file>
    </nzb>
    "#
    .trim();

    let nzb = Nzb::parse(no_poster);
    assert!(nzb.is_err_and(|e| e
        == ParseNzbError::FileAttribute {
            attribute: "subject".to_string()
        }))
}
