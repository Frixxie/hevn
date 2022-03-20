use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct MyCollector {
    pub url: String,
    pub room: String,
}

impl MyCollector {
    pub fn from_json(json: &Path) -> Vec<Self> {
        let file = std::fs::read_to_string(json).unwrap();
        serde_json::from_str(&file).unwrap()
    }
}
