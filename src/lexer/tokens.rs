#[derive(Debug)]
pub enum Errors {
    UnrecognizedToken,
    UnexpectedToken,
    UnterminatedMultiLineComment,
    UnterminatedString,
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::UnrecognizedToken => write!(f, "Unrecognized token"),
            Errors::UnexpectedToken => write!(f, "Unexpected token"),
            Errors::UnterminatedMultiLineComment => write!(f, "Unterminated multi-line comment"),
            Errors::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Illegal(Errors),
    EOF,
    Newline,

    Number,
    Numbar,
    Noob,
    Troof,
    Yarn,

    Word(String),

    Comma,
    ExclamationMark,
    QuestionMark,

    SingleLineComment,
    MultiLineComment(String),

    NumberValue(String),
    NumbarValue(String),
    YarnValue(String),
    TroofValue(String),
}
