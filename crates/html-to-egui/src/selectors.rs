use super::*;

/// Slectors available to use with the Div element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivSelectors {
    /// No selector.
    None,
    /// Flex row selector. Makes the div element children horizontal (side by side).
    FlexRow,
}

impl DivSelectors {
    /// Returns the string representation of the selector.
    fn as_str(&self) -> &'static str {
        match self {
            DivSelectors::None => "",
            DivSelectors::FlexRow => "flex-row",
        }
    }
}

impl Deref for DivSelectors {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Display for DivSelectors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<DivSelectors> for String {
    fn from(val: DivSelectors) -> Self {
        val.as_str().to_string()
    }
}

impl From<DivSelectors> for std::borrow::Cow<'static, str> {
    fn from(val: DivSelectors) -> Self {
        val.as_str().into()
    }
}

impl AsRef<str> for DivSelectors {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
