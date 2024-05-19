#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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
    Identifier(String),

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

impl Token {
    pub fn to_name(&self) -> String {
        match self {
            Token::Illegal(_) => "Illegal".to_string(),
            Token::EOF => "EOF".to_string(),
            Token::Newline => "Newline".to_string(),
            Token::Number => "Number".to_string(),
            Token::Numbar => "Numbar".to_string(),
            Token::Noob => "Noob".to_string(),
            Token::Troof => "Troof".to_string(),
            Token::Yarn => "Yarn".to_string(),
            Token::Word(w) => format!("Word_{}", w).to_string(),
            Token::Identifier(_) => "Identifier".to_string(),
            Token::Comma => "Comma".to_string(),
            Token::ExclamationMark => "ExclamationMark".to_string(),
            Token::QuestionMark => "QuestionMark".to_string(),
            Token::SingleLineComment => "SingleLineComment".to_string(),
            Token::MultiLineComment(_) => "MultiLineComment".to_string(),
            Token::NumberValue(_) => "NumberValue".to_string(),
            Token::NumbarValue(_) => "NumbarValue".to_string(),
            Token::YarnValue(_) => "YarnValue".to_string(),
            Token::TroofValue(_) => "TroofValue".to_string(),
        }
    }
}
