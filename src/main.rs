use futures::{stream, StreamExt};
use std::{error::Error, result::Result};
use tui::backend::CrosstermBackend;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Terminal;

mod docker;
mod docker_compose;
mod log_collector;

use log_collector::LogCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::thread::spawn(|| {
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        if let Ok(mut terminal) = Terminal::new(backend) {
            terminal.clear();
            terminal.draw(|f| {
                let size = f.size();
                let block = Block::default().title("Block").borders(Borders::ALL);
                let text = vec![Spans::from("This is a paragraph with several lines. You can change style your text the way you want.")];
                let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true});
                f.render_widget(paragraph, size);
            });
        }
    });

    let services = docker_compose::services().await?;
    println!("main: services: {:?}", services);

    let mut collectors = vec![];
    services.iter().for_each(|s| {
        let mut collector = LogCollector::new(&s.service_name, &s.container_name);
        collector.start();
        collectors.push(collector);
        println!("main: collector: '{}' started", s.service_name);
    });
    /*
        let mut collectors = stream::select_all(collectors);
        while let Some((service_name, total, diff)) = collectors.next().await {
            println!("main: {}: {} [+{}]", service_name, total, diff);
        }
    */
    Ok(())
}
