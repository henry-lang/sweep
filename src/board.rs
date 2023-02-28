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

        let mut board = Self {
            size,
            num_bombs,
            flagged_squares: 0,
            flagged_bombs: 0,
            squares: vec![Square::new(Content::Empty(0)); num_squares]
        };

        let mut rng = StdRand::seed(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("get system time for random")
                .as_secs(),
        );

        for _ in 0..num_bombs {
            let pos = loop {
                let pos = (rng.next_lim_usize(w), rng.next_lim_usize(h));
                if let Content::Bomb = board.square(pos).content {
                    continue;
                }
                break pos;
            };

            board.square_mut(pos).content = Content::Bomb;

            for c in pos.0.saturating_sub(1)..=(pos.0 + 1).min(board.size.0 - 1) {
                for r in pos.1.saturating_sub(1)..=(pos.1 + 1).min(board.size.1 - 1) {
                    if let Content::Empty(ref mut adj) = board.square_mut((c, r)).content {
                        *adj += 1;
                    }
                }
            }
        }
        
        board
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
            // Possibly make this better somehow - just doing a dfs thing rn
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
