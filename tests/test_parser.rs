mod common;

use chrono::DateTime;
use common::get_nzb;
use nzb_rs::{File, Segment};

#[test]
fn test_spec_example() {
    let nzb = get_nzb("spec_example.nzb");
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
    assert!(nzb.file().has_extension("r01"));
    assert!(nzb.file().has_extension(".R01"));
    assert!(!nzb.file().has_extension("..r01"));
    assert!(nzb.has_extension("r01"));
    assert!(nzb.has_extension(".R01"));
    assert!(!nzb.has_extension("..r01"));
    assert_eq!(nzb.file().posted_at, DateTime::from_timestamp(1071674882, 0).unwrap());
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
        vec!["alt.binaries.mojo".to_string(), "alt.binaries.newzbin".to_string()]
    );
}

#[test]
fn test_big_buck_bunny() {
    let nzb = get_nzb("big_buck_bunny.nzb");

    assert_eq!(nzb.meta.title, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
    assert_eq!(nzb.meta.category, None);

    assert_eq!(nzb.files.len(), 5);
    assert!(!nzb.is_rar());
    assert!(!nzb.is_obfuscated());
    assert!(nzb.has_par2());
    assert_eq!(nzb.size(), 22_704_889);
    assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    assert_eq!(nzb.file().stem(), Some("Big Buck Bunny - S01E01"));
    assert_eq!(nzb.file().extension(), Some("mkv"));
    assert!(nzb.file().has_extension("mkv"));
    assert!(nzb.file().has_extension(".MKv"));
    assert!(!nzb.file().has_extension("..MKv"));
    assert!(nzb.has_extension("mkv"));
    assert!(nzb.has_extension(".MKv"));
    assert!(!nzb.has_extension("..MKv"));
    assert_eq!(nzb.file().posted_at, DateTime::from_timestamp(1706440708, 0).unwrap());

    assert_eq!(
        nzb.files.iter().map(|f| f.subject.clone()).collect::<Vec<_>>(),
        vec![
            "[1/5] - \"Big Buck Bunny - S01E01.mkv\" yEnc (1/24) 16981056",
            "[2/5] - \"Big Buck Bunny - S01E01.mkv.par2\" yEnc (1/1) 920",
            "[3/5] - \"Big Buck Bunny - S01E01.mkv.vol00+01.par2\" yEnc (1/2) 717788",
            "[4/5] - \"Big Buck Bunny - S01E01.mkv.vol01+02.par2\" yEnc (1/3) 1434656",
            "[5/5] - \"Big Buck Bunny - S01E01.mkv.vol03+04.par2\" yEnc (1/5) 2869192"
        ]
    );

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
    assert_eq!(nzb.par2_percentage().floor(), 22.0);
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
                Segment::new(
                    739668u32,
                    13u32,
                    "a4c15599055848dda1eff3b6b406fa78-8111735210252@example"
                ),
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

#[test]
fn test_valid_nzb_with_one_missing_segment() {
    let nzb = get_nzb("valid_nzb_with_one_missing_segment.nzb");

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
    assert!(nzb.file().has_extension("mkv"));
    assert!(nzb.file().has_extension(".MKv"));
    assert!(!nzb.file().has_extension("..MKv"));
    assert!(nzb.has_extension("mkv"));
    assert!(nzb.has_extension(".MKv"));
    assert!(!nzb.has_extension("..MKv"));
    assert_eq!(nzb.file().posted_at, DateTime::from_timestamp(1706440708, 0).unwrap());

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

#[test]
fn test_bad_subject() {
    let nzb = get_nzb("bad_subject.nzb");

    assert_eq!(nzb.file().name(), None);
    assert_eq!(nzb.file().stem(), None);
    assert_eq!(nzb.file().extension(), None);
    assert_eq!(nzb.file().is_par2(), false);
    assert_eq!(nzb.file().is_rar(), false);
    assert_eq!(nzb.is_rar(), false);
    assert_eq!(nzb.has_par2(), false);
    assert_eq!(nzb.is_obfuscated(), true);
}

#[test]
fn test_non_standard_meta() {
    let nzb = get_nzb("non_standard_meta.nzb");

    assert_eq!(nzb.meta.title, None);
    assert_eq!(nzb.meta.category, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
}

#[test]
fn test_no_meta() {
    let nzb = get_nzb("no_meta.nzb");

    assert_eq!(nzb.meta.title, None);
    assert_eq!(nzb.meta.category, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
}

#[test]
fn test_single_meta() {
    let nzb = get_nzb("single_meta.nzb");

    assert_eq!(nzb.meta.title, Some("title".to_string()));
    assert_eq!(nzb.meta.category, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
}

#[test]
fn test_nzb_with_no_head() {
    let nzb = get_nzb("nzb_with_no_head.nzb");

    assert_eq!(nzb.meta.title, None);
    assert_eq!(nzb.meta.category, None);
    assert!(nzb.meta.passwords.is_empty());
    assert!(nzb.meta.tags.is_empty());
}

#[test]
fn test_one_rar_file() {
    let nzb = get_nzb("one_rar_file.nzb");

    assert!(nzb.has_rar());
    assert!(!nzb.is_rar());
    assert!(!nzb.has_par2());
}

#[test]
fn test_multi_rar() {
    let nzb = get_nzb("multi_rar.nzb");

    assert!(nzb.has_rar());
    assert!(nzb.is_rar());
    assert!(!nzb.has_par2());
}
