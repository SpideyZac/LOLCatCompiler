use crate::lexer::lexer;
use crate::lexer::tokens;
use crate::parser::ast;

#[derive(Debug, Clone)]
pub struct ParserError<'a> {
    pub message: &'a str,
    pub token: lexer::LexedToken,
}

#[derive(Debug, Clone)]
pub struct ParserReturn<'a> {
    pub ast: ast::ProgramNode,
    pub errors: Vec<ParserError<'a>>,
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    pub tokens: Vec<lexer::LexedToken>,
    pub current: usize,
    pub errors: Vec<ParserError<'a>>,
    pub levels: Vec<usize>,
    pub level: usize,
    pub stmts: Vec<ast::StatementNode>,
}

impl<'a> Parser<'a> {
    // General Functions
    pub fn parse(t: Vec<lexer::LexedToken>) -> ParserReturn<'a> {
        let mut p = Parser {
            tokens: t,
            current: 0,
            errors: Vec::new(),
            levels: Vec::new(),
            level: 0,
            stmts: Vec::new(),
        };

        let program = p.parse_program();

        let mut filtered_errors: Vec<ParserError<'a>> = Vec::new();
        for (i, error) in p.errors.iter().enumerate() {
            let mut found_match = false;
            for (j, error2) in p.errors.iter().enumerate() {
                if i == j {
                    continue;
                }

                if error2.token.index >= error.token.index && p.levels[j] == p.levels[i] {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                filtered_errors.push(error.clone());
            }
        }

        ParserReturn {
            ast: program,
            errors: filtered_errors,
        }
    }

    pub fn check_ending(&mut self) -> bool {
        if self.check(tokens::Token::Newline) {
            self.consume_newline();
            return true;
        }
        if self.check(tokens::Token::Comma) {
            self.consume(tokens::Token::Comma);
            return true;
        }

        false
    }
}

impl<'a> Parser<'a> {
    // Parser Functions
    pub fn create_error(&mut self, parser_error: ParserError<'a>) {
        self.errors.push(parser_error);
        self.levels.push(self.level);
        self.prev_level();
    }

    pub fn check(&self, token: tokens::Token) -> bool {
        if self.peek().token == token {
            return true;
        }
        false
    }

    pub fn special_check(&self, name: &str) -> bool {
        if self.peek().token.to_name() == name.to_string() {
            return true;
        }
        false
    }

    pub fn check_amount(&self, token: tokens::Token, amount: usize) -> bool {
        if self.peek_amount(amount).token == token {
            return true;
        }
        false
    }

    pub fn next_level(&mut self) {
        self.level += 1;
    }

    pub fn prev_level(&mut self) {
        self.level -= 1;
    }

    pub fn reset(&mut self, num: usize) {
        self.current = num;
    }

    pub fn skip_newline(&mut self) {
        while self.check(tokens::Token::Newline) {
            self.advance();
        }
    }

    pub fn consume(&mut self, token: tokens::Token) -> Option<ast::TokenNode> {
        self.skip_newline();

        if self.check(token) {
            self.advance();
            return Some(ast::TokenNode {
                token: self.previous(),
            });
        }
        None
    }

    pub fn special_consume(&mut self, name: &str) -> Option<ast::TokenNode> {
        self.skip_newline();

        if self.special_check(name) {
            self.advance();
            return Some(ast::TokenNode {
                token: self.previous(),
            });
        }
        None
    }

    pub fn consume_newline(&mut self) -> Option<ast::TokenNode> {
        if self.check(tokens::Token::Newline) {
            self.advance();
            return Some(ast::TokenNode {
                token: self.previous(),
            });
        }
        None
    }

    pub fn previous(&self) -> lexer::LexedToken {
        self.tokens[self.current - 1].clone()
    }

    pub fn peek(&self) -> lexer::LexedToken {
        self.tokens[self.current].clone()
    }

