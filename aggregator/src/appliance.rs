use serde::Deserialize;
use std::path::Path;
use util::{ShellyS1, ShellyS1Error, ShellyStatus, SmartAppliance};

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

pub struct Heater {
    room: String,
    device: ShellyS1,
}

impl Heater {
    pub fn new(url: String, room: String) -> Self {
        let device = ShellyS1::new(room.clone(), url);
        Self { room, device }
    }

    pub fn get_room(&self) -> &str {
        &self.room
    }

    pub async fn get_status(&self) -> Result<ShellyStatus, ShellyS1Error> {
        self.device.get_status().map_err(|e| e)
    }

    pub async fn turn_on(&self) -> Result<String, ShellyS1Error> {
        match self.device.turn_on() {
            Ok(_) => Ok(format!("{} turned on", self.room)),
            Err(e) => Err(e),
        }
    }

    pub async fn turn_off(&self) -> Result<String, ShellyS1Error> {
        match self.device.turn_off() {
            Ok(_) => Ok(format!("Turned off {}", self.room)),
            Err(e) => Err(e),
        }
    }
}
