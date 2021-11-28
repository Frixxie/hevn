#[macro_use]
extern crate log;
extern crate simplelog;
extern crate util;

use util::{Collector, EnvData};

use simplelog::*;

use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use futures::future::try_join_all;
use reqwest::Client;
use serde_json;
use std::fs::File;
use std::path::PathBuf;

async fn get_temperature(client: web::Data<Client>, collector: Collector) -> EnvData {
    let resp: serde_json::Value = client
        .get(&collector.url())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    EnvData::new(
        collector.room(),
        resp["temperature"].as_f64().unwrap(),
        resp["humidity"].as_f64().unwrap(),
    )
}

#[get("/")]
async fn collect(
    req: HttpRequest,
    client: web::Data<Client>,
    collectors: web::Data<Vec<Collector>>,
) -> impl Responder {
    let mut responses = Vec::new();
    // This is to enable the tokio::threads to take ownership of a copy of client and collector
    for collector in collectors.to_vec().iter() {
        let tmp_collector = collector.clone();
        let tmp_client = client.clone();
        responses.push(tokio::spawn(async move {
            get_temperature(tmp_client, tmp_collector).await
        }));
    }
    let res = try_join_all(responses).await.unwrap();
    let con_info = req.connection_info();
    for r in &res {
        info!("{},{}", con_info.host(), r)
    }
    web::Json(res)
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
    HttpServer::new(|| {
        App::new()
            .service(collect)
            .app_data(web::Data::new(Client::new()))
            .app_data(web::Data::new(Collector::from_json(&PathBuf::from(
                "../collectors.json".to_string(),
            ))))
    })
    .bind("0.0.0.0:65535")
    .unwrap()
    .run()
    .await
}
