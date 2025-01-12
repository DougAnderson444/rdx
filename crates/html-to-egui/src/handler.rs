//! Handler module for types to represent a handler function_name(maybe, some, args) in the html attribute,
use super::*;

/// Type to represent a handler function_name(maybe, some, args) in the html attribute,
/// in response to an [Action].
/// This is used to type check the function name and the arguments in the html
///
/// # Example
/// ```rust
/// # use html_to_egui::Handler;
/// let func = Handler::builder()
///    .named("increment".to_owned())
///    .args(vec!["key".to_owned(), "value".to_owned()])
///    .build();
///
/// // Handler automatically converts into a string
/// assert_eq!(func.to_string(), "increment(key, value)");
#[derive(bon::Builder, Debug)]
pub struct Handler {
    named: String,
    args: Option<Vec<String>>,
}

impl Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.named.clone();
        s.push('(');
        if let Some(args) = &self.args {
            for (i, arg) in args.iter().enumerate() {
                s.push_str(arg);
                if i < args.len() - 1 {
                    s.push_str(", ");
                }
            }
        }
        s.push(')');
        write!(f, "{}", s)
    }
}

impl From<Handler> for std::borrow::Cow<'static, str> {
    fn from(val: Handler) -> Self {
        val.to_string().into()
    }
}
