use serde_json::{Result, Value};
use std::fs;

fn main() {
    let ini_path: &str = "ini.json";

    let ini_str: String = fs::read_to_string(ini_path)
        .expect("Error reading ini.json");
    let ini_vars: Value = serde_json::from_str(&ini_str)
        .expect("Error interpreting ini.json");

    let pulse_path = ini_vars["pulse_path"].as_str().expect("Error interpreting pulse_path variable");

    let pulse_str: String = fs::read_to_string(pulse_path)
        .expect("Error reading pulse shape");

    println!("{}", pulse_str)
}
