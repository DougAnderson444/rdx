// mod cache;

use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

use crate::{template::Template, Error};

#[derive(Parser)]
#[grammar = "egui_layout.pest"]
struct EguiLayoutParser;

/// Props are a HashMap of key-value pairs
pub type Props = HashMap<String, String>;

/// Functions are a HashMap of function name and list of arguments
pub type Functions = HashMap<String, Vec<String>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Component {
    Horizontal {
        content: Option<String>,
        props: Props,
        children: Vec<Component>,
    },
    Vertical {
        content: Option<String>,
        props: HashMap<String, String>,
        children: Vec<Component>,
    },
    Button {
        content: Option<String>,
        props: HashMap<String, String>,
        /// function name and list of arguments
        functions: HashMap<String, Vec<String>>,
    },
    Label {
        content: String,
        template: Option<Template>,
        props: HashMap<String, String>,
    },
    Text {
        content: String,
        template: Option<Template>,
        props: HashMap<String, String>,
    },
    TextEdit {
        content: String,
        props: HashMap<String, String>,
        functions: HashMap<String, Vec<String>>,
        template: Option<Template>,
    },
    // List an iterator
}

impl FromTagName for Component {
    fn from_tag_name(
        tag_name: &str,
        content: Option<String>,
        props: HashMap<String, String>,
        children: Vec<Component>,
        functions: Option<HashMap<String, Vec<String>>>,
        template: Option<Template>,
    ) -> Option<Self> {
        match tag_name {
            "Horizontal" => Some(Component::Horizontal {
                content,
                props,
                children,
            }),
            "Vertical" => Some(Component::Vertical {
                content,
                props,
                children,
            }),
            "Button" => Some(Component::Button {
                content,
                props,
                functions: functions.unwrap_or_default(),
            }),
            "Label" => Some(Component::Label {
                content: content.unwrap(),
                props,
                template,
            }),
            "TextEdit" => Some(Component::TextEdit {
                content: content.unwrap(),
                props,
                functions: functions.unwrap_or_default(),
                template,
            }),
            "Text" => Some(Component::Text {
                content: content.unwrap(),
                template,
                props,
            }),
            _ => None,
        }
    }
}

/// Helper trait to map a tag name to a Component enum variant
trait FromTagName {
    fn from_tag_name(
        tag_name: &str,
        content: Option<String>,
        props: Props,
        children: Vec<Component>,
        functions: Option<Functions>,
        template: Option<Template>,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// Helper macro to create a Component from a tag name
macro_rules! component_from_tag {
    ($tag_name:expr, $content:expr, $props:expr, $children:expr, $functions:expr, $template:expr, $span:expr) => {
        Component::from_tag_name(
            $tag_name, $content, $props, $children, $functions, $template,
        )
        .ok_or_else(|| {
            Error::Parse(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Unknown tag name: {}", $tag_name),
                },
                $span,
            )))
        })
    };
}

