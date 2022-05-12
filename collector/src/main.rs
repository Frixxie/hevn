extern crate util;
use actix_web::{get, web, App, HttpServer, Responder, Result};
use reader::read_dht11;
use structopt::StructOpt;
use tokio::sync::Mutex;
use util::{Collector, EnvData};
mod reader;
mod stored_data;

use stored_data::StoredData;

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

    /// Limit of data to store
    #[structopt(short = "l", long = "limit", default_value = "5")]
    limit: usize,
}

#[get("/predict")]
async fn predict(stored_data: web::Data<StoredData>) -> Result<impl Responder> {
    let possible_expected_data = stored_data.predict(stored_data.get_timestamp().await).await;
    let deviation = stored_data.get_expected_deviation(2.0).await;
    return Ok(format!("{:?}, {:?}", possible_expected_data, deviation));
}

#[get("/read")]
async fn read(pin: web::Data<Pin>, collector: web::Data<Collector>) -> Result<impl Responder> {
    let my_pin = pin.pin.lock().await;
    loop {
        match read_dht11(*my_pin) {
            Ok((temperature, humidity)) => {
                return Ok(web::Json(EnvData::new(
                    collector.room(),
                    temperature,
                    humidity,
                )))
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

#[get("/data")]
async fn data(
    pin: web::Data<Pin>,
    collector: web::Data<Collector>,
    stored_data: web::Data<StoredData>,
) -> Result<impl Responder, actix_web::Error> {
    let my_pin = pin.pin.lock().await;
    let possible_expected_data = stored_data.predict(stored_data.get_timestamp().await).await;
    let deviation = stored_data.get_expected_deviation(2.0).await;
    let mut tries = 0;
    loop {
        match read_dht11(*my_pin) {
            Ok((temp, humi)) => {
                // Check if the data is valid
                if let Some((devi_temp, devi_humi)) = deviation {
                    if let Some(true) = possible_expected_data.as_ref().map(|data| {
                        dbg!(
                            temp,
                            data.temperature as f32 - devi_temp,
                            data.temperature as f32 + devi_temp,
                            humi,
                            data.humidity as f32 - devi_humi,
                            data.humidity as f32 + devi_humi
                        );
                        (data.temperature as f32 - temp as f32).abs() > devi_temp
                            || (data.humidity as f32 - humi as f32).abs() > devi_humi
                            || tries > 16
                    }) {
                        tries += 1;
                        continue;
                    }
                }

                let env_data = EnvData::new(collector.room(), temp, humi);

                // Store the data
                stored_data.add(env_data.clone()).await;
                if stored_data.len().await > stored_data.get_lim() {
                    stored_data.remove().await;
                }

                return Ok(web::Json(env_data));
            }
            Err(e) => {
                println!("{}", e);
            }
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

    let stored_data = web::Data::new(StoredData::new(opt.limit));
    let my_pin = web::Data::new(Pin::new(opt.gpio_pin));
    let my_collector = web::Data::new(Collector::new(opt.room.clone(), opt.host.clone()));

    HttpServer::new(move || {
        App::new()
            .service(data)
            .service(read)
            .service(predict)
            .app_data(my_pin.clone())
            .app_data(my_collector.clone())
            .app_data(stored_data.clone())
    })
    .bind(format!("{}:{}", host, opt.port))?
    .run()
    .await
}
