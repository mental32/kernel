/// Various VGA colors that may be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    /// Black
    Black = 0,

    /// Blue
    Blue = 1,

    /// Green
    Green = 2,

    /// Cyan
    Cyan = 3,

    /// Red
    Red = 4,

    /// Magenta
    Magenta = 5,

    /// Browm
    Brown = 6,

    /// Light gray
    LightGray = 7,

    /// Dark gray
    DarkGray = 8,

    /// Light blue
    LightBlue = 9,

    /// Light green
    LightGreen = 10,

    /// Light cyan
    LightCyan = 11,

    /// Light red
    LightRed = 12,

    /// Pink
    Pink = 13,

    /// Yellow
    Yellow = 14,

    /// White
    White = 15,
}

const COLORS: &[Color; 8] = &[
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

impl Color {
    /// Get a color from an index.
    pub fn from_usize(other: usize) -> Option<Self> {
        if other >= COLORS.len() {
            None
        } else {
            Some(COLORS[other])
        }
    }
}
