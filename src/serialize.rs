use std::fs;
use serde::{Serialize, Deserialize};

pub fn serialize<T: Serialize>(data: &Vec<T>, path: &str) {
    let serialized_data = serde_json::to_string_pretty(data).unwrap();
    fs::write(path, serialized_data).unwrap();
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(path: &str) -> Result<T, serde_json::Error> {
    let data = fs::read_to_string(path).unwrap();
    let deserialize_data = serde_json::from_str(&data);
    deserialize_data
}
