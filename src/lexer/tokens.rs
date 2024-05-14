#[derive(Debug)]
pub enum Errors {
    UnrecognizedToken,
    UnexpectedToken,
    CompilerError,
    UnterminatedMultiLineComment,
    UnterminatedString,
    Unknown,
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::UnrecognizedToken => write!(f, "Unrecognized token"),
            Errors::UnexpectedToken => write!(f, "Unexpected token"),
            Errors::CompilerError => write!(f, "Compiler error"),
            Errors::UnterminatedMultiLineComment => write!(f, "Unterminated multi-line comment"),
            Errors::UnterminatedString => write!(f, "Unterminated string"),
            Errors::Unknown => write!(f, "Unknown error"),
        }
    }
}