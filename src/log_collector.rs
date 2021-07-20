use crossbeam::channel::{self, Receiver, Sender};
use futures::prelude::*;
use std::sync::{Arc, Mutex};
use std::{error::Error, process::Stdio, result::Result, task::Poll, thread};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{process::Command, runtime::Runtime};

pub struct LogCollector {
    service_name: String,
    marker: usize,
    notifier: (Sender<usize>, Receiver<usize>),
    transfer: (Sender<String>, Receiver<String>),
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogCollector {
    pub fn new(service_name: String) -> Self {
        LogCollector {
            service_name,
            marker: 0,
            notifier: channel::unbounded(),
            transfer: channel::unbounded(),
            logs: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn start(&mut self) {
        let name = self.service_name.clone();
        let notifier = self.notifier.0.clone();
        let transfer = self.transfer.0.clone();
        thread::spawn(move || {
            println!("collector: logs: spawn");
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                println!("collector: logs: block_on");
                LogCollector::logs(name, notifier, transfer).await;
            });
        });
        let logs = self.logs.clone();
        let transfer = self.transfer.1.clone();
        thread::spawn(move || {
            println!("collector: collector: spawn");
            while let Ok(line) = transfer.recv() {
                let mut store = logs.lock().expect("failed to lock");
                println!("collector: collector: recv");
                store.push(line);
            }
        });
    }

    async fn logs(
        service_name: String,
        notifier: Sender<usize>,
        transfer: Sender<String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("docker")
            .args(&["logs", "-f", service_name.as_str()])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("failed to get child output");
        let mut reader = BufReader::new(stdout).lines();

        let mut count = 0;
        while let Some(line) = reader.next_line().await? {
            println!("collector: {}", line);
            transfer.send(line);
            count += 1;
            notifier.send(count);
        }

        Ok(())
    }

    fn len(&self) -> usize {
        let store = self.logs.lock().expect("failed to lock");
        let count = store.len();
        count
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
