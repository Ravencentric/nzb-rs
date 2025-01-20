#![cfg(feature = "serde")]

mod common;

use chrono::DateTime;
use common::get_nzb;
use nzb_rs::{File, Segment};
use serde_json;

#[test]
fn test_serde_feature() {
    let original = get_nzb("spec_example.nzb");
    let serialized = serde_json::to_string(&original).unwrap();
    let nzb = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, nzb);

    assert_eq!(nzb.meta.title, Some("Your File!".to_string()));
    assert_eq!(nzb.meta.passwords, vec!["secret"]);
    assert_eq!(nzb.meta.tags, vec!["HD"]);
    assert_eq!(nzb.meta.category, Some("TV".to_string()));
    assert_eq!(nzb.files.len(), 1);
    assert!(nzb.is_rar());
    assert!(nzb.is_obfuscated());
    assert_eq!(nzb.file().name(), Some("abc-mr2a.r01"));
    assert_eq!(nzb.file().stem(), Some("abc-mr2a"));
    assert_eq!(nzb.file().extension(), Some("r01"));
    assert_eq!(nzb.size(), 106_895);
    assert_eq!(nzb.files[0].segments.len(), 2);
    assert_eq!(
        nzb.files[0].segments,
        vec![
            Segment {
                size: 102_394,
                number: 1,
                message_id: "123456789abcdef@news.newzbin.com".to_string(),
            },
            Segment {
                size: 4_501,
                number: 2,
                message_id: "987654321fedbca@news.newzbin.com".to_string(),
            }
        ]
    );
    assert_eq!(
        nzb.files[0].groups,
        vec!["alt.binaries.newzbin".to_string(), "alt.binaries.mojo".to_string(),]
    );
}

#[test]
fn test_valid_nzb_with_one_missing_segment() {
    let original = get_nzb("valid_nzb_with_one_missing_segment.nzb");
    let serialized = serde_json::to_string(&original).unwrap();
    let nzb = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, nzb);

    assert_eq!(nzb.meta.title, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
    assert_eq!(nzb.meta.category, None);

    assert_eq!(nzb.files.len(), 5);
    assert!(!nzb.is_rar());
    assert!(!nzb.is_obfuscated());
    assert!(nzb.has_par2());
    assert_eq!(nzb.size(), 21_965_221);
    assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    assert_eq!(nzb.file().stem(), Some("Big Buck Bunny - S01E01"));
    assert_eq!(nzb.file().extension(), Some("mkv"));

    assert_eq!(
        nzb.filenames(),
        vec![
            "Big Buck Bunny - S01E01.mkv",
            "Big Buck Bunny - S01E01.mkv.par2",
            "Big Buck Bunny - S01E01.mkv.vol00+01.par2",
            "Big Buck Bunny - S01E01.mkv.vol01+02.par2",
            "Big Buck Bunny - S01E01.mkv.vol03+04.par2"
        ]
    );
    // assert_eq!(
    //     nzb.filestems(),
    //     vec![
    //         "Big Buck Bunny - S01E01",
    //         "Big Buck Bunny - S01E01.mkv",
    //         "Big Buck Bunny - S01E01.mkv.vol00+01",
    //         "Big Buck Bunny - S01E01.mkv.vol01+02",
    //         "Big Buck Bunny - S01E01.mkv.vol03+04"
    //     ]
    // );
    // assert_eq!(nzb.extensions(), vec!["mkv", "par2"]);
    assert_eq!(nzb.posters(), vec!["John <nzb@nowhere.example>"]);
    assert_eq!(nzb.groups(), vec!["alt.binaries.boneless"]);
    assert_eq!(nzb.par2_size(), 5_183_128);
    assert_eq!(nzb.par2_percentage().floor(), 23.0);
    assert_eq!(
        nzb.file(),
        &File::new(
            "John <nzb@nowhere.example>",
            DateTime::from_timestamp(1706440708, 0).unwrap(),
            r#"[1/5] - "Big Buck Bunny - S01E01.mkv" yEnc (1/24) 16981056"#,
            vec!["alt.binaries.boneless"],
            vec![
                Segment::new(
                    739067u32,
                    1u32,
                    "9cacde4c986547369becbf97003fb2c5-9483514693959@example"
                ),
                Segment::new(
                    739549u32,
                    2u32,
                    "70a3a038ce324e618e2751e063d6a036-7285710986748@example"
                ),
                Segment::new(
                    739728u32,
                    3u32,
                    "a209875cefd44440aa91590508b48f5b-4625756912881@example"
                ),
                Segment::new(
                    739664u32,
                    4u32,
                    "44057720ed4e45e4bce21d53249d03f8-8250738040266@example"
                ),
                Segment::new(
                    739645u32,
                    5u32,
                    "cfc13d14583c484483aa49ac420bad27-9491395432062@example"
                ),
                Segment::new(
                    739538u32,
                    6u32,
                    "5e90857531be401e9d0b632221fe2fb7-9854527985639@example"
                ),
                Segment::new(
                    739708u32,
                    7u32,
                    "c33a2bba79494840a09d750b19d3b287-2550637855678@example"
                ),
                Segment::new(
                    739490u32,
                    8u32,
                    "38006019d94f4ecc8f19c389c00f1ebe-7841585708380@example"
                ),
                Segment::new(
                    739667u32,
                    9u32,
                    "b75a2425bef24fd5affb00dc3db789f6-7051027232703@example"
                ),
                Segment::new(
                    739540u32,
                    10u32,
                    "79a027e3bfde458ea2bd0db1632fc84e-7270120407913@example"
                ),
                Segment::new(
                    739657u32,
                    11u32,
                    "fb2bd74e1257487a9240ef0cf81765cc-7147741101314@example"
                ),
                Segment::new(
                    739647u32,
                    12u32,
                    "d39ca8be78c34e3fa6f3211f1b397b3a-4725950858191@example"
                ),
                // 13th Segment is missing here
                Segment::new(
                    739721u32,
                    14u32,
                    "2f1cec363ed24584b4127af86ac312ad-7204153818612@example"
                ),
                Segment::new(
                    739740u32,
                    15u32,
                    "30ff3514896543a8ac91ec80346a5d40-9134304686352@example"
                ),
                Segment::new(
                    739538u32,
                    16u32,
                    "1f75cfa20d884b5b972cfd2e9ebef249-8919850122587@example"
                ),
                Segment::new(
                    739646u32,
                    17u32,
                    "8e22b0f973de4393a0a30ab094565316-6722799721412@example"
                ),
                Segment::new(
                    739610u32,
                    18u32,
                    "faddf83650cc4de1a8bee68cffca40a1-5979589815618@example"
                ),
                Segment::new(
                    739514u32,
                    19u32,
                    "6b8c23e43d4240da812b547babdc0423-6409257710918@example"
                ),
                Segment::new(
                    739920u32,
                    20u32,
                    "802bd0dcef134ac690044e0a09fece60-8492061912475@example"
                ),
                Segment::new(
                    739634u32,
                    21u32,
                    "efc4b3966a1f4b7787677e9e9a214727-5444471572012@example"
                ),
                Segment::new(
                    739691u32,
                    22u32,
                    "247efca709114fd181bcaef0f487925f-4076317880026@example"
                ),
                Segment::new(
                    739638u32,
                    23u32,
                    "665d9fc5edba4faca68ae835b702b4c7-9814601723860@example"
                ),
                Segment::new(
                    510541u32,
                    24u32,
                    "962fddf3e07444988731b52aeaa9b2aa-1283919353788@example"
                ),
            ]
        )
    )
}
