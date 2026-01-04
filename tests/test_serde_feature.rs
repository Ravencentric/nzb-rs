#![cfg(feature = "serde")]
use std::path::PathBuf;

use nzb_rs::Nzb;
use rstest::rstest;

fn get_file(name: &str) -> PathBuf {
    PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("nzbs")
        .join(name)
}

#[rstest]
#[case::spec_example_nzb(get_file("spec_example.nzb"))]
#[case::spec_example_nzb_gz(get_file("spec_example.nzb.gz"))]
fn test_serde_feature(#[case] nzb_file: PathBuf) {
    let original = Nzb::parse_file(nzb_file).unwrap();
    let serialized = serde_json::to_string(&original).unwrap();
    let nzb = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, nzb);
}

#[rstest]
#[case::valid_nzb_with_bad_segments(get_file("valid_nzb_with_bad_segments.nzb"))]
#[case::valid_nzb_with_bad_segments_gz(get_file("valid_nzb_with_bad_segments.nzb.gz"))]
fn test_valid_nzb_with_one_missing_segment(#[case] nzb_file: PathBuf) {
    let original = Nzb::parse_file(nzb_file).unwrap();
    let serialized = serde_json::to_string(&original).unwrap();
    let nzb = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, nzb);
}
