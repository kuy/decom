use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use flaterm_macro::layout;
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
    let tick_rate = Duration::from_millis(500);
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

    loop {
        let _ = terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("flaterm-example")
                .borders(Borders::ALL);

            let text: Vec<Spans> = vec![
                "The quick brown fox jumps over the lazy dog".into(),
                "The quick brown fox jumps over the lazy dog".into(),
                "The quick brown fox jumps over the lazy dog".into(),
            ];
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, size);
        });

        terminal.draw(|f| {
            let layout = layout! {
                <Block>
                </Block>
            };
            eprintln!("{:?}", layout);
        });

        match rx.recv() {
            Ok(Event::Input(event)) => match event.code {
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
