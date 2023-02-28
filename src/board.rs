use core::str::FromStr;
use std::{time::SystemTime, collections::{VecDeque, HashSet}};

use crossterm::style::Color;
use tinyrand::{Rand, Seeded, StdRand};

use crate::buffer::BufCell;

pub enum Difficulty {
    Debug,
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn percentage_bombs(self) -> f32 {
        match self {
            Difficulty::Debug => 0.01,
            Difficulty::Easy => 0.10,
            Difficulty::Medium => 0.15,
            Difficulty::Hard => 0.20,
        }
    }
}

impl FromStr for Difficulty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "debug" => Self::Debug,
            "easy" => Self::Easy,
            "medium" => Self::Medium,
            "hard" => Self::Hard,
            _ => Err(())?
        })
    }
}

#[derive(Copy, Clone)]
pub enum Content {
    Empty(u8),
    Bomb,
}

#[derive(Copy, Clone)]
enum Visibility {
    Flagged,
    Uncovered,
    Covered
}

#[derive(Copy, Clone)]
pub struct Square {
    content: Content,
    visibility: Visibility
}

impl Square {
    pub fn new(content: Content) -> Self {
        Self {
            content,
            visibility: Visibility::Covered
        }
    }
}

impl From<Square> for BufCell {
    fn from(square: Square) -> Self {
        match square {
            Square { visibility: Visibility::Flagged, .. } => BufCell {
                content: 'F',
                fg: Color::Red,
                ..Default::default()
            },
            Square { visibility: Visibility::Covered, .. } => BufCell {
                content: '.',
                ..Default::default()
            },
            Square {
                visibility: Visibility::Uncovered,
                content: Content::Empty(0),
                ..
            } => BufCell {
                content: '.',
                fg: Color::DarkGrey,
                ..Default::default()
            },
            Square {
                visibility: Visibility::Uncovered,
                content: Content::Empty(adj),
                ..
            } => BufCell {
                content: (adj + b'0') as char,
                fg: match adj {
                    0 => unreachable!(),
                    1 => Color::Blue,
                    2 => Color::Green,
                    3 => Color::Red,
                    _ => Color::Yellow,
                },
                ..Default::default()
            },
            Square {
                visibility: Visibility::Uncovered,
                content: Content::Bomb,
                ..
            } => BufCell {
                content: '*',
                fg: Color::Black,
                bg: Color::Red,
                ..Default::default()
            },
        }
    }
}

pub struct Board {
    size: (usize, usize),
    num_bombs: usize,
    flagged_squares: usize, // How many squares in general have been flagged
    flagged_bombs: usize,   // How many bombs in specific have been flagged
    pub squares: Vec<Square>,
}

impl Board {
    pub fn square(&self, pos: (usize, usize)) -> Square {
        let (col, row) = pos;
        self.squares[col + row * self.size.0]
    }

    pub fn square_mut(&mut self, pos: (usize, usize)) -> &mut Square {
        let (col, row) = pos;
        &mut self.squares[col + row * self.size.0]
    }

    pub fn generate(size: (usize, usize), difficulty: Difficulty) -> Self {
        let (w, h) = size;
        let num_squares = w * h;
        let num_bombs = (difficulty.percentage_bombs() * num_squares as f32) as usize;

        let mut rng = StdRand::seed(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("get system time for random")
                .as_secs(),
        );
        let mut squares = vec![Square::new(Content::Empty(0)); num_squares];

        for _ in 0..num_bombs {
            let idx = loop {
                let col = rng.next_lim_usize(w);
                let row = rng.next_lim_usize(h);
                let idx = col + row * w;
                if let Content::Bomb = squares[idx].content {
                    continue;
                }
                break idx;
            };

            squares[idx] = Square::new(Content::Bomb);

            for offset in [1, w, w - 1, w + 1] {
                if let Some(i) = idx.checked_sub(offset) {
                    if let Content::Empty(ref mut adj) = squares[i].content {
                        *adj += 1;
                    }
                }
                if idx + offset < num_squares {
                    if let Content::Empty(ref mut adj) = squares[idx + offset].content {
                        *adj += 1;
                    }
                }
            }
        }

        Self {
            size,
            num_bombs,
            flagged_bombs: 0,
            flagged_squares: 0,
            squares,
        }
    }

    // For now, we won't return an Option<GameEnding> or whatever
    pub fn flag_square(&mut self, pos: (usize, usize)) {
        let square = self.square(pos);

        match square.visibility {
            Visibility::Uncovered => return,
            Visibility::Flagged => {
                if let Content::Bomb = square.content {
                    self.flagged_bombs -= 1;
                }
                self.square_mut(pos).visibility = Visibility::Covered;
                self.flagged_squares -= 1;
            }
            Visibility::Covered if self.flagged_squares < self.num_bombs => {
                if let Content::Bomb = square.content {
                    self.flagged_bombs += 1;
                }

                self.square_mut(pos).visibility = Visibility::Flagged;
                self.flagged_squares += 1;
            }
            _ => {}
        }
    }

    pub fn uncover_square(&mut self, pos: (usize, usize)) {
        if let Content::Empty(0) = self.square(pos).content {
            let mut queue = VecDeque::from([pos]);
            let mut visited = HashSet::from([pos]);

            while let Some(next) = queue.pop_front() {
                self.square_mut(next).visibility = Visibility::Uncovered;
                if !matches!(self.square(next).content, Content::Empty(0)) {
                    continue;
                }

                // println!("{:?}", visited);

                if next.0 > 0 && !visited.contains(&(next.0 - 1, next.1)) {
                    queue.push_back((next.0 - 1, next.1));
                    visited.insert((next.0 - 1, next.1));
                }
                if next.0 < self.size.0 - 1 && !visited.contains(&(next.0 + 1, next.1)) {
                    queue.push_back((next.0 + 1, next.1));
                    visited.insert((next.0 + 1, next.1));
                }
                if next.1 > 0 && !visited.contains(&(next.0, next.1 - 1)) {
                    queue.push_back((next.0, next.1 - 1));
                    visited.insert((next.0, next.1 - 1));
                }

                if next.1 < self.size.1 - 1 && !visited.contains(&(next.0, next.1 + 1)) {
                    queue.push_back((next.0, next.1 + 1));
                    visited.insert((next.0, next.1 + 1));
                }
            }
        } else {
            self.square_mut(pos).visibility = Visibility::Uncovered;
        }
    }
}
