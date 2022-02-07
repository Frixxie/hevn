extern crate util;
use reqwest::Client;
use std::time::SystemTime;
use structopt::StructOpt;
use util::EnvData;

#[derive(Debug, StructOpt)]
#[structopt(name = "Lumberjack", about = "Tool to interact with the Hevn project")]
struct Opt {
    #[structopt(
        short = "u",
        long = "url",
        default_value = "https://fasteraune.com/hevn"
    )]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let opt = Opt::from_args();

    let res: Vec<EnvData> = client.get(opt.url.as_str()).send().await?.json().await?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    for r in res {
        println!("{},{}", now, r);
    }
    Ok(())
}
