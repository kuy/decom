use crate::{Direction, LayoutPlanner, Node, Rect};
use crossterm::{
    cursor, queue,
    style::{self},
    terminal,
};
use std::io::{stdout, Stdout};

pub struct Context {
    area: Rect,
    direction: Direction,
    last: bool,
}

pub fn render(node: &Node, context: Option<Context>) {
    let mut stdout = stdout();

    match node.name.as_str() {
        "Block" => {
            if let Some(Context {
                area,
                direction,
                last,
            }) = &context
            {
                if !last {
                    match direction {
                        Direction::Column => {
                            draw_vertical(&mut stdout, area.x + area.w, area.y, area.y + area.h)
                        }
                        Direction::Row => {
                            draw_horizontal(&mut stdout, area.y + area.h, area.x, area.x + area.w)
                        }
                    }
                }
            }
        }
        _ => (),
    }

    if !node.children.is_empty() {
        let area = context.map(|c| c.area.clone()).unwrap_or_else(|| {
            let (w, h) = terminal::size().unwrap_or_else(|err| {
                panic!("Failed to get terminal size: {:?}", err);
            });
            Rect { x: 0, y: 0, w, h }
        });

        let mut planner = LayoutPlanner::default();
        let plan = planner.analyze(node, area);
        let len = plan.len();
        plan.into_iter().enumerate().for_each(|(i, (child, sub))| {
            render(
                &child,
                Some(Context {
                    area: sub,
                    direction: planner.direction.clone(),
                    last: i == (len - 1),
                }),
            );
        });
    }
}

fn draw_horizontal(out: &mut Stdout, y: u16, x1: u16, x2: u16) {
    for x in x1..x2 {
        queue!(out, cursor::MoveTo(x, y), style::Print("-"));
    }
}

fn draw_vertical(out: &mut Stdout, x: u16, y1: u16, y2: u16) {
    for y in y1..y2 {
        queue!(out, cursor::MoveTo(x, y), style::Print("|"));
    }
}
