use futures::StreamExt;
use std::{error::Error, result::Result};

mod docker;
mod docker_compose;
mod log_collector;

use log_collector::LogCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let services = docker_compose::services().await?;
    println!("main: services: {:?}", services);

    /*
    let mut collector = LogCollector::new("periodic-output".into());
    collector.start();
    println!("main: started");

    while let Some((total, diff)) = collector.next().await {
        println!("main: {} [+{}]", total, diff);
    }
    */

    Ok(())
}
