use crate::{Direction, Node, Rect};

#[derive(Debug)]
pub enum SizeClaim {
    Fixed(u16),
    Fill,
}

impl Default for SizeClaim {
    fn default() -> Self {
        SizeClaim::Fill
    }
}

pub struct LayoutPlanner {
    direction: Direction,
    claims: Vec<(Node, SizeClaim)>,
}

impl LayoutPlanner {
    pub fn new() -> Self {
        Self {
            direction: Default::default(),
            claims: vec![],
        }
    }

    pub fn analyze(&mut self, node: &Node, area: Rect) -> Vec<(Node, Rect)> {
        if let Some(dir) = node.prop::<Direction>("direction") {
            self.direction = dir;
        }

        node.children.iter().for_each(|child| {
            self.claims.push((child.clone(), claim(child)));
        });

        self.plan(&area)
    }

    fn norm(&self, area: &Rect) -> u16 {
        match self.direction {
            Direction::Column => area.w,
            Direction::Row => area.h,
        }
    }

    // TODO: Need assertions of constraint
    fn consume(&self, area: &Rect, sub: &Rect) -> Rect {
        match self.direction {
            Direction::Column => Rect {
                x: area.x + sub.w,
                y: area.y,
                w: area.w - sub.w,
                h: area.h,
            },
            Direction::Row => Rect {
                x: area.x,
                y: area.y + sub.h,
                w: area.w,
                h: area.h - sub.h,
            },
        }
    }

    fn plan(&self, area: &Rect) -> Vec<(Node, Rect)> {
        let mut ret = Vec::new();

        let mut num_of_fills = self.claims.len() as u16;
        let total = self
            .claims
            .iter()
            .filter_map(|(_, claim)| {
                if let SizeClaim::Fixed(n) = claim {
                    num_of_fills -= 1;
                    Some(n)
                } else {
                    None
                }
            })
            .fold(0, |acc, n| acc + n);

        if self.direction == Direction::Row {
            assert!(total <= area.h, "total {} <= area.h {}", total, area.h);
        } else {
            assert!(total <= area.w, "total {} <= area.w {}", total, area.w);
        }

        let remained_norm = self.norm(area) - total;
        // TODO: let rem = remained_norm % num_of_fill;
        let base = remained_norm / num_of_fills;

        let mut rest = area.clone();
        self.claims.iter().for_each(|(node, claim)| {
            let rect = match self.direction {
                Direction::Column => match claim {
                    SizeClaim::Fixed(n) => {
                        let will_consume = Rect {
                            x: rest.x,
                            y: rest.y,
                            h: rest.h,
                            w: n.clone(),
                        };
                        rest = self.consume(&rest, &will_consume);
                        will_consume
                    }
                    SizeClaim::Fill => {
                        let will_consume = Rect {
                            x: rest.x,
                            y: rest.y,
                            h: rest.h,
                            w: base,
                        };
                        rest = self.consume(&rest, &will_consume);
                        will_consume
                    }
                },
                Direction::Row => match claim {
                    SizeClaim::Fixed(n) => {
                        let will_consume = Rect {
                            x: rest.x,
                            y: rest.y,
                            h: n.clone(),
                            w: rest.w,
                        };
                        rest = self.consume(&rest, &will_consume);
                        will_consume
                    }
                    SizeClaim::Fill => {
                        let will_consume = Rect {
                            x: rest.x,
                            y: rest.y,
                            h: base,
                            w: rest.w,
                        };
                        rest = self.consume(&rest, &will_consume);
                        will_consume
                    }
                },
            };

            ret.push((node.clone(), rect));
        });

        ret
    }
}

fn claim(node: &Node) -> SizeClaim {
    if let Some(height) = node.prop("height") {
        SizeClaim::Fixed(height)
    } else if let Some(width) = node.prop("width") {
        SizeClaim::Fixed(width)
    } else {
        SizeClaim::Fill
    }
}
