use crate::lexer::tokens;

#[derive(Debug, Clone)]
pub struct LexedToken {
    pub token: tokens::Token,
    pub start: usize,
    pub end: usize,
    pub index: usize,
}

fn is_int(c: char) -> bool {
    c.is_digit(10)
}

fn is_char(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

pub struct Lexer<'a> {
    src: &'a str,

    pos: usize,
    read_pos: usize,
    curr_ch: char,
    token_count: usize,
}

impl<'a> Lexer<'a> {
    pub fn init(src: &'a str) -> Self {
        let mut l = Self {
            src,
            pos: 0,
            read_pos: 0,
            curr_ch: '\0',
            token_count: 0,
        };

        l.read_ch();
        l
    }

    fn read_ch(&mut self) {
        if self.read_pos >= self.src.len() {
            self.curr_ch = '\0';
            return;
        }

        self.curr_ch = self.src.chars().nth(self.read_pos).unwrap();
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn peek_ch(&self) -> char {
        if self.read_pos >= self.src.len() {
            return '\0';
        }

        self.src.chars().nth(self.read_pos).unwrap()
    }

    fn read_number(&mut self) -> tokens::Token {
        let start_pos = self.pos;
        let mut is_float = false;

        while is_int(self.peek_ch()) || self.peek_ch() == '.' {
            self.read_ch();
            if self.curr_ch == '.' && !is_float {
                is_float = true;
            } else if self.curr_ch == '.' && is_float {
                break;
            }
        }

        if is_float {
            return tokens::Token::NumbarValue(self.src[start_pos..self.read_pos].to_string());
        }

        tokens::Token::NumberValue(self.src[start_pos..self.read_pos].to_string())
    }

    pub fn special_check_identifier(&self, word: &str) -> bool {
        return match word {
            // ignore this crap lmao
            "I" => false,
            "HAS" => false,
            "A" => false,
            "R" => false,
            "ITZ" => false,
            "AN" => false,
            "SUM" => false,
            "OF" => false,
            "DIFF" => false,
            "PRODUKT" => false,
            "QUOSHUNT" => false,
            "MOD" => false,
            "BIGGR" => false,
            "SMALLR" => false,
            "BOTH" => false,
            "EITHER" => false,
            "WON" => false,
            "NOT" => false,
            "ALL" => false,
            "ANY" => false,
            "MKAY" => false,
            "SAEM" => false,
            "DIFFRINT" => false,
            "MAEK" => false,
            "VISIBLE" => false,
            "GIMMEH" => false,
            "IT" => false,
            "O" => false,
            "RLY" => false,
            "YA" => false,
            "NO" => false,
            "WAI" => false,
            "OIC" => false,
            "MEBBE" => false,
            "WTF" => false,
            "OMG" => false,
            "GTFO" => false,
            "OMGWTF" => false,
            "IM" => false,
            "IN" => false,
            "YR" => false,
            "TIL" => false,
            "WILE" => false,
            "OUTTA" => false,
            "UPPIN" => false,
            "NERFIN" => false,
            "HOW" => false,
            "IZ" => false,
            "IF" => false,
            "U" => false,
            "SAY" => false,
            "SO" => false,
            "HAI" => false,
            "KTHXBYE" => false,
            "SMOOSH" => false,
            "NUMBER" => false,
            "NUMBAR" => false,
            "YARN" => false,
            "TROOF" => false,
            "NOOB" => false,
            "FOUND" => false,
            _ => true,
        };
    }

    fn read_word(&mut self) -> tokens::Token {
        let start_pos = self.pos;

        while is_char(self.peek_ch()) || is_int(self.peek_ch()) {
            self.read_ch();
        }

        let word = &self.src[start_pos..self.read_pos];
        if self.special_check_identifier(word) {
            return tokens::Token::Identifier(word.to_string());
        }
        tokens::Token::Word(word.to_string())
    }

    fn read_string(&mut self) -> tokens::Token {
        self.read_ch();
        let mut ignore = false;

        let mut string_array: Vec<char> = Vec::new();

        while (self.curr_ch != '"' || ignore) && !is_newline(self.curr_ch) && self.curr_ch != '\0' {
            if self.curr_ch == ':' && !ignore {
                ignore = true;
            } else {
                ignore = false;
                string_array.push(self.curr_ch);
            }
            self.read_ch();
        }

        if self.curr_ch == '\0' || self.curr_ch != '"' {
            return tokens::Token::Illegal(tokens::Errors::UnterminatedString);
        }

        tokens::Token::YarnValue(string_array.iter().collect())
    }

    fn la(&mut self, t: &str) -> bool {
        if self.read_pos + t.len() >= self.src.len() {
            return false;
        }
        let mut success = false;
        if self.src[self.read_pos..self.read_pos + t.len()] == *t {
            success = true;
        }

        if success {
            for _ in 0..t.len() {
                self.read_ch();
            }
        }

        success
    }

    fn read_multiline(&mut self) -> tokens::Token {
        let mut comment_contents: Vec<char> = Vec::new();

        while self.curr_ch != '\0' {
            if self.la("TLDR") {
                break;
            }

            comment_contents.push(self.curr_ch);
            self.read_ch();
        }

        if self.curr_ch == '\0' {
            return tokens::Token::Illegal(tokens::Errors::UnterminatedMultiLineComment);
        }

        tokens::Token::MultiLineComment(comment_contents.iter().collect())
    }

    fn skip_whitespace(&mut self) {
        while self.curr_ch == ' ' || self.curr_ch == '\t' || self.curr_ch == '\r' {
            self.read_ch();
        }
    }

    fn skip_single_comment(&mut self) {
        while !is_newline(self.curr_ch) && self.curr_ch != '\0' {
            self.read_ch();
        }
    }

    pub fn next_token(&mut self) -> LexedToken {
        self.skip_whitespace();
        let start = self.pos;

        let token = match self.curr_ch {
            '0'..='9' => self.read_number(),
            '-' => {
                if is_int(self.peek_ch()) {
                    self.read_number()
                } else {
                    tokens::Token::Illegal(tokens::Errors::UnexpectedToken)
                }
            }
            'W' => {
                if self.la("IN") {
                    tokens::Token::TroofValue("WIN".to_string())
                } else {
                    self.read_word()
                }
            }
            'F' => {
                if self.la("AIL") {
                    tokens::Token::TroofValue("FAIL".to_string())
                } else {
                    self.read_word()
                }
            }
            'A'..='Z' => {
                if self.curr_ch == 'O' && self.la("BTW") {
                    self.read_multiline()
                } else {
                    self.read_word()
                }
            }
            'a'..='z' => self.read_word(),
            '_' => self.read_word(),
            '"' => self.read_string(),
            ',' => tokens::Token::Comma,
            '!' => tokens::Token::ExclamationMark,
            '?' => tokens::Token::QuestionMark,
            '\n' => tokens::Token::Newline,

            '\0' => tokens::Token::EOF,
            _ => tokens::Token::Illegal(tokens::Errors::UnrecognizedToken),
        };

        if let tokens::Token::SingleLineComment = token {
            self.skip_single_comment();
        }

        let end = self.read_pos;
        self.read_ch();

        self.token_count += 1;
        LexedToken {
            token,
            start,
            end,
            index: self.token_count - 1,
        }
    }

    pub fn get_tokens(&mut self) -> Vec<LexedToken> {
        let mut tokens: Vec<LexedToken> = Vec::new();

        while self.curr_ch != '\0' {
            let token = self.next_token();
            match token.token {
                tokens::Token::SingleLineComment => {}
                tokens::Token::MultiLineComment(_) => {}
                _ => tokens.push(token),
            }
        }
        tokens.push(self.next_token());

        tokens
    }

    pub fn has_errors(tokens: &Vec<LexedToken>) -> bool {
        for token in tokens {
            if let tokens::Token::Illegal(_) = token.token {
                return true;
            }
        }

        false
    }

    pub fn get_first_error<'b>(tokens: &'b Vec<LexedToken>) -> Option<&'b LexedToken> {
        for token in tokens {
            if let tokens::Token::Illegal(_) = token.token {
                return Some(token);
            }
        }

        None
    }
}
