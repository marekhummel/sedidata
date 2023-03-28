use json;
use std::{fs::File, io::Read};

use crate::service::gameapi::parsing::champselect::parse_champselect_info;

pub fn main() {
    let mut file = File::open("data/ChampSelect.json").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf);
    let json = json::parse(buf.as_str()).unwrap();

    let x = parse_champselect_info(&json);
    println!("{:#?}", x);
}
