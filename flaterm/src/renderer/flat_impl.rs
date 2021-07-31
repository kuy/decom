use crate::Node;
use crossterm::{
    cursor, queue,
    style::{self, Stylize},
};
use std::io::stdout;

pub fn render(_node: &Node) {
    let mut stdout = stdout();
    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("█".magenta())
                );
            }
        }
    }
}
