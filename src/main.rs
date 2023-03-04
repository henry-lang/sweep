use std::io::{Write, stdout};
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


fn board_to_screen_coords(board: (usize, usize)) -> (usize, usize) {
    (board.0 * 2 + 1, board.1 + 1)
}

struct Game<W: Write> {
    board: Board,
    screen: Screen<W>,
    status_message: String,
    selection: (usize, usize),
    term_size: (usize, usize),
    
}

impl<W: Write> Game<W> {
    pub fn new(difficulty: Difficulty, out: W) -> crossterm::Result<Self> {
        let (w, h) = terminal::size()?;
        let term_size @ (w, h) = (w as usize, h as usize);
        let board_size @ (board_w, board_h) = ((w - 2) / 2, h - 2);

        Ok(Self {
            board: Board::generate(board_size, difficulty),
            screen: Screen::new(out, term_size)?,
            status_message: String::new(),
            selection: (board_w / 2, board_h / 2),
            term_size
        })
    }

    pub fn render(&mut self) {
        let (w, h) = self.term_size;

        for c in 0..w {
            self.screen.set_content(c, 0, '━');
            self.screen.set_content(c, h - 1, '━');
        }

        for r in 0..h {
            self.screen.set_content(0, r, '┃');
            self.screen.set_content(w - 1, r, '┃');
        }

        self.screen.set_content(0, 0, '┏');
        self.screen.set_content(w - 1, 0, '┓');
        self.screen.set_content(0, h - 1, '┗');
        self.screen.set_content(w - 1, h - 1, '┛');

        self.screen.write_text((1, 0), &self.status_message);

        for (i, square) in self.board.squares.iter().enumerate() {
            self.screen.set(i % self.board.width() * 2 + 1, i / self.board.width() + 1, *square)
        }
    }

    pub fn main_loop(&mut self) -> crossterm::Result<()> {
        loop {
            self.render();
            self.screen.flush(board_to_screen_coords(self.selection))?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('k') | KeyCode::Up => {
                        if self.selection.1 == 0 {
                            self.selection.1 = self.board.height() - 1;
                        } else {
                            self.selection.1 -= 1;
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if self.selection.1 == self.board.height() - 1 {
                            self.selection.1 = 0;
                        } else {
                            self.selection.1 += 1;
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if self.selection.0 == 0 {
                            self.selection.0 = self.board.width() - 1
                        } else {
                            self.selection.0 -= 1;
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if self.selection.0 == self.board.width() - 1 {
                            self.selection.0 = 0;
                        } else {
                            self.selection.0 += 1;
                        }
                    }
                    KeyCode::Char('f') => if self.board.flag_square(self.selection) {
                        
                    }
                    KeyCode::Char(' ') => if self.board.uncover_square(self.selection) {
                        /*self.render();
                        self.screen.flush((selection.0 * 2 + 1, selection.1 + 1))?;
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
                        */
                    }
                    _ => {}
                },
                // Resizing currently just ends it, as the board can't resize during a game
                Event::Resize(_, _) => break,
                _ => {}
            }
        }

        Ok(())
    }
}

fn main() -> crossterm::Result<()> {
    let difficulty = args().nth(1).map(|d| d.parse().unwrap_or_else(|_| {
        println!("Invalid difficulty: {d}");
        std::process::exit(1);
    })).unwrap_or(Difficulty::Medium);

    let mut stdout = stdout();

    enable_raw_mode()?;

    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::SetCursorStyle::SteadyBlock)?;
    stdout.execute(terminal::Clear(ClearType::All))?;


    Game::new(difficulty, stdout)?.main_loop()?;


    disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    std::io::stdout().execute(terminal::Clear(ClearType::All))?;

    Ok(())
}
