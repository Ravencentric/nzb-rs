use nzb_rs::NZB;
use std::{env, fs};

pub fn get_nzb_string(file: &str) -> String {
    let path = env::current_dir().unwrap().join("tests").join("nzbs").join(file);
    return fs::read_to_string(path).unwrap();
}

#[allow(dead_code)]
pub fn get_nzb(nzb: &str) -> NZB {
    NZB::parse(get_nzb_string(nzb)).unwrap()
}
