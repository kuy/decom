use crossbeam::channel::{self, Receiver, Sender};
use futures::prelude::*;
use std::{
    error::Error,
    process::Stdio,
    result::Result,
    sync::{Arc, Mutex},
    task::Poll,
    thread,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{process::Command, runtime::Runtime};

pub struct LogCollector {
    inner: Arc<Mutex<LogCollectorInner>>,
}

struct LogCollectorInner {
    service_name: String,
    logs: Vec<String>,
    marker: usize,
    channel: (Sender<usize>, Receiver<usize>),
}

impl LogCollector {
    pub fn new(service_name: String) -> Self {
        LogCollector {
            inner: Arc::new(Mutex::new(LogCollectorInner {
                service_name,
                logs: Default::default(),
                marker: 0,
                channel: channel::unbounded(),
            })),
        }
    }

    pub fn start(&mut self) {
        let local = self.inner.clone();
        thread::spawn(move || {
            println!("collector: thread spawn");
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                println!("collector: enter block_on");
                let _ = collect_logs(local).await;
            });
        });
    }
}

async fn collect_logs(inner: Arc<Mutex<LogCollectorInner>>) -> Result<(), Box<dyn Error>> {
    let mut inner = inner.lock().unwrap();

    let mut child = Command::new("docker")
        .args(&["logs", "-f", inner.service_name.as_str()])
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("failed to get child output");
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        println!("collector: {}", line);
        inner.logs.push(line);
        let _ = inner.channel.0.send(inner.logs.len());
    }

    Ok(())
}

impl Stream for LogCollector {
    type Item = (usize, usize);

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        println!("collector: poll_next: enter");
        let mut inner = self.inner.lock().unwrap();

        println!("collector: poll_next: locked");
        let len = inner.logs.len();
        let diff = len - inner.marker;
        if diff == 0 {
            let receiver = inner.channel.1.clone();
            std::mem::drop(inner);

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
            inner.marker = len;
            std::mem::drop(inner);

            println!("collector: poll_next: ready");
            Poll::Ready(Some((len, diff)))
        }
    }
}
