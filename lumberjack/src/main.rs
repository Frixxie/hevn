extern crate util;
use reqwest::Client;
use util::EnvData;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let res: EnvData = client
        .get("https://fasteraune.com/hevn")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("{}", res);
}