fn parse_element(pair: pest::iterators::Pair<'_, Rule>) -> Result<Component, Error> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::element => {
            let mut inner = pair.into_inner();

            let mut props = HashMap::default();
            let mut functions = HashMap::default();

            let tag_name = inner
                .clone()
                .filter_map(|p| match p.as_rule() {
                    Rule::open_tag => p
                        .into_inner() // <== tag_inner
                        .filter_map(|p| match p.as_rule() {
                            Rule::identifier => Some(p.as_str()),
                            _ => None,
                        })
                        .next(),
                    _ => None,
                })
                .next()
                .unwrap();

            // let open_tag = inner.next().unwrap();
            // let tag_inner = open_tag.into_inner();
            // let tag_name = tag_inner.next().unwrap().as_str();

            for p in inner
                .next() // <== open_tag
                .unwrap()
                .into_inner() // <== tag_inner
                .filter(|p| p.as_rule() == Rule::attribute)
            {
                let mut attr_inner = p.into_inner();
                let name = attr_inner.next().unwrap().as_str().to_string();
                let value = attr_inner.next().unwrap();
                match value.as_rule() {
                    Rule::string | Rule::inner_string => {
                        props.insert(name, value.as_str().to_string());
                    }
                    // on_click=increment(1)
                    // props on_click needs to be saved with the function name Increment
                    // function increment, needs to be saved with the argument 1
                    Rule::functions => {
                        let mut func_inner = value.into_inner();
                        let func_name = func_inner.next().unwrap().as_str().to_string();
                        let args = func_inner
                            .filter_map(|p| match p.as_rule() {
                                Rule::identifier => Some(p.as_str().to_string()),
                                _ => None,
                            })
                            .collect();
                        props.insert(name, func_name.clone());
                        functions.insert(func_name, args);
                    }
                    _ => {
                        return Err(Error::Parse(Box::new(pest::error::Error::new_from_span(
                            pest::error::ErrorVariant::CustomError {
                                message: format!(
                                    "Expected string or function call, got {:?}",
                                    value
                                ),
                            },
                            value.as_span(),
                        ))))
                    }
                }
            }

            // content is Rule::text, if any
            let content = inner
                .clone()
                .filter_map(|p| match p.as_rule() {
                    Rule::text => Some(p.as_str().to_string()),
                    _ => None,
                })
                .next();

            let children: Vec<Component> = inner
                .filter_map(|p| match p.as_rule() {
                    Rule::element => Some(parse_element(p)),
                    Rule::text => {
                        let text = p.as_str();
                        if text.trim().is_empty() {
                            None
                        } else {
                            Some(Ok(Component::Text {
                                content: text.to_string(),
                                template: None,
                                props: HashMap::default(),
                            }))
                        }
                    }
                    _ => None,
                })
                .collect::<Result<_, _>>()?;

            // make a new Template from the content. If there is a TeplatePart::Dynamic, then
            // we have a template. Otherwise, it's just a string so set template to None.
            let template = content
                .as_ref()
                .map(|c| Template::new(c))
                .filter(|t| {
                    t.parts
                        .iter()
                        .any(|p| matches!(p, crate::template::TemplatePart::Dynamic(_)))
                })
                .map(Some)
                .unwrap_or(None);

            let res = component_from_tag!(
                tag_name,
                content,
                props,
                children,
                Some(functions),
                template,
                span
            )?;

            Ok(res)
        }
        _ => Err(Error::Parse(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Expected element, got {:?}", pair.as_rule()),
            },
            span,
        )))),
    }
}

