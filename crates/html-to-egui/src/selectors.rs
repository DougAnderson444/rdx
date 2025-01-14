use super::*;

/// Slectors available to use with the Div element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selectors {
    /// No selector.
    None,
    /// Flex row selector. Makes the div element children horizontal (side by side).
    FlexRow,
}

impl Selectors {
    /// Default is empty string.
    const DEFAULT: &'static str = "";
    /// Constant for the flex row selector.
    const FLEX_ROW: &'static str = "flex-row";

    /// Returns the string representation of the selector.
    fn as_str(&self) -> &'static str {
        match self {
            Selectors::None => Self::DEFAULT,
            Selectors::FlexRow => Self::FLEX_ROW,
        }
    }
}

impl TryFrom<&str> for Selectors {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            Selectors::FLEX_ROW => Ok(Selectors::FlexRow),
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
