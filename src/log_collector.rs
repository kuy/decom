use crossbeam::channel::{self, Receiver, Sender};
use futures::prelude::*;
use std::sync::{Arc, Mutex};
use std::{error::Error, process::Stdio, result::Result, task::Poll, thread};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{process::Command, runtime::Runtime};

pub struct LogCollector {
    container_name: String,
    marker: usize,
    notifier: (Sender<usize>, Receiver<usize>),
    transfer: (Sender<String>, Receiver<String>),
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogCollector {
    pub fn new(container_name: &str) -> Self {
        LogCollector {
            container_name: container_name.to_string(),
            marker: 0,
            notifier: channel::unbounded(),
            transfer: channel::unbounded(),
            logs: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn start(&mut self) {
        // Main: command runner
        let name = self.container_name.clone();
        let transfer = self.transfer.0.clone();
        thread::spawn(move || {
            println!("collector: logs: spawn");
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                println!("collector: logs: block_on");
                LogCollector::logs(name, transfer).await;
            });
        });

        // Sub: storing logs
        let logs = self.logs.clone();
        let notifier = self.notifier.0.clone();
        let transfer = self.transfer.1.clone();
        thread::spawn(move || {
            println!("collector: collector: spawn");
            while let Ok(line) = transfer.recv() {
                let mut logs = logs.lock().expect("failed to lock");
                println!("collector: collector: recv");
                logs.push(line);
                notifier.send(logs.len());
            }
        });
    }

    async fn logs(container_name: String, transfer: Sender<String>) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("docker")
            .args(&["logs", "-f", container_name.as_str()])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("failed to get child output");
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await? {
            println!("collector: {}", line);
            transfer.send(line);
        }

        Ok(())
    }

    fn len(&self) -> usize {
        let logs = self.logs.lock().expect("failed to lock");
        logs.len()
    }
}

impl Stream for LogCollector {
    type Item = (usize, usize);

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        println!("collector: poll_next: enter");
        let count = self.len();
        let diff = count - self.marker;
        if diff == 0 {
            let receiver = self.notifier.1.clone();
            let waker = cx.waker().clone();
            thread::spawn(move || {
                if let Ok(count) = receiver.recv() {
                    println!("collector: poll_next: recv lines={}", count);
                } else {
                    println!("collector: poll_next: failed");
                }
                println!("collector: poll_next: wake");
                waker.wake();
            });

            println!("collector: poll_next: pending");
            Poll::Pending
        } else {
            self.marker = count;

            println!("collector: poll_next: ready");
            Poll::Ready(Some((count, diff)))
        }
    }
}