pub(crate) fn parse(input: &str) -> Result<Vec<Component>, Error> {
    let mut ast = vec![];

    let pairs =
        EguiLayoutParser::parse(Rule::document, input).map_err(|e| Error::Parse(Box::new(e)))?;

    for pair in pairs {
        if pair.as_rule() == Rule::element {
            ast.push(parse_element(pair)?);
        }
    }

    Ok(ast)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    // fn init_logger() {
    //     let subscriber = tracing_subscriber::fmt()
    //         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //         .finish();
    //     if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    //         tracing::warn!("failed to set subscriber: {}", e);
    //     }
    // }

    #[test]
    fn test_parse_and_generate() {
        let input = r#"<Horizontal>
        </Horizontal>"#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Horizontal {
                content: None,
                props: HashMap::default(),
                children: vec![]
            }]
        );
    }

    #[test]
    fn complex_test() {
        let input = r#"
            <Horizontal>
                <Button>Click me!</Button>
                <Vertical>
                    <Label>Hello, world!</Label>
                </Vertical>
            </Horizontal>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Horizontal {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::Button {
                        content: Some("Click me!".to_string()),
                        props: HashMap::default(),
                        functions: HashMap::default(),
                    },
                    Component::Vertical {
                        content: None,
                        props: HashMap::default(),
                        children: vec![Component::Label {
                            content: "Hello, world!".to_string(),
                            props: HashMap::default(),
                            template: None,
                        }]
                    }
                ]
            }]
        );
    }

    #[test]
    fn parse_with_property_attributes() {
        tracing::info!("*** Starting test ***");

        let input = r#"
            <Horizontal>
                <Button color="red">Click me!</Button>
                <Vertical>
                    <Label>Hello, world!</Label>
                </Vertical>
            </Horizontal>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Horizontal {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::Button {
                        content: Some("Click me!".to_string()),
                        props: vec![("color".to_string(), "red".to_string())]
                            .into_iter()
                            .collect(),
                        functions: HashMap::default(),
                    },
                    Component::Vertical {
                        content: None,
                        props: HashMap::default(),
                        children: vec![Component::Label {
                            content: "Hello, world!".to_string(),
                            props: HashMap::default(),
                            template: None,
                        }]
                    }
                ]
            }]
        );
    }

    #[test]
    fn test_on_click_attr() {
        let input = r#"
            <Horizontal>
                <Button on_click="handle_click">Click me!</Button>
                <Vertical>
                    <Label>Hello, world!</Label>
                </Vertical>
            </Horizontal>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Horizontal {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::Button {
                        content: Some("Click me!".to_string()),
                        props: vec![("on_click".to_string(), "handle_click".to_string())]
                            .into_iter()
                            .collect(),
                        functions: HashMap::default(),
                    },
                    Component::Vertical {
                        content: None,
                        props: HashMap::default(),
                        children: vec![Component::Label {
                            content: "Hello, world!".to_string(),
                            props: HashMap::default(),
                            template: None,
                        }]
                    }
                ]
            }]
        );
    }

    // test function calls as attributes (increment button)
    #[test]
    fn test_function_call_attr() {
        let input = r#"
            <Horizontal>
                <Button on_click=increment()>Increment</Button>
                <Vertical>
                    <Label>The count is {{count}}</Label>
                </Vertical>
            </Horizontal>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Horizontal {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::Button {
                        content: Some("Increment".to_string()),
                        props: vec![("on_click".to_string(), "increment".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![("increment".to_string(), vec![])]
                            .into_iter()
                            .collect(),
                    },
                    Component::Vertical {
                        content: None,
                        props: HashMap::default(),
                        children: vec![Component::Label {
                            content: "The count is {{count}}".to_string(),
                            props: HashMap::default(),
                            template: Some(Template::new("The count is {{count}}")),
                        }]
                    }
                ]
            }]
        );
    }

    // TextEdit for username and password
    #[test]
    fn test_text_edit() {
        let input = r#"
            <Vertical>
                <TextEdit on_change=handle_change(username)>Username is {{username}}</TextEdit>
                <TextEdit on_change=handle_change(password)>Password</TextEdit>
                <Button on_click=login(username,password)>Login</Button>
                <Button on_click=logout(username, password)>Logout</Button>
            </Vertical>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Vertical {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::TextEdit {
                        content: "Username is {{username}}".to_string(),
                        props: vec![("on_change".to_string(), "handle_change".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "handle_change".to_string(),
                            vec!["username".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                        template: Some(Template::new("Username is {{username}}")),
                    },
                    Component::TextEdit {
                        content: "Password".to_string(),
                        props: vec![("on_change".to_string(), "handle_change".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "handle_change".to_string(),
                            vec!["password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                        template: None
                    },
                    Component::Button {
                        content: Some("Login".to_string()),
                        props: vec![("on_click".to_string(), "login".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "login".to_string(),
                            vec!["username".to_string(), "password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                    },
                    Component::Button {
                        content: Some("Logout".to_string()),
                        props: vec![("on_click".to_string(), "logout".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "logout".to_string(),
                            vec!["username".to_string(), "password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                    }
                ]
            }]
        );
    }

    #[test]
    fn test_text_edit_with_comments() {
        let input = r#"
            <Vertical>
                // This is a comment
                <TextEdit on_change=handle_change(username)>Username is {{username}}</TextEdit>
                /* This is a block comment */
                <TextEdit on_change=handle_change(password)>Password</TextEdit>
                <Button on_click=login(username,password)>Login</Button>
                <Button on_click=logout(username, password)>Logout</Button>
            </Vertical>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Vertical {
                content: None,
                props: HashMap::default(),
                children: vec![
                    Component::TextEdit {
                        content: "Username is {{username}}".to_string(),
                        props: vec![("on_change".to_string(), "handle_change".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "handle_change".to_string(),
                            vec!["username".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                        template: Some(Template::new("Username is {{username}}")),
                    },
                    Component::TextEdit {
                        content: "Password".to_string(),
                        props: vec![("on_change".to_string(), "handle_change".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "handle_change".to_string(),
                            vec!["password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                        template: None
                    },
                    Component::Button {
                        content: Some("Login".to_string()),
                        props: vec![("on_click".to_string(), "login".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "login".to_string(),
                            vec!["username".to_string(), "password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                    },
                    Component::Button {
                        content: Some("Logout".to_string()),
                        props: vec![("on_click".to_string(), "logout".to_string())]
                            .into_iter()
                            .collect(),
                        functions: vec![(
                            "logout".to_string(),
                            vec!["username".to_string(), "password".to_string()]
                        )]
                        .into_iter()
                        .collect(),
                    }
                ]
            }]
        );
    }

    // test Text with {{template}} in it
    #[test]
    fn test_text_with_template() {
        let input = r#"
            <Vertical>
                <Text>My name is {{name}}</Text>
            </Vertical>
        "#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            vec![Component::Vertical {
                content: None,
                props: HashMap::default(),
                children: vec![Component::Text {
                    content: "My name is {{name}}".to_string(),
                    template: Some(Template::new("My name is {{name}}")),
                    props: HashMap::default(),
                }]
            }]
        );
    }
}
