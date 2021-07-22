use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// use futures::{stream, StreamExt};
use decom_core::{docker_compose, LogCollector};
use std::{
    error::Error,
    io,
    result::Result,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    text::Spans,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let services = docker_compose::services().await?;
    // println!("main: services: {:?}", services);

    let mut collectors = vec![];
    services.iter().for_each(|s| {
        let mut collector = LogCollector::new(&s.service_name, &s.container_name);
        collector.start();
        collectors.push(collector);
        // println!("main: collector: '{}' started", s.service_name);
    });
    // let collectors = stream::select_all(collectors);

    /*
    thread::spawn(|| {
        while let Some((service_name, total, diff)) = collectors.next().await {
            println!("main: {}: {} [+{}]", service_name, total, diff);
        }
    });
    */

    let max = (services.len() - 1) as i32;
    let mut current = 0;

    loop {
        let _ = terminal.draw(|f| {
            let service = services.get(current as usize).unwrap();
            let size = f.size();
            let block = Block::default()
                .title(service.service_name.clone())
                .borders(Borders::ALL);

            let collector = collectors.get(current as usize).unwrap();
            let text: Vec<Spans> = collector
                .slice()
                .into_iter()
                .map(|line| Spans::from(line))
                .collect();
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, size);
        });

        match rx.recv() {
            Ok(Event::Input(event)) => match event.code {
                KeyCode::Char('h') => {
                    current = clamp(current, -1, max);
                }
                KeyCode::Char('l') => {
                    current = clamp(current, 1, max);
                }
                KeyCode::Char('q') => {
                    disable_raw_mode();
                    execute!(terminal.backend_mut(), LeaveAlternateScreen);
                    terminal.show_cursor();
                    break;
                }
                _ => {}
            },
            _ => (),
        }
    }

    Ok(())
}

fn clamp(n: i32, d: i32, max: i32) -> i32 {
    let mut n = n + d;
    if n < 0 {
        n = 0;
    } else if n > max {
        n = max;
    }
    n
}
