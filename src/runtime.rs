use crate::colors::{get_rgb, Colors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClsColor {
    Black,
    Green,
    Yellow,
    Blue,
    Red,
    Buff,
    Cyan,
    Magenta,
    Orange,
}

#[derive(Debug, Default)]
pub struct Runtime {
    pub lines: Vec<String>,
    pub clear_requested: bool,
    pub bg_color: ClsColor, // last used background color
}

impl Runtime {
    /// CLS with optional color argument.
    /// - None: clear using existing bg_color
    /// - Some(c): set bg_color to c, then clear
    pub fn cls(&mut self, color: Option<ClsColor>) {
        if let Some(c) = color {
            self.bg_color = c;
        }
        self.lines.clear();
        self.clear_requested = true;
    }

    pub fn print(&mut self, s: String) {
        self.lines.push(s);
    }

    pub fn current_bg_rgba(&self) -> [u8; 4] {
        self.bg_color.to_rgba()
    }
}

impl Default for ClsColor {
    fn default() -> Self {
        ClsColor::Green
    }
}

impl ClsColor {
    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Black),
            1 => Some(Self::Green),
            2 => Some(Self::Yellow),
            3 => Some(Self::Blue),
            4 => Some(Self::Red),
            5 => Some(Self::Buff),
            6 => Some(Self::Cyan),
            7 => Some(Self::Magenta),
            8 => Some(Self::Orange),
            _ => None,
        }
    }

    pub fn to_rgba(self) -> [u8; 4] {
        match self {
            Self::Black => get_rgb(Colors::Black),
            Self::Green => get_rgb(Colors::BrightGreen),
            Self::Yellow => get_rgb(Colors::BrightYellow),
            Self::Blue => get_rgb(Colors::BrightBlue),
            Self::Red => get_rgb(Colors::BrightRed),
            Self::Buff => get_rgb(Colors::Apricot),     // closest palette match
            Self::Cyan => get_rgb(Colors::BrightCyan),
            Self::Magenta => get_rgb(Colors::BrightMagenta),
            Self::Orange => get_rgb(Colors::BrightOrange),
        }
    }
}