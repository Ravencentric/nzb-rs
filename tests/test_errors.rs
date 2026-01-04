use std::io;
use std::path::{Path, PathBuf};

use nzb_rs::{FileAttributeKind, Nzb, ParseNzbError, ParseNzbFileError};

fn get_file(name: &str) -> PathBuf {
    std::fs::canonicalize(file!())
        .unwrap()
        .parent()
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
            assert_eq!(source.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(file, get_file("invalid_gzipped_nzb.nzb.gz"));
        }
        _ => panic!(),
    }
}

#[test]
fn test_bad_nzb_file() {
    let file = get_file("invalid_bytes.nzb");
    let nzb = Nzb::parse_file(file);
    assert!(nzb.is_err());

    let error = nzb.unwrap_err();

    match error {
        ParseNzbFileError::Io { source, file } => {
            assert_eq!(source.kind(), io::ErrorKind::InvalidData);
            assert_eq!(file, get_file("invalid_bytes.nzb"));
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
    assert_eq!(
        nzb.unwrap_err(),
        ParseNzbError::FileAttribute {
            attribute: FileAttributeKind::Poster
        }
    );
}

#[test]
fn test_file_with_bad_date() {
    let bad_date = r#"
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

    let nzb = Nzb::parse(bad_date);
    assert_eq!(
        nzb.unwrap_err(),
        ParseNzbError::FileAttribute {
            attribute: FileAttributeKind::Date
        }
    );
}

#[test]
fn test_file_with_missing_subject() {
    let no_subject = r#"
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

    let nzb = Nzb::parse(no_subject);
    assert_eq!(
        nzb.unwrap_err(),
        ParseNzbError::FileAttribute {
            attribute: FileAttributeKind::Subject
        }
    );
}

#[test]
fn test_nzb_with_only_par2_files() {
    let nzb = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
    <nzb xmlns="http://www.newzbin.com/DTD/2003/nzb">
        <file poster="Joe Bloggs &lt;bloggs@nowhere.example&gt;" date="1590927494" subject="[1/1] - &quot;[Baz] Foobar - 09 (1080p) [0000BEEF].par2&quot; yEnc (1/1) 388">
            <groups>
                <group>alt.binaries.boneless</group>
            </groups>
            <segments>
                <segment bytes="581" number="1">MtUwAvUsIaGzDhHhJgXsXaFv-1690927494721@nyuu</segment>
            </segments>
        </file>
    </nzb>
    "#
    .trim();

    let nzb = Nzb::parse(nzb);

    assert!(nzb.is_err());
    assert_eq!(nzb.unwrap_err(), ParseNzbError::OnlyPar2Files)
}
