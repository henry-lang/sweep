use std::io::{self, Write};

use crossterm::{
    cursor,
    style::{self, Attribute, Attributes, Color},
    terminal::{self, enable_raw_mode},
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

    pub fn resize(&mut self, size: (usize, usize)) {
        self.buffers[0].resize(size);
        self.buffers[1].resize(size);
        self.size = size;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.buffers[1 - self.current].reset();
    }

    pub fn fill(&mut self, with: &BufCell) {
        for cell in self.buffers[self.current].cells.iter_mut() {
            *cell = *with;
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let diff = self.buffers[self.current].diff(&self.buffers[1 - self.current]);
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut attrs = Attributes::default();
        let mut last_pos: Option<(usize, usize)> = None;

        self.buf
            .queue(terminal::Clear(terminal::ClearType::Purge))?;

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
            .queue(style::SetAttribute(Attribute::Reset))?;

        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
        self.buf.flush()?;

        Ok(())
    }
}
