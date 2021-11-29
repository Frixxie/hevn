extern crate util;
use actix_web::{get, web, App, HttpServer, Responder};
mod reader;
use reader::read_dht11;
use util::{Collector, EnvData};

#[get("/data")]
async fn read_from_sensor(pin: web::Data<Pin>, collector: web::Data<Collector>) -> impl Responder {
    let (temp, humi) = read_dht11(pin.get_pin()).unwrap();
    let data = EnvData::new(collector.room(), temp as f64, humi as f64);
    web::Json(data)
}

struct Pin {
    pin: u8,
}

impl Pin {
    fn new(pin: u8) -> Self {
        Self { pin }
    }
    fn get_pin(&self) -> u8 {
        return self.pin.clone();
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(read_from_sensor)
            .app_data(web::Data::new(Pin::new(8)))
            .app_data(web::Data::new(Collector::new(
                "Bedroom".to_string(),
                "0.0.0.0".to_string(),
            )))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
