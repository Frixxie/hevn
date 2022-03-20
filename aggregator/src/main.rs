#[macro_use]
mod appliance;
extern crate log;
extern crate simplelog;
extern crate util;

use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use appliance::{Heater, MyCollector};
use log::{error, info};
use simplelog::*;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;
use structopt::StructOpt;
use util::{Collector, EnvData, ShellyStatus, SmartAppliance};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Aggregator",
    about = "Connection point for the SmartAppliances"
)]
struct Opt {
    #[structopt(short = "l", long = "log-file", default_value = "aggregator.log")]
    log_file: String,

    #[structopt(short = "c", long = "collectors", default_value = "collectors.json")]
    collectors: String,
}

#[get("/")]
async fn collect(
    req: HttpRequest,
    collectors: web::Data<Vec<Collector>>,
) -> Result<impl Responder, Error> {
    let mut resp: Vec<EnvData> = Vec::new();
    for collector in collectors.iter() {
        let data = collector.get_status().await;
        match data {
            Ok(d) => resp.push(d),
            Err(e) => error!("{}", e),
        }
    }
    let con_info = req.connection_info();
    for data in &resp {
        info!("{},{}", con_info.host(), data);
    }
    Ok(web::Json(resp))
}

#[get("/heater/{id}")]
async fn heater_status(id: web::Path<String>, heaters: web::Data<Vec<Heater>>) -> impl Responder {
    for h in heaters.iter() {
        if h.get_room() == id.to_string() {
            return web::Json(h.get_status().await.unwrap());
        }
    }
    return web::Json(ShellyStatus::default());
}

#[get("/heater/{id}/on")]
async fn heater_on(id: web::Path<String>, heaters: web::Data<Vec<Heater>>) -> impl Responder {
    for h in heaters.iter() {
        if h.get_room() == id.to_string() {
            let r = h.turn_on().await.map_err(|e| error!("{}", e));
            info!("{:?}", r);
            return web::Json(h.get_status().await.unwrap());
        }
    }
    return web::Json(ShellyStatus::default());
}

#[get("/heater/{id}/off")]
async fn heater_off(id: web::Path<String>, heaters: web::Data<Vec<Heater>>) -> impl Responder {
    for h in heaters.iter() {
        if h.get_room() == id.to_string() {
            let r = h.turn_off().await.map_err(|e| error!("{}", e));
            info!("{:?}", r);
            return web::Json(h.get_status().await.unwrap());
        }
    }
    return web::Json(ShellyStatus::default());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Off,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(opt.log_file).unwrap(),
        ),
    ])
    .unwrap();

    HttpServer::new(move || {
        let collectors: Vec<Collector> =
            MyCollector::from_json(&PathBuf::from(opt.collectors.clone()))
                .iter()
                .map(|my_collector| {
                    Collector::new(
                        my_collector.get_room().to_string(),
                        my_collector.get_url().to_string(),
                    )
                })
                .collect();

        let mut my_heater: Vec<Heater> = Vec::new();
        my_heater.push(Heater::new(
            "192.168.0.101".to_string(),
            "bedroom".to_string(),
        ));

        App::new()
            .service(collect)
            .service(heater_status)
            .service(heater_on)
            .service(heater_off)
            .app_data(web::Data::new(collectors))
            .app_data(web::Data::new(my_heater))
    })
    .bind("0.0.0.0:65535")
    .unwrap()
    .run()
    .await
}
