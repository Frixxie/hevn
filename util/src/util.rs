use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Deserialize, Clone, Debug)]
pub struct Collector {
    room: String,
    url: String,
}

impl Collector {
    pub fn new(room: String, url: String) -> Self {
        Self { room, url }
    }

    pub fn from_json(json: &Path) -> Vec<Self> {
        let file = std::fs::read_to_string(json).unwrap();
        serde_json::from_str(&file).unwrap()
    }

    pub fn room(&self) -> String {
        self.room.clone()
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct EnvData {
    room: String,
    temperature: i16,
    humidity: u16,
}

impl EnvData {
    pub fn new(room: String, temperature: i16, humidity: u16) -> Self {
        Self {
            room,
            temperature,
            humidity,
        }
    }
}

impl fmt::Display for EnvData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.room, self.temperature, self.humidity)
    }
}
