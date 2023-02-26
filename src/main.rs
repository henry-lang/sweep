use std::io::stdout;

use board::{Board, Difficulty};
use buffer::BufCell;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    ExecutableCommand,
};

use crate::screen::Screen;

mod board;
mod buffer;
mod screen;

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    let (w, h) = terminal::size()?;
    let (w, h) = (w as usize, h as usize);

    let board = Board::generate(((w - 2) / 2, h - 2), Difficulty::Easy);
    let mut stdout = stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;

    let mut screen = Screen::new(stdout, (w as usize, h as usize))?;

    loop {
        for c in 0..w {
            screen.set_content(c, 0, '━');
            screen.set_content(c, h - 1, '━');
        }

        for r in 0..h {
            screen.set_content(0, r, '┃');
            screen.set_content(w - 1, r, '┃');
        }

        screen.set_content(0, 0, '┏');
        screen.set_content(w - 1, 0, '┓');
        screen.set_content(0, h - 1, '┗');
        screen.set_content(w - 1, h - 1, '┛');

        for (i, s) in board.squares.iter().enumerate() {
            screen.set(
                i % ((w - 2) / 2) * 2 + 1,
                i / ((w - 2) / 2) + 1,
                *s
            )
        }

        screen.flush()?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char(' ') => println!("Space bar"),
                _ => {}
            },
            _ => {}
        }
    }

    disable_raw_mode()?;
    Ok(())
}
