use crate::pest::Rule;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] Box<pest::error::Error<Rule>>),

    /// Function not found in wasm component instance. Do you have the right plugin name?
    #[error("Function not found: {0}")]
    FuncNotFound(String),

    /// Wrong return type
    #[error("Wrong return type: {0}")]
    WrongReturnType(String),

    /// Instance not found
    #[error("Instance not found")]
    InstanceNotFound,

    /// From anyhow
    #[error("Anyhow Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}
