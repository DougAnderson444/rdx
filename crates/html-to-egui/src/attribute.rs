use super::*;

/// These are the data attibutes asscoiated with the action handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    DataOnClick,
    DataOnChange,
}

impl Attribute {
    // Define constants for the attribute strings
    const DATA_ON_CLICK: &'static str = "data-on-click";
    const DATA_ON_CHANGE: &'static str = "data-on-change";

    // Method to get the string representation
    fn as_str(&self) -> &'static str {
        match self {
            Attribute::DataOnClick => Self::DATA_ON_CLICK,
            Attribute::DataOnChange => Self::DATA_ON_CHANGE,
        }
    }
}

impl From<Attribute> for Action {
    fn from(attr: Attribute) -> Self {
        match attr {
            Attribute::DataOnClick => Action::OnClick,
            Attribute::DataOnChange => Action::OnChange,
        }
    }
}

impl From<&Attribute> for Action {
    fn from(attr: &Attribute) -> Self {
        match attr {
            Attribute::DataOnClick => Action::OnClick,
            Attribute::DataOnChange => Action::OnChange,
        }
    }
}

impl TryFrom<&str> for Attribute {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            Attribute::DATA_ON_CLICK => Ok(Attribute::DataOnClick),
            Attribute::DATA_ON_CHANGE => Ok(Attribute::DataOnChange),
            _ => Err(Error::InvalidConversion(s.to_string())),
        }
    }
}

impl From<Attribute> for &'static str {
    fn from(val: Attribute) -> Self {
        val.as_str()
    }
}

impl From<Attribute> for String {
    fn from(val: Attribute) -> Self {
        val.as_str().to_string()
    }
}

impl From<Attribute> for std::borrow::Cow<'static, str> {
    fn from(val: Attribute) -> Self {
        val.as_str().into()
    }
}

impl Deref for Attribute {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
