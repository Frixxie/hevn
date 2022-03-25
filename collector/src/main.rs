extern crate util;
use actix_web::{get, web, App, HttpServer, Responder, Result};
use reader::read_dht11;
use structopt::StructOpt;
use tokio::sync::Mutex;
use util::{Collector, EnvData};
mod reader;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Collector",
    about = "Collector for DHT11 sensor using Raspberry Pi"
)]
struct Opt {
    #[structopt(short = "r", long = "room")]
    room: String,

    #[structopt(short = "h", long = "host", default_value = "0.0.0.0")]
    host: String,

    #[structopt(short = "p", long = "port", default_value = "5000")]
    port: String,

    #[structopt(short = "g", long = "gpio", default_value = "14")]
    gpio_pin: u8,
}

#[get("/data")]
async fn read_from_sensor(
    pin: web::Data<Pin>,
    collector: web::Data<Collector>,
) -> Result<impl Responder, actix_web::Error> {
    let my_pin = pin.pin.lock().await;
    match read_dht11(*my_pin) {
        Ok((temp, humi)) => Ok(web::Json(EnvData::new(collector.room(), temp, humi))),
        Err(e) => {
            println!("{}", e);
            Ok(web::Json(EnvData::new(collector.room(), 0, 0)))
        }
    }
}

struct Pin {
    pin: Mutex<u8>,
}

impl Pin {
    fn new(pin: u8) -> Self {
        Self {
            pin: Mutex::new(pin),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let host = opt.host.clone();

    HttpServer::new(move || {
        let collector = Collector::new(opt.room.clone(), opt.host.clone());
        App::new()
            .service(read_from_sensor)
            .app_data(web::Data::new(Pin::new(opt.gpio_pin)))
            .app_data(web::Data::new(collector))
    })
    .bind(format!("{}:{}", host, opt.port))?
    .run()
    .await
}
