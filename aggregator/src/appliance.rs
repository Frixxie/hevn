use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct MyCollector {
    url: String,
    room: String,
}

impl MyCollector {
    pub fn from_json(json: &Path) -> Vec<Self> {
        let file = std::fs::read_to_string(json).unwrap();
        serde_json::from_str(&file).unwrap()
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_room(&self) -> &str {
        &self.room
    }
}
