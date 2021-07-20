use crossbeam::channel::{self, Receiver, Sender};
use futures::prelude::*;
use std::{error::Error, process::Stdio, result::Result, task::Poll, thread};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{process::Command, runtime::Runtime};

pub struct LogCollector {
    service_name: String,
    count: usize,
    marker: usize,
    channel: (Sender<usize>, Receiver<usize>),
}

impl LogCollector {
    pub fn new(service_name: String) -> Self {
        LogCollector {
            channel: channel::unbounded(),
            service_name,
            count: 0,
            marker: 0,
        }
    }

    pub fn start(&mut self) {
        let name = self.service_name.clone();
        let sender = self.channel.0.clone();
        thread::spawn(move || {
            println!("collector: thread spawn");
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                println!("collector: enter block_on");
                let _ = LogCollector::logs(name, sender).await;
            });
        });
    }

    async fn logs(service_name: String, sender: Sender<usize>) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("docker")
            .args(&["logs", "-f", service_name.as_str()])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("failed to get child output");
        let mut reader = BufReader::new(stdout).lines();

        let mut count = 0;
        while let Some(line) = reader.next_line().await? {
            println!("collector: {}", line);
            // inner.logs.push(line);
            count += 1;
            let _ = sender.send(count);
        }

        Ok(())
    }
}

impl Stream for LogCollector {
    type Item = (usize, usize);

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        println!("collector: poll_next: enter");
        let diff = self.count - self.marker;
        if diff == 0 {
            let receiver = self.channel.1.clone();
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
            self.marker = self.count;

            println!("collector: poll_next: ready");
            Poll::Ready(Some((self.count, diff)))
        }
    }
}
