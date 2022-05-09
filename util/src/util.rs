use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

//Different kinds of appliences currently supported
pub enum Appliences {
    ShellyS1,
    Collector,
}

pub trait SmartInfo {
    fn app_type(&self) -> Appliences;
}

#[async_trait]
pub trait SmartAppliance: SmartInfo {
    type Status;
    type Error: std::error::Error + Default;
    /// This functions returns the current status of the appliance
    async fn get_status(&self) -> Result<Self::Status, Self::Error>;
    /// This function turns the appliance on
    async fn turn_on(&self) -> Result<(), Self::Error> {
        Err(Self::Error::default())
    }
    /// This function turns the appliance off
    async fn turn_off(&self) -> Result<(), Self::Error> {
        Err(Self::Error::default())
    }
}

/// Data type for the data the collector uses
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EnvData {
    pub room: String,
    pub temperature: i16,
    pub humidity: u16,
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
        write!(
            f,
            "{},{},{}",
            self.room,
            (self.temperature as f32) / 10.0,
            (self.humidity as f32) / 10.0
        )
    }
}

#[derive(Debug, Clone)]
pub struct Collector {
    room: String,
    url: String,
    client: Client,
}

impl Collector {
    pub fn new(room: String, url: String) -> Self {
        Self {
            room,
            url,
            client: Client::new(),
        }
    }

    pub fn room(&self) -> String {
        self.room.clone()
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub async fn read(&self) -> Result<EnvData, Box<CollectorError>> {
        let res = self
            .client
            .get(format!("{}/read", &self.url()))
            .send()
            .await
            .map_err(|e| CollectorError {
                error: Some(Box::new(e)),
            })?;
        Ok(res.json().await.map_err(|e| CollectorError {
            error: Some(Box::new(e)),
        })?)
    }
}

#[derive(Debug, Default)]
pub struct CollectorError {
    error: Option<Box<dyn std::error::Error>>,
}

impl std::error::Error for CollectorError {}

impl fmt::Display for CollectorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.error {
            Some(e) => write!(f, "{}", e),
            None => write!(f, "Unknown error"),
        }
    }
}

impl SmartInfo for Collector {
    fn app_type(&self) -> Appliences {
        Appliences::Collector
    }
}

#[async_trait]
impl SmartAppliance for Collector {
    type Status = EnvData;
    type Error = CollectorError;

    async fn get_status(&self) -> Result<Self::Status, Self::Error> {
        let res = self
            .client
            .get(format!("{}/data", &self.url()))
            .send()
            .await
            .map_err(|e| CollectorError {
                error: Some(Box::new(e)),
            })?;
        Ok(res.json().await.map_err(|e| CollectorError {
            error: Some(Box::new(e)),
        })?)
    }

    async fn turn_on(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn turn_off(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ShellyS1 {
    room: String,
    url: String,
    client: Client,
}

impl ShellyS1 {
    pub fn new(room: String, url: String) -> Self {
        Self {
            room,
            url,
            client: Client::new(),
        }
    }
}

impl fmt::Display for ShellyS1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.room, self.url,)
    }
}

#[derive(Debug, Serialize)]
pub struct ShellyStatus {
    is_on: bool,
    has_timer: bool,
    timer_started: u32,
    timer_duration: u32,
    timer_remaining: u32,
    overpower: bool,
    power: f32,
    meter_overpower: f32,
    timestamp: u32,
    temperature: f32,
}

impl Default for ShellyStatus {
    fn default() -> Self {
        Self {
            is_on: false,
            has_timer: false,
            timer_started: 0,
            timer_duration: 0,
            timer_remaining: 0,
            overpower: false,
            power: 0.0,
            meter_overpower: 0.0,
            timestamp: 0,
            temperature: 0.0,
        }
    }
}

impl fmt::Display for ShellyStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
            self.is_on,
            self.has_timer,
            self.timer_started,
            self.timer_duration,
            self.timer_remaining,
            self.overpower,
            self.power,
            self.meter_overpower,
            self.timestamp,
            self.temperature
        )
    }
}

impl SmartInfo for ShellyS1 {
    fn app_type(&self) -> Appliences {
        Appliences::ShellyS1
    }
}

#[derive(Debug, Default)]
pub struct ShellyS1Error {
    error: Option<Box<dyn std::error::Error>>,
}

impl std::error::Error for ShellyS1Error {}

impl fmt::Display for ShellyS1Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.error {
            Some(e) => write!(f, "{}", e),
            None => write!(f, "Unknown error"),
        }
    }
}

#[async_trait]
impl SmartAppliance for ShellyS1 {
    type Status = ShellyStatus;
    type Error = ShellyS1Error;

    async fn get_status(&self) -> Result<Self::Status, Self::Error> {
        let url = format!("http://{}/status", self.url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ShellyS1Error {
                error: Some(Box::new(e)),
            })?;
        let status: Value = response.json().await.map_err(|e| ShellyS1Error {
            error: Some(Box::new(e)),
        })?;
        Ok(ShellyStatus {
            is_on: status["relays"][0]["ison"].as_bool().unwrap_or(false),
            has_timer: status["relays"][0]["has_timer"].as_bool().unwrap_or(false),
            timer_started: status["relays"][0]["timer_started"].as_u64().unwrap_or(0) as u32,
            timer_duration: status["relays"][0]["timer_duration"].as_u64().unwrap_or(0) as u32,
            timer_remaining: status["relays"][0]["timer_remaining"].as_u64().unwrap_or(0) as u32,
            overpower: status["relays"][0]["overpower"].as_bool().unwrap_or(false),
            power: status["meters"][0]["power"].as_f64().unwrap_or(0.0) as f32,
            meter_overpower: status["meters"][0]["overpower"].as_f64().unwrap_or(0.0) as f32,
            timestamp: status["meters"][0]["timestamp"].as_u64().unwrap_or(0) as u32,
            temperature: status["temperature"].as_f64().unwrap_or(0.0) as f32,
        })
    }

    async fn turn_on(&self) -> Result<(), Self::Error> {
        let url = format!("http://{}/relay/0?turn=on", self.url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ShellyS1Error {
                error: Some(Box::new(e)),
            })?;
        Ok(())
    }

    async fn turn_off(&self) -> Result<(), Self::Error> {
        let url = format!("http://{}/relay/0?turn=off", self.url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ShellyS1Error {
                error: Some(Box::new(e)),
            })?;
        Ok(())
    }
}
