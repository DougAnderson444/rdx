use super::*;

/// Enumerate the action handlers, such as on-click, on-change, etc.
///
/// The action handlers are attributed as data-on-click, data-on-change, etc,
/// since data attributes must begin with `data-`. then the actual
/// function name comes after.
///
/// This enum enumerates the 'on-click', 'on-change', etc. so that
/// the html crate can use this enum to build the html text programmatically
/// in a type safe way, without typo errors.
pub enum Action {
    OnClick,
    OnChange,
}

impl Action {
    // Define constants for the attribute strings
    const ON_CLICK: &'static str = "on-click";
    const ON_CHANGE: &'static str = "on-change";

    // Method to get the string representation
    fn as_str(&self) -> &'static str {
        match self {
            Action::OnClick => Self::ON_CLICK,
            Action::OnChange => Self::ON_CHANGE,
        }
    }
}

impl From<Action> for Attribute {
    fn from(val: Action) -> Self {
        match val {
            Action::OnClick => Attribute::DataOnClick,
            Action::OnChange => Attribute::DataOnChange,
        }
    }
}

impl From<Action> for &'static str {
    fn from(val: Action) -> Self {
        val.as_str()
    }
}

impl From<Action> for String {
    fn from(val: Action) -> Self {
        val.as_str().to_string()
    }
}

// impl into std::borrow::Cow<'static, str>>
impl From<Action> for std::borrow::Cow<'static, str> {
    fn from(val: Action) -> Self {
        val.as_str().into()
    }
}

impl Deref for Action {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
