use crate::{LayoutPlanner, Node, Rect as FlatermRect};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

pub fn render(node: &Node, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
    let consumed = match node.name.as_str() {
        "Block" => {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(title) = node.prop::<String>("title") {
                block = block.title(title);
            }
            f.render_widget(block, area);
            area
        }
        _ => area,
    };

    if !node.children.is_empty() {
        let rest = shrink(&consumed);

        let mut planner = LayoutPlanner::default();
        let plan = planner.analyze(node, rest.into());

        plan.into_iter().for_each(|(child, sub)| {
            render(&child, f, sub.into());
        });
    }
}

fn shrink(rect: &Rect) -> Rect {
    Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width - 2,
        height: rect.height - 2,
    }
}

impl From<FlatermRect> for Rect {
    fn from(rect: FlatermRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
        }
    }
}

impl From<Rect> for FlatermRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            w: rect.width,
            h: rect.height,
        }
    }
}