    pub fn peek_amount(&self, amount: usize) -> lexer::LexedToken {
        self.tokens[self.current + amount].clone()
    }

    pub fn advance(&mut self) -> Option<lexer::LexedToken> {
        if !self.is_at_end() {
            self.current += 1;
            return Some(self.peek());
        }
        None
    }

    pub fn is_at_end(&self) -> bool {
        self.check(tokens::Token::EOF)
    }
}

impl<'a> Parser<'a> {
    // Node Functions
    pub fn parse_program(&mut self) -> ast::ProgramNode {
        self.next_level();
        while !self.is_at_end() {
            let parsed_statement = self.parse_statement();
            if let None = parsed_statement {
                self.create_error(ParserError {
                    message: "Expected valid statement line",
                    token: self.peek(),
                });
                return ast::ProgramNode {
                    statements: self.stmts.clone(),
                };
            }
            self.stmts.push(parsed_statement.unwrap());
        }

        self.prev_level();
        ast::ProgramNode {
            statements: self.stmts.clone(),
        }
    }

    pub fn parse_statement(&mut self) -> Option<ast::StatementNode> {
        self.next_level();

        let expression = self.parse_expression();
        if let Some(expression) = expression {
            if !self.check_ending() {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::Expression(expression),
            });
        }

        self.create_error(ParserError {
            message: "Expected valid statement or expression",
            token: self.peek(),
        });
        None
    }

    pub fn parse_expression(&mut self) -> Option<ast::ExpressionNode> {
        self.next_level();

        self.skip_newline();

        if self.special_check("NumberValue") {
            if let Some(number_value) = self.parse_number_value() {
                self.prev_level();
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::NumberValue(number_value),
                });
            }
        }

        if self.special_check("NumbarValue") {
            if let Some(numbar_value) = self.parse_numbar_value() {
                self.prev_level();
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::NumbarValue(numbar_value),
                });
            }
        }

        if self.special_check("YarnValue") {
            if let Some(yarn_value) = self.parse_yarn_value() {
                self.prev_level();
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::YarnValue(yarn_value),
                });
            }
        }

        if self.special_check("TroofValue") {
            if let Some(troof_value) = self.parse_troof_value() {
                self.prev_level();
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::TroofValue(troof_value),
                });
            }
        }

        self.create_error(ParserError {
            message: "Expected valid expression",
            token: self.peek(),
        });
        None
    }

    pub fn parse_number_value(&mut self) -> Option<ast::NumberValueNode> {
        self.next_level();

        let token = self.special_consume("NumberValue");
        if let Some(token) = token {
            self.prev_level();
            return Some(ast::NumberValueNode { token });
        }

        self.create_error(ParserError {
            message: "Expected number value token",
            token: self.peek(),
        });
        None
    }

    pub fn parse_numbar_value(&mut self) -> Option<ast::NumbarValueNode> {
        self.next_level();

        let token = self.special_consume("NumbarValue");
        if let Some(token) = token {
            self.prev_level();
            return Some(ast::NumbarValueNode { token });
        }

        self.create_error(ParserError {
            message: "Expected numbar value token",
            token: self.peek(),
        });
        None
    }

    pub fn parse_yarn_value(&mut self) -> Option<ast::YarnValueNode> {
        self.next_level();

        let token = self.special_consume("YarnValue");
        if let Some(token) = token {
            self.prev_level();
            return Some(ast::YarnValueNode { token });
        }

        self.create_error(ParserError {
            message: "Expected yarn value token",
            token: self.peek(),
        });
        None
    }

    pub fn parse_troof_value(&mut self) -> Option<ast::TroofValueNode> {
        self.next_level();

        let token = self.special_consume("TroofValue");
        if let Some(token) = token {
            self.prev_level();
            return Some(ast::TroofValueNode { token });
        }

        self.create_error(ParserError {
            message: "Expected troof value token",
            token: self.peek(),
        });
        None
    }
}
