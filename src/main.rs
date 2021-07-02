use std::{error::Error, result::Result};

mod docker_compose;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ids = docker_compose::containers().await?;
    println!("ids: {:?}", ids);
    Ok(())
}
