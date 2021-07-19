use futures::prelude::*;
use std::{
    error::Error,
    process::Stdio,
    result::Result,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
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
    waker: Option<Waker>,
}

impl LogCollector {
    pub fn new(service_name: String) -> Self {
        LogCollector {
            inner: Arc::new(Mutex::new(LogCollectorInner {
                service_name,
                logs: Default::default(),
                marker: 0,
                waker: None,
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

        if let Some(waker) = inner.waker.clone() {
            waker.wake();
        }
    }

    Ok(())
}

impl Stream for LogCollector {
    type Item = (usize, usize);

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut inner = self.inner.lock().unwrap();

        let diff = inner.logs.len() - inner.marker;
        if diff == 0 {
            // inner.waker.replace(cx.waker().clone());
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Some((inner.logs.len(), diff)))
        }
    }
}
