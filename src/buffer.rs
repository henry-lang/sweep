use crossterm::style::{Attributes, Color};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BufCell {
    pub content: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attributes,
}

impl BufCell {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

impl Default for BufCell {
    fn default() -> Self {
        Self {
            content: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            attrs: Default::default(),
        }
    }
}

pub struct Buffer {
    size: (usize, usize),    // cols, rows
    pub cells: Vec<BufCell>, // .len() should always be size.0 * size.1
}

impl Buffer {
    pub fn empty(size: (usize, usize)) -> Self {
        Self::filled_with(size, Default::default())
    }

    fn cell_index(&self, col: usize, row: usize) -> usize {
        col + row * self.size.1
    }

    pub fn filled_with(size: (usize, usize), cell: BufCell) -> Self {
        Self {
            size,
            cells: vec![cell; size.0 * size.1],
        }
    }

    pub fn reset(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    pub fn diff<'a>(
        &'a self,
        other: &'a Buffer,
    ) -> impl Iterator<Item = (usize, usize, &'a BufCell)> + 'a {
        self.cells
            .iter()
            .zip(other.cells.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, _))| (i % self.size.0, i / self.size.0, a))
    }
}
