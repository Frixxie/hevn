extern crate util;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
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

    #[structopt(short = "d", long = "database_url", default_value = "")]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let opt = Opt::from_args();

    let res: Vec<EnvData> = client.get(opt.url.as_str()).send().await?.json().await?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    for r in &res {
        println!("{},{}", now, r);
    }

    if !opt.database_url.is_empty() {
        let pool = PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(5))
            .connect(&opt.database_url)
            .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS hevn (time INT, room TEXT, temp REAL, hum REAL)")
            .execute(&pool)
            .await?;

        for r in &res {
            sqlx::query("INSERT INTO hevn (time, room, temp, hum) VALUES ($1, $2, $3, $4)")
                .bind(now as i64)
                .bind(r.room.as_str())
                .bind(r.temperature)
                .bind(r.humidity as i16)
                .execute(&pool)
                .await?;
        }
    }

    Ok(())
}
