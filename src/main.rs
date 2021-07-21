use futures::{stream, StreamExt};
use std::{error::Error, result::Result};

mod docker;
mod docker_compose;
mod log_collector;

use log_collector::LogCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let services = docker_compose::services().await?;
    println!("main: services: {:?}", services);

    let mut collectors = vec![];
    services.iter().for_each(|s| {
        let mut collector = LogCollector::new(&s.service_name, &s.container_name);
        collector.start();
        collectors.push(collector);
        println!("main: collector: '{}' started", s.service_name);
    });

    let mut collectors = stream::select_all(collectors);
    while let Some((service_name, total, diff)) = collectors.next().await {
        println!("main: {}: {} [+{}]", service_name, total, diff);
    }

    Ok(())
}
