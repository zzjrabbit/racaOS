use super::color::{Color, NamedColor};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Flags: u16 {
        const INVERSE = 0b0000_0000_0001;
        const HIDDEN = 0b0001_0000_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub content: char,
    pub foreground: Color,
    pub background: Color,
    pub flags: Flags,
}

impl Cell {
    pub fn reset(&self) -> Self {
        Self {
            background: self.background,
            ..Default::default()
        }
    }

    pub fn with_content(&self, content: char) -> Self {
        Self { content, ..*self }
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            content: ' ',
            foreground: Color::Indexed(NamedColor::BrightWhite as u8),
            background: Color::Indexed(NamedColor::Black as u8),
            flags: Flags::empty(),
        }
    }
}
