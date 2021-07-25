use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use flaterm_macro::layout;
use std::{
    error::Error,
    io::{self, Stdout},
    result::Result,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame, Terminal,
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
    let tick_rate = Duration::from_millis(1000);
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
            let layout = layout! {
                <Block title="flaterm-example">
                    <Block title="tabs" height=3 />
                    <Block title="content" />
                </Block>
            };
            let area = f.size();
            render(&layout, f, area);
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

// enum Hint {
//     Consumed(Rect),
//     Claimed(Rect),
//     Unknown,
// }

fn render(node: &flaterm::Node, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) -> Rect {
    let consumed = match node.name.as_str() {
        "Block" => {
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, area);
            area
        }
        "Text" => {
            let block = Block::default().title("Text").borders(Borders::ALL);
            f.render_widget(block, area);
            area
        }
        _ => area,
    };

    if !node.children.is_empty() {
        let mut rest = shrink(&consumed);
        node.children.iter().for_each(|child| {
            let consumed = render(child, f, rest);
            rest = consume(&rest, &consumed);
        });
    }

    consumed
}

fn shrink(rect: &Rect) -> Rect {
    Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width - 2,
        height: rect.height - 2,
    }
}

// TODO: Need 'direction' and 'order' params
fn consume(area: &Rect, sub: &Rect) -> Rect {
    // TODO: Need assertions of constraint
    Rect {
        x: area.x,
        y: area.y + sub.height,
        width: area.width,
        height: area.height - sub.height,
    }
}
