use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Write};

fn main() -> Result<()> {
    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".magenta()))?;
            }
        }
    }
    stdout.flush()?;

    screen! {
        <Block direction=column>
            <Block width=20>
                <List>{ options }</List>
            </Block>
            <Block direction=row>
                <Block height=1>
                    { "Service:" }
                    { service_name }
                </Block>
                <Block>
                    { logs }
                </Block>
            </Block>
        </Block>
    }

    Ok(())
}
