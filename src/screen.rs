use std::io::{self, Write};

use crossterm::{
    cursor,
    style::{self, Attribute, Attributes, Color},
    QueueableCommand,
};

use crate::buffer::{BufCell, Buffer};

pub struct Screen<W: Write> {
    buf: W,
    buffers: [Buffer; 2],
    current: usize, // Current buffer used for rendering
    size: (usize, usize),
}

impl<W: Write> Screen<W> {
    pub fn new(buf: W, size: (usize, usize)) -> crossterm::Result<Self> {
        Ok(Self {
            buf,
            buffers: [Buffer::empty(size), Buffer::empty(size)],
            current: 0,
            size,
        })
    }

    pub fn reset(&mut self) {
        self.buffers[1 - self.current].reset();
    }

    pub fn fill(&mut self, with: BufCell) {
        for cell in self.buffers[self.current].cells.iter_mut() {
            *cell = with;
        }
    }

    pub fn set_content(&mut self, col: usize, row: usize, to: char) {
        self.buffers[self.current].cells[col + row * self.size.0].content = to;
    }

    pub fn set(&mut self, col: usize, row: usize, to: impl Into<BufCell>) {
        self.buffers[self.current].cells[col + row * self.size.0] = to.into();
    }

    pub fn flush(&mut self, cursor: (usize, usize)) -> io::Result<()> {
        let diff = self.buffers[self.current].diff(&self.buffers[1 - self.current]);
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut attrs = Attributes::default();
        let mut last_pos: Option<(usize, usize)> = None;

        for (col, row, cell) in diff {
            if !matches!(last_pos, Some(p) if col == p.0 + 1 && row == p.1) {
                self.buf.queue(cursor::MoveTo(col as u16, row as u16))?;
            }

            if cell.fg != fg {
                fg = cell.fg;
                self.buf.queue(style::SetForegroundColor(fg))?;
            }

            if cell.bg != bg {
                bg = cell.bg;
                self.buf.queue(style::SetBackgroundColor(bg))?;
            }

            if cell.attrs != attrs {
                attrs = cell.attrs;
                self.buf.queue(style::SetAttributes(attrs))?;
            }

            last_pos = Some((col, row));

            self.buf.queue(style::Print(cell.content))?;
        }

        self.buf
            .queue(style::SetForegroundColor(Color::Reset))?
            .queue(style::SetBackgroundColor(Color::Reset))?
            .queue(style::SetAttribute(Attribute::Reset))?
            .queue(cursor::MoveTo(cursor.0 as u16, cursor.1 as u16))?;

        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
        self.buf.flush()?;

        Ok(())
    }
}
