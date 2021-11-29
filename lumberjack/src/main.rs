extern crate util;
use reqwest::Client;
use util::EnvData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let res: Vec<EnvData> = client
        .get("https://fasteraune.com/hevn")
        .send()
        .await?
        .json()
        .await?;

    for r in res {
        println!("{}", r);
    }
    Ok(())
}
