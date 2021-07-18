use crossbeam::thread;
use futures::prelude::*;
use std::{error::Error, process::Stdio, result::Result};
use tokio::process::Command;
use tokio::runtime::Runtime;
use tokio_util::codec::{FramedRead, LinesCodec};

pub struct LogCollector {
    service_name: String,
    logs: Vec<String>,
}

impl LogCollector {
    pub fn new(service_name: String) -> Self {
        LogCollector {
            service_name,
            logs: Default::default(),
        }
    }

    pub fn start(&self) {
        thread::scope(|s| {
            let _ = s
                .spawn(move |_| {
                    println!("log_collector: thread spawn");
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        println!("log_collector: enter block_on");
                        let _ = self.logs().await; // Ignore errors
                    });
                })
                .join();
        })
        .unwrap();
    }

    async fn logs(&self) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("docker")
            .args(&["logs", "-f", self.service_name.as_str()])
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let stdout = child.stdout.take().expect("failed to get child output");
        let mut reader = FramedRead::new(stdout, LinesCodec::new());
        while let Some(line) = reader.next().await {
            println!("{}", line?);
        }
        Ok(())
    }
}
