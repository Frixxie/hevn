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
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Aggregator",
    about = "Connection point for the SmartAppliances"
)]
struct Opt {
    #[structopt(short = "l", long = "log-file", default_value = "~/.aggregator.log")]
    log_file: String,

    #[structopt(short = "c", long = "collectors", default_value = "collectors.json")]
    collectors: String
}

#[derive(Deserialize)]
struct MyCollector {
    url: String,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();


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
            File::create(opt.log_file).unwrap(),
        ),
    ])
    .unwrap();

    HttpServer::new(move || {
        let collectors: Vec<Collector> =
            MyCollector::from_json(&PathBuf::from(opt.collectors.clone()))
                .iter()
                .map(|my_collector| {
                    Collector::new(my_collector.room.to_string(), my_collector.url.to_string())
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
