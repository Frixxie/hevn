extern crate util;
use reqwest::Client;
use util::EnvData;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let res: Vec<EnvData> = client
        .get("https://fasteraune.com/hevn")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for r in res {
        println!("{}", r);
    }
}
