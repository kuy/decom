use crate::Node;
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

// enum Hint {
//     Consumed(Rect),
//     Claimed(Rect),
//     Unknown,
// }

pub fn render(node: &Node, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) -> Rect {
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
