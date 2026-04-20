#![cfg(feature = "serde")]
use std::path::PathBuf;

use chrono::DateTime;
use nzb_rs::{File, Files, Nzb, Parity, ParseNzbError, Segment};
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

#[test]
fn test_files_deserialize_rejects_empty_collections() {
    let error = serde_json::from_str::<Files>("[]").unwrap_err();
    assert!(error.to_string().contains(&ParseNzbError::FileElement.to_string()));
}

#[test]
fn test_files_deserialize_rejects_only_par2_collections() {
    let par2_only = vec![File::new(
        "John <nzb@nowhere.example>",
        DateTime::from_timestamp(1706440708, 0).unwrap(),
        r#"[1/1] - "Big Buck Bunny - S01E01.mkv.par2" yEnc (1/1) 920"#,
        vec!["alt.binaries.boneless"],
        vec![Segment::new(
            920,
            1,
            "9cacde4c986547369becbf97003fb2c5-9483514693959@example",
        )],
    )];

    let serialized = serde_json::to_string(&par2_only).unwrap();
    let error = serde_json::from_str::<Files>(&serialized).unwrap_err();
    assert!(error.to_string().contains("non-`.par2` entries only"));
}

#[test]
fn test_parity_deserialize_rejects_payload_collections() {
    let payload_only = vec![File::new(
        "John <nzb@nowhere.example>",
        DateTime::from_timestamp(1706440708, 0).unwrap(),
        r#"[1/1] - "Big Buck Bunny - S01E01.mkv" yEnc (1/1) 920"#,
        vec!["alt.binaries.boneless"],
        vec![Segment::new(
            920,
            1,
            "9cacde4c986547369becbf97003fb2c5-9483514693959@example",
        )],
    )];

    let serialized = serde_json::to_string(&payload_only).unwrap();
    let error = serde_json::from_str::<Parity>(&serialized).unwrap_err();
    assert!(error.to_string().contains("`.par2` files only"));
}
