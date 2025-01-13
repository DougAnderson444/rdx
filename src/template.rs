use std::fmt::Display;

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

// asemble the parts in sequence to return the string template literal
impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match part {
                TemplatePart::Static(s) => write!(f, "{}", s)?,
                TemplatePart::Dynamic(key) => write!(f, "{{{{{}}}}}", key)?,
            }
        }
        Ok(())
    }
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
    pub(crate) fn render(
        &self,
        values: impl IntoIterator<Item = (String, String)> + Clone,
    ) -> String {
        let mut result = String::new();

        for part in &self.parts {
            let vals = values.clone();
            match part {
                TemplatePart::Static(s) => result.push_str(s),
                TemplatePart::Dynamic(key) => {
                    if let Some(value) =
                        vals.into_iter()
                            .find_map(|(k, v)| if k == *key { Some(v) } else { None })
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
    use std::collections::HashMap;

    use super::*;

    // test content without any {{brackets}} returns its original self
    #[test]
    fn test_no_replacements() {
        let template = Template::new("This is a template");
        let values: HashMap<String, String> = HashMap::<String, String>::new();

        let result = template.render(values.iter().map(|(k, v)| (k.to_string(), v.to_string())));

        assert_eq!(result, "This is a template");
    }

    #[test]
    fn test_template() {
        let template = Template::new("This {{word_var}} is replaced with {{a_value}}");
        let mut values = HashMap::new();
        values.insert("word_var".to_string(), "template".to_string());
        values.insert("a_value".to_string(), "content".to_string());

        let result = template.render(values.iter().map(|(k, v)| (k.to_string(), v.clone())));
        assert_eq!(result, "This template is replaced with content");
    }

    #[test]
    fn test_three_replacements() {
        let template =
            Template::new("This {{word_var}} is replaced with {{a_value}} or these {{words_here}}");

        // print the template
        eprintln!("{:?}", template);

        let mut values = HashMap::new();
        values.insert("word_var".to_string(), "template".to_string());
        values.insert("a_value".to_string(), "content".to_string());
        values.insert("words_here".to_string(), "other words".to_string());

        let result = template.render(values.iter().map(|(k, v)| (k.to_string(), v.clone())));
        assert_eq!(
            result,
            "This template is replaced with content or these other words"
        );
    }
    // handle the case where a placeholder in the template doesn't have a corresponding value in the provided HashMap.
    #[test]
    fn test_missing_value() {
        let template = Template::new("This {{word_var}} is replaced with {{a_value}}");
        let values: HashMap<String, String> = HashMap::<String, String>::new();

        let result = template.render(values.iter().map(|(k, v)| (k.to_string(), v.to_string())));

        assert_eq!(result, "This {{word_var}} is replaced with {{a_value}}");
    }

    // test Display
    #[test]
    fn test_display() {
        let template = Template::new("This {{word_var}} is replaced with {{a_value}}");
        assert_eq!(
            format!("{}", template),
            "This {{word_var}} is replaced with {{a_value}}"
        );
    }
}
