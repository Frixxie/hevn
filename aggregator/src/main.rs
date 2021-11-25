use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Collector {
    room: String,
    url: String,
}

impl Collector {
    fn from_json(json: &PathBuf) -> Vec<Self> {
        let file = std::fs::read_to_string(json).unwrap();
        serde_json::from_str(&file).unwrap()
    }
}

#[derive(Serialize)]
struct Temperature {
    room: String,
    temperature: f64,
    humidity: f64,
}

impl Temperature {
    fn new(room: String, temperature: f64, humidity: f64) -> Self {
        Temperature {
            room,
            temperature,
            humidity,
        }
    }
}

#[get("/")]
async fn get_temp(
    client: web::Data<Client>,
    stations: web::Data<Vec<Collector>>,
) -> impl Responder {
    let mut res: Vec<Temperature> = Vec::new();
    for station in stations.iter() {
        let resp: serde_json::Value = client
            .get(&station.url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        res.push(Temperature::new(
            station.room.clone(),
            resp["temperature"].as_f64().unwrap(),
            resp["humidity"].as_f64().unwrap(),
        ))
    }
    web::Json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_temp)
            .app_data(web::Data::new(Client::new()))
            .app_data(web::Data::new(Collector::from_json(&PathBuf::from(
                "collectors.json".to_string(),
            ))))
    })
    .bind("127.0.0.1:65535")
    .unwrap()
    .run()
    .await
}
