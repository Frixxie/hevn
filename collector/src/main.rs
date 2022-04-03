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
    stored_data: web::Data<StoredData>,
) -> Result<impl Responder, actix_web::Error> {
    let my_pin = pin.pin.lock().await;
    match stored_data.predict().await {
        Some(data) => println!("{}", data),
        None => {}
    }
    loop {
        match read_dht11(*my_pin) {
            Ok((temp, humi)) => {
                let env_data = EnvData::new(collector.room(), temp, humi);
                stored_data.add(env_data.clone()).await;
                if stored_data.len().await > 15 {
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

fn mean<T>(values: &[T]) -> Option<T>
where
    T: std::ops::Add<Output = T>
        + std::ops::Div<Output = T>
        + Default
        + Copy
        + std::convert::Into<T>
        + std::convert::TryFrom<usize>,
{
    let len = match T::try_from(values.len()) {
        Ok(len) => len,
        Err(_) => return None,
    };
    Some(values.iter().fold(T::default(), |a, b| a + *b) / len)
}

fn linear_regression<T>(xs: &[T], ys: &[T]) -> Option<(T, T)>
where
    T: std::ops::Add<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Sub<Output = T>
        + Copy
        + Default
        + std::convert::TryFrom<usize>,
{
    let xy = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (*x * *y))
        .collect::<Vec<T>>();

    let x2 = xs.iter().map(|x| (*x * *x)).collect::<Vec<T>>();

    let xy_mean = mean(&xy).unwrap();
    let x_mean = mean(xs).unwrap();
    let y_mean = mean(ys).unwrap();
    let x2_mean = mean(&x2).unwrap();

    let slope = (xy_mean - x_mean * y_mean) / (x2_mean - x_mean * x_mean);
    let intercept = y_mean - slope * x_mean;

    Some((slope, intercept))
}

struct StoredData {
    s_data: Mutex<Vec<EnvData>>,
}

impl StoredData {
    fn new() -> Self {
        StoredData {
            s_data: Mutex::new(Vec::new()),
        }
    }

    async fn add(&self, data: EnvData) {
        let mut s_data = self.s_data.lock().await;
        s_data.push(data);
    }

    async fn remove(&self) -> Option<EnvData> {
        let mut s_data = self.s_data.lock().await;
        let len = s_data.len();
        if len > 0 {
            Some(s_data.remove(len - 1))
        } else {
            None
        }
    }

    async fn len(&self) -> usize {
        let s_data = self.s_data.lock().await;
        s_data.len()
    }

    /// predict the temperature and humidity
    /// based on the last 5 values
    /// using linear regression
    async fn predict(&self) -> Option<EnvData> {
        let s_data = self.s_data.lock().await;

        if s_data.len() < 5 {
            return None;
        }

        let x = (0..s_data.len()).map(|x| x as i32).collect::<Vec<_>>();
        let humis = s_data.iter().map(|x| x.humidity as i32).collect::<Vec<_>>();
        let temps = s_data
            .iter()
            .map(|x| x.temperature as i32)
            .collect::<Vec<_>>();

        let res_humi = linear_regression(&x, &humis).unwrap();
        let res_temp = linear_regression(&x, &temps).unwrap();

        let predicted_humi = res_humi.0 * (x.len()) as i32 + res_humi.1;
        let predicted_temp = res_temp.0 * (x.len()) as i32 + res_temp.1;

        Some(EnvData::new(
            s_data[0].room.clone(),
            predicted_temp as i16,
            predicted_humi as u16,
        ))
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

    let stored_data = web::Data::new(StoredData::new());
    let my_pin = web::Data::new(Pin::new(opt.gpio_pin));
    let my_collector = web::Data::new(Collector::new(opt.room.clone(), opt.host.clone()));

    HttpServer::new(move || {
        App::new()
            .service(read_from_sensor)
            .app_data(my_pin.clone())
            .app_data(my_collector.clone())
            .app_data(stored_data.clone())
    })
    .bind(format!("{}:{}", host, opt.port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        let xs = [0, 1, 2, 3, 4];
        let ys = [-2, 0, 2, 4, 6];

        let res = linear_regression(&xs, &ys);
        assert_eq!(res.unwrap(), (2, -2));
    }

    #[test]
    fn test_mean() {
        let xs = [1, 2, 3, 4, 5];
        let ys = [2, 4, 6, 8, 10];

        let res = mean(&xs);
        assert_eq!(res.unwrap(), 3);
        let res = mean(&ys);
        assert_eq!(res.unwrap(), 6);
    }

    #[tokio::test]
    async fn test_stored_data_add_remove() {
        let s_data = StoredData::new();
        let data = EnvData::new("test".to_string(), 10, 20);
        s_data.add(data.clone()).await;
        let res = s_data.remove().await.unwrap();
        assert_eq!(res, data);
        assert_eq!(s_data.len().await, 0);
    }

    #[tokio::test]
    async fn test_stored_data_predict_one() {
        let s_data = StoredData::new();
        let data = EnvData::new("test".to_string(), 10, 20);
        s_data.add(data.clone()).await;
        let res = s_data.predict().await;
        assert_eq!(res, None);
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_increase() {
        let s_data = StoredData::new();
        for i in 0..5 {
            let data = EnvData::new("test".to_string(), i * 2, (i as i16).try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 10, 5)));
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_decrease() {
        let s_data = StoredData::new();
        for i in (5..10).into_iter().rev() {
            let data = EnvData::new("test".to_string(), i, (i as i16).try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 4, 4)));
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_same() {
        let s_data = StoredData::new();
        for _ in 0..5 {
            let data = EnvData::new("test".to_string(), 5, 5.try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 5, 5)));
    }
}
