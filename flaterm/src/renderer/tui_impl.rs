use crate::{Node, PropValue};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

#[derive(Debug)]
enum SizeClaim {
    Fixed(u16),
    Fill,
}

impl Default for SizeClaim {
    fn default() -> Self {
        SizeClaim::Fill
    }
}

pub fn render(node: &Node, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
    let consumed = match node.name.as_str() {
        "Block" => {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(title) = prop_value::<String>(&node, "title") {
                block = block.title(title);
            }
            f.render_widget(block, area);
            area
        }
        _ => area,
    };

    if !node.children.is_empty() {
        let rest = shrink(&consumed);

        let mut planner = LayoutPlanner::new();
        let plan = planner.analyze(node, rest);

        plan.into_iter().for_each(|(child, sub)| {
            render(&child, f, sub);
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

fn prop_value<T>(node: &Node, prop_key: &str) -> Option<T>
where
    T: From<PropValue>,
{
    node.props
        .iter()
        .find_map(|(key, prop)| match (prop, key.as_str() == prop_key) {
            (PropValue::LiteralString(_str), true) => Some(prop.clone().into()),
            (PropValue::LiteralNumber(_num), true) => Some(prop.clone().into()),
            _ => None,
        })
}

struct LayoutPlanner {
    claims: Vec<(Node, SizeClaim)>,
}

impl LayoutPlanner {
    pub fn new() -> Self {
        Self { claims: vec![] }
    }

    pub fn analyze(&mut self, node: &Node, area: Rect) -> Vec<(Node, Rect)> {
        node.children.iter().for_each(|child| {
            self.claims.push((child.clone(), claim(child)));
        });
        self.plan(&area)
    }

    fn plan(&self, area: &Rect) -> Vec<(Node, Rect)> {
        let mut ret = Vec::new();

        let mut num_of_fill = self.claims.len() as u16;
        let total = self
            .claims
            .iter()
            .filter_map(|(_, claim)| {
                if let SizeClaim::Fixed(n) = claim {
                    num_of_fill -= 1;
                    Some(n)
                } else {
                    None
                }
            })
            .fold(0, |acc, n| acc + n);

        assert!(
            total <= area.height,
            "total {} <= area.height {}",
            total,
            area.height
        );

        let remained_height = area.height - total;
        // let rem = remained_height % num_of_fill;
        let base = remained_height / num_of_fill;

        let mut rest = area.clone();
        self.claims.iter().for_each(|(node, claim)| {
            let rect = match claim {
                SizeClaim::Fixed(n) => {
                    let will_consume = Rect {
                        x: rest.x,
                        y: rest.y,
                        height: n.clone(),
                        width: rest.width,
                    };
                    rest = consume(&rest, &will_consume);
                    will_consume
                }
                SizeClaim::Fill => {
                    let will_consume = Rect {
                        x: rest.x,
                        y: rest.y,
                        height: base,
                        width: rest.width,
                    };
                    rest = consume(&rest, &will_consume);
                    will_consume
                }
            };
            ret.push((node.clone(), rect));
        });

        ret
    }
}

fn claim(node: &Node) -> SizeClaim {
    if let Some(height) = prop_value(node, "height") {
        SizeClaim::Fixed(height)
    } else {
        SizeClaim::Fill
    }
}
