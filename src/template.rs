use rhai::Dynamic;

/// Holds the template parts (Static and Dynamic).
///
/// Renders the template with the provided values.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub(crate) parts: Vec<TemplatePart>,
}

/// Represents a part of the template.
///
/// Static parts are just strings.
/// Dynamic parts are placeholders that will be replaced with values.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Static(String),
    Dynamic(String),
}

impl Template {
    /// Create a new Template from the provided string.
    pub(crate) fn new(template: &str) -> Self {
        let mut parts = Vec::new();
        let mut current = String::new();

        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                if !current.is_empty() {
                    parts.push(TemplatePart::Static(current));
                    current = String::new();
                }
                while let Some(ch) = chars.next() {
                    if ch == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second '}'
                        break;
                    }
                    current.push(ch);
                }
                parts.push(TemplatePart::Dynamic(current.trim().to_string()));
                current = String::new();
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            parts.push(TemplatePart::Static(current));
        }

        Template { parts }
    }

    /// Render the template with the provided values.
    ///
    /// Pass it an object that implements Iterator so it can perform lookups on the values given
    /// the key.
    pub(crate) fn render<'a>(
        &self,
        mut values: impl Iterator<Item = (&'a str, bool, &'a Dynamic)>,
    ) -> String {
        let mut result = String::new();
        for part in &self.parts {
            match part {
                TemplatePart::Static(s) => result.push_str(s),
                TemplatePart::Dynamic(key) => {
                    if let Some(value) =
                        values.find_map(
                            |(k, _, v)| {
                                if k == key {
                                    Some(v.to_string())
                                } else {
                                    None
                                }
                            },
                        )
                    {
                        result.push_str(&value);
                    } else {
                        result.push_str(&format!("{{{{{}}}}}", key));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr as _};

    use super::*;

    #[test]
    fn test_template() {
        let template = Template::new("This {{word_var}} is replaced with {{a_value}}");
        let mut values = HashMap::new();
        values.insert(
            "word_var".to_string(),
            Dynamic::from_str("template").unwrap(),
        );
        values.insert("a_value".to_string(), Dynamic::from_str("content").unwrap());

        let result = template.render(values.iter().map(|(k, v)| (k.as_str(), false, v)));
        assert_eq!(result, "This template is replaced with content");
    }

    // handle the case where a placeholder in the template doesn't have a corresponding value in the provided HashMap.
    #[test]
    fn test_missing_value() {
        let template = Template::new("This {{word_var}} is replaced with {{a_value}}");
        let values: HashMap<_, Dynamic> = HashMap::<String, rhai::Dynamic>::new();

        let result = template.render(values.iter().map(|(k, v)| (k.as_str(), false, v)));
        assert_eq!(result, "This {{word_var}} is replaced with {{a_value}}");
    }
}
