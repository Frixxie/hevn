#[macro_use]
extern crate log;
extern crate simplelog;
extern crate util;

use util::{Collector, EnvData, SmartAppliance};

use simplelog::*;

use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use serde::Deserialize;
use std::fs::File;
use std::io::Error;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct MyCollector {
    url: IpAddr,
    room: String,
}

impl MyCollector {
    fn from_json(json: &Path) -> Vec<Self> {
        let file = std::fs::read_to_string(json).unwrap();
        serde_json::from_str(&file).unwrap()
    }
}

#[get("/")]
async fn collect(
    req: HttpRequest,
    collectors: web::Data<Vec<Collector>>,
) -> Result<impl Responder, Error> {
    let resp: Vec<EnvData> = collectors
        .iter()
        .map(|collector| collector.get_status())
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect();
    let con_info = req.connection_info();
    for data in &resp {
        info!("{},{}", con_info.host(), data);
    }
    Ok(web::Json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("aggregator.log").unwrap(),
        ),
    ])
    .unwrap();

    HttpServer::new(move || {
        let collectors: Vec<Collector> =
            MyCollector::from_json(&PathBuf::from("../collectors.json".to_string()))
                .iter()
                .map(|my_collector| {
                    Collector::new(my_collector.url.to_string(), my_collector.room.to_string())
                })
                .collect();

        App::new()
            .service(collect)
            .app_data(web::Data::new(collectors))
    })
    .bind("0.0.0.0:65535")
    .unwrap()
    .run()
    .await
}
