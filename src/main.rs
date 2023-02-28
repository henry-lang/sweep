use std::io::stdout;
use std::env::args;

use board::{Board, Difficulty};
use crossterm::{
    style::Color,
    cursor,
    event::{self, Event, KeyCode},
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    ExecutableCommand,
};

use crate::buffer::BufCell;
use crate::screen::Screen;

mod board;
mod buffer;
mod screen;

fn render(screen: &mut Screen<impl std::io::Write>, board: &Board, w: usize, h: usize) {
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

    for (i, square) in board.squares.iter().enumerate() {
        screen.set(i % board.width() * 2 + 1, i / board.width() + 1, *square)
    }
}

fn main() -> crossterm::Result<()> {
    let difficulty = args().nth(1).map(|d| d.parse().unwrap_or_else(|_| {
        println!("Invalid difficulty: {d}");
        std::process::exit(1);
    })).unwrap_or(Difficulty::Medium);

    enable_raw_mode()?;
    let (w, h) = terminal::size()?;
    let term_size @ (w, h) = (w as usize, h as usize);
    let board_size @ (board_w, board_h) = ((w - 2) / 2, h - 2);

    let mut board = Board::generate(board_size, difficulty);
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::SetCursorStyle::SteadyBlock)?;
    stdout.execute(terminal::Clear(ClearType::All))?;

    let mut screen = Screen::new(stdout, term_size)?;
    let mut selection = (board_w / 2, board_h / 2);

    loop {
        render(&mut screen, &board, w, h);
        screen.flush((selection.0 * 2 + 1, selection.1 + 1))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('k') | KeyCode::Up => {
                    if selection.1 == 0 {
                        selection.1 = board_h - 1;
                    } else {
                        selection.1 -= 1;
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if selection.1 == board_h - 1 {
                        selection.1 = 0;
                    } else {
                        selection.1 += 1;
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if selection.0 == 0 {
                        selection.0 = board_w - 1
                    } else {
                        selection.0 -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if selection.0 == board_w - 1 {
                        selection.0 = 0;
                    } else {
                        selection.0 += 1;
                    }
                }
                KeyCode::Char('f') => if board.flag_square(selection) {
                    
                }
                KeyCode::Char(' ') => if board.uncover_square(selection) {
                    render(&mut screen, &board, w, h);
                    screen.flush((selection.0 * 2 + 1, selection.1 + 1))?;
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    for off in 0..10 {
                        render(&mut screen, &board, w, h);
                        for c in (selection.0 * 2 + 1).saturating_sub(off * 2)..=((selection.0 * 2 + 1) + off * 2).min(w - 1) {
                            for r in selection.1.saturating_sub(off)..=(selection.1 + off).min(h - 1) {
                                screen.set(c, r, BufCell {content: ' ', bg: Color::Red, ..Default::default()})
                            }
                        }
                        screen.flush((selection.0 * 2 + 1, selection.1 + 1))?;
                        std::thread::sleep(std::time::Duration::from_millis(30));
                    }
                    break;
                }
                _ => {}
            },
            // Resizing currently just ends it, as the board can't resize during a game
            Event::Resize(_, _) => break,
            _ => {}
        }
    }

    disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    std::io::stdout().execute(terminal::Clear(ClearType::All))?;

    Ok(())
}
