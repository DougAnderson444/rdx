#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

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

    /// Parse error
    #[error("Parse Error: {0}")]
    Parse(String),

    /// Html parse error
    #[error("Html Parse Error: {0}")]
    HtmlParseError(String),
}
