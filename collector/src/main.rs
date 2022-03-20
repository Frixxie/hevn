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
    pin: web::Data<Mutex<Pin>>,
    collector: web::Data<Collector>,
) -> Result<impl Responder, actix_web::Error> {
    let pin = pin.lock().await;
    let (temp, humi) = read_dht11(pin.get_pin())?;
    println!("{}, {}", temp, humi);
    Ok(web::Json(EnvData::new(collector.room(), temp, humi)))
}

struct Pin {
    pin: u8,
}

impl Pin {
    fn new(pin: u8) -> Self {
        Self { pin }
    }
    fn get_pin(&self) -> u8 {
        self.pin
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
            .app_data(web::Data::new(Mutex::new(Pin::new(opt.gpio_pin))))
            .app_data(web::Data::new(collector))
    })
    .bind(format!("{}:{}", host, opt.port))?
    .run()
    .await
}
