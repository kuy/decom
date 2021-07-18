use std::{error::Error, result::Result};
use tokio::time::{sleep, Duration};

mod docker;
mod docker_compose;
mod log_collector;

use log_collector::LogCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let ids = docker_compose::containers().await?;
    // let mapping = docker::names(ids).await?;
    // println!("ids: {:?}", mapping);

    let collector = LogCollector::new("periodic-output".into());
    collector.start();
    println!("collector started");

    loop {
        println!("loop");
        sleep(Duration::from_secs(60)).await;
    }

    // docker::logs("periodic-output".into()).await?;
    Ok(())
}
