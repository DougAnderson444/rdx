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

    /// Selector Kind error
    #[error("Selector Kind Error: {0}")]
    Selector(#[from] scraper::error::SelectorErrorKind<'static>),
}
