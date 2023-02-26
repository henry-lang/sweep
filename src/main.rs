use std::io::stdout;

use buffer::BufCell;
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use tinyrand::{Rand, StdRand};

use crate::screen::Screen;

mod buffer;
mod screen;

#[derive(Copy, Clone)]
enum Square {
    Empty(u8),
    Bomb,
}

enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn percentage_bombs(self) -> f32 {
        match self {
            Difficulty::Easy => 0.10,
            Difficulty::Medium => 0.15,
            Difficulty::Hard => 0.20,
        }
    }
}

struct Board {
    size: (usize, usize),
    squares: Vec<Square>,
}

impl Board {
    pub fn generate(size: (usize, usize), difficulty: Difficulty) -> Self {
        let (w, h) = size;
        let num_squares = w * h;
        let num_bombs = (difficulty.percentage_bombs() * num_squares as f32) as usize;

        let mut rng = StdRand::default();
        let mut squares = vec![Square::Empty(0); num_squares];

        for _ in 0..num_bombs {
            let col = rng.next_lim_usize(w);
            let row = rng.next_lim_usize(h);
            let idx = col + row * w;

            squares[idx] = Square::Bomb;
            
            for offset in [1, w, w - 1, w + 1] {
                if let Some(i) = idx.checked_sub(offset) {
                    if let Square::Empty(ref mut adj) = squares[i] {
                        *adj += 1;
                    }
                }
                if idx + offset < num_squares {
                    if let Square::Empty(ref mut adj) = squares[idx + offset] {
                        *adj += 1;
                    }
                }
            }
        }

        Self { size, squares }
    }
}

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    let (w, h) = terminal::size()?;
    let (w, h) = (w as usize, h as usize);

    let board = Board::generate((w - 2, h - 2), Difficulty::Easy);

    let mut screen = Screen::new(stdout(), (w as usize, h as usize))?;

    disable_raw_mode()?;
    Ok(())
}
