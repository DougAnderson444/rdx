use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

use crate::Error;

#[derive(Parser)]
#[grammar = "egui_layout.pest"]
struct EguiLayoutParser;

#[derive(Debug, PartialEq, Clone)]
pub enum Component {
    Document {
        children: Vec<Component>,
    },
    Horizontal {
        content: Option<String>,
        props: HashMap<String, String>,
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
    },
    Label {
        content: String,
        props: HashMap<String, String>,
    },
    Text {
        content: String,
        props: HashMap<String, String>,
    },
}

fn parse_element(pair: pest::iterators::Pair<'_, Rule>) -> Result<Component, Error> {
    match pair.as_rule() {
        Rule::document => {
            let inner = pair.clone().into_inner();
            let children: Vec<Component> = inner
                .filter_map(|p| match p.as_rule() {
                    Rule::element => Some(parse_element(p)),
                    _ => None,
                })
                .collect::<Result<_, _>>()?;
            Ok(Component::Document { children })
        }
        Rule::element => {
            let mut inner = pair.clone().into_inner();
            let open_tag = inner.next().unwrap();
            let mut tag_inner = open_tag.into_inner();
            let tag_name = tag_inner.next().unwrap().as_str();

            let mut props = HashMap::default();

            for p in tag_inner.filter(|p| p.as_rule() == Rule::attribute) {
                let mut attr_inner = p.into_inner();
                let name = attr_inner.next().unwrap().as_str().to_string();
                let value = attr_inner.next().unwrap().as_str().to_string();
                props.insert(name, value);
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
                                props: HashMap::default(),
                            }))
                        }
                    }
                    _ => None,
                })
                .collect::<Result<_, _>>()?;

            let res = match tag_name {
                "Horizontal" => Component::Horizontal {
                    content,
                    props,
                    children,
                },
                "Vertical" => Component::Vertical {
                    content,
                    props,
                    children,
                },
                "Button" => Component::Button { content, props },
                "Label" => Component::Label {
                    content: content.unwrap(),
                    props,
                },
                _ => {
                    return Err(Error::Parse(Box::new(pest::error::Error::new_from_span(
                        pest::error::ErrorVariant::CustomError {
                            message: format!("Unknown tag: {}", tag_name),
                        },
                        pair.as_span(),
                    ))))
                }
            };
            Ok(res)
        }
        _ => {
            return Err(Error::Parse(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Expected element, got {:?}", pair.as_rule()),
                },
                pair.as_span(),
            ))))
        }
    }
}

pub(crate) fn parse(input: &str) -> Result<Component, Error> {
    let pairs =
        EguiLayoutParser::parse(Rule::document, input).map_err(|e| Error::Parse(Box::new(e)))?;
    parse_element(pairs.peek().unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_parse_and_generate() {
        let input = r#"<Horizontal>
        </Horizontal>"#;
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            Component::Document {
                children: vec![Component::Horizontal {
                    content: None,
                    props: HashMap::default(),
                    children: vec![]
                }]
            }
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
            Component::Document {
                children: vec![Component::Horizontal {
                    content: None,
                    props: HashMap::default(),
                    children: vec![
                        Component::Button {
                            content: Some("Click me!".to_string()),
                            props: HashMap::default(),
                        },
                        Component::Vertical {
                            content: None,
                            props: HashMap::default(),
                            children: vec![Component::Label {
                                content: "Hello, world!".to_string(),
                                props: HashMap::default()
                            }]
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn parse_with_property_attributes() {
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
            Component::Document {
                children: vec![Component::Horizontal {
                    content: None,
                    props: HashMap::default(),
                    children: vec![
                        Component::Button {
                            content: Some("Click me!".to_string()),
                            props: vec![("color".to_string(), "red".to_string())]
                                .into_iter()
                                .collect(),
                        },
                        Component::Vertical {
                            content: None,
                            props: HashMap::default(),
                            children: vec![Component::Label {
                                content: "Hello, world!".to_string(),
                                props: HashMap::default()
                            }]
                        }
                    ]
                }]
            }
        );
    }
}
