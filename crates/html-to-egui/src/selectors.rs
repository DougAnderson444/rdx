//! CSS tailwind style class selectors for the elements which translate into egui layout and styling.
use super::*;

/// Selectors available to use with the Div element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Selectors {
    /// Flex row selector. Makes the div element children horizontal (side by side).
    FlexRow,
    /// Absolute positioning selector.
    Absolute,
    /// Monospace font selector.
    Monospace,
    /// Text color black.
    TextBlack,
    /// Grid
    Grid,
    /// Number of Grid columns
    GridCols,
    GridCols1,
    GridCols2,
    GridCols3,
    GridCols4,
}

impl Selectors {
    /// Constant for the flex row selector.
    const FLEX_ROW: &'static str = "flex-row";
    /// For absolute positioning
    const ABSOLUTE: &'static str = "absolute";
    /// Monospace
    const MONOSPACE: &'static str = "monospace";
    /// Text color black
    const TEXT_BLACK: &'static str = "text-black";
    /// Grid  
    const GRID: &'static str = "grid";
    /// The number of columns in the grid
    const GRID_COLS: &'static str = "grid-cols-";
    const GRID_COLS_1: &'static str = "grid-cols-1";
    const GRID_COLS_2: &'static str = "grid-cols-2";
    const GRID_COLS_3: &'static str = "grid-cols-3";
    const GRID_COLS_4: &'static str = "grid-cols-4";

    /// Returns the string representation of the selector.
    fn as_str(&self) -> &'static str {
        match self {
            Selectors::FlexRow => Self::FLEX_ROW,
            Selectors::Absolute => Self::ABSOLUTE,
            Selectors::Monospace => Self::MONOSPACE,
            Selectors::TextBlack => Self::TEXT_BLACK,
            Selectors::Grid => Self::GRID,
            Selectors::GridCols => Self::GRID_COLS,
            Selectors::GridCols1 => Self::GRID_COLS_1,
            Selectors::GridCols2 => Self::GRID_COLS_2,
            Selectors::GridCols3 => Self::GRID_COLS_3,
            Selectors::GridCols4 => Self::GRID_COLS_4,
        }
    }
}

impl TryFrom<&str> for Selectors {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            Selectors::FLEX_ROW => Ok(Selectors::FlexRow),
            Selectors::ABSOLUTE => Ok(Selectors::Absolute),
            Selectors::MONOSPACE => Ok(Selectors::Monospace),
            Selectors::TEXT_BLACK => Ok(Selectors::TextBlack),
            Selectors::GRID => Ok(Selectors::Grid),
            Selectors::GRID_COLS_1 => Ok(Selectors::GridCols1),
            Selectors::GRID_COLS_2 => Ok(Selectors::GridCols2),
            Selectors::GRID_COLS_3 => Ok(Selectors::GridCols3),
            Selectors::GRID_COLS_4 => Ok(Selectors::GridCols4),
            _ => Err(Error::InvalidConversion(s.to_string())),
        }
    }
}

impl TryFrom<String> for Selectors {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Selectors::try_from(s.as_str())
    }
}

impl TryFrom<&String> for Selectors {
    type Error = Error;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Selectors::try_from(s.as_str())
    }
}

impl Deref for Selectors {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for Selectors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Selectors> for String {
    fn from(val: Selectors) -> Self {
        val.as_str().to_string()
    }
}

impl From<Selectors> for std::borrow::Cow<'static, str> {
    fn from(val: Selectors) -> Self {
        val.as_str().into()
    }
}

impl AsRef<str> for Selectors {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
