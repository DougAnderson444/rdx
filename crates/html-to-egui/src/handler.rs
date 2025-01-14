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
        // assert kebab case
        s = to_kebab_case(&s);
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

/// Type to represent a handler function_name(maybe, some, args) in the html attribute,
fn to_kebab_case(s: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            kebab_case.push('-');
            prev_is_upper = false;
        } else if c.is_uppercase() {
            if i != 0 && !prev_is_upper {
                kebab_case.push('-');
            }
            kebab_case.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            kebab_case.push(c);
            prev_is_upper = false;
        }
    }

    kebab_case
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler() {
        let func = Handler::builder()
            .named("increment".to_owned())
            .args(vec!["key".to_owned(), "value".to_owned()])
            .build();

        assert_eq!(func.to_string(), "increment(key, value)");
    }

    #[test]
    fn test_to_kebab_case() {
        let example1 = "HelloWorldExample";
        let example2 = "hello_world_example";
        let example3 = "helloWorld_example";

        let expected = "hello-world-example";

        assert_eq!(to_kebab_case(example1), expected);
        assert_eq!(to_kebab_case(example2), expected);
        assert_eq!(to_kebab_case(example3), expected);
    }
}
