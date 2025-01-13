//! Crate Errors

pub enum Error {
    /// Parsing error
    Parse(String),
    /// Invalid action
    InvalidConversion(String),
}
