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

                if (error2.token.index >= error.token.index && p.levels[j] == p.levels[i])
                    || p.current > error.token.index
                {
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
        if self.check_newline() {
            self.consume_newlines();
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

    pub fn check(&mut self, token: tokens::Token) -> bool {
        if self.peek().token == token {
            return true;
        }
        false
    }

    pub fn check_newline(&self) -> bool {
        self.peek().token == tokens::Token::Newline
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

    pub fn consume(&mut self, token: tokens::Token) -> Option<ast::TokenNode> {
        if self.check(token) {
            self.advance();
            return Some(ast::TokenNode {
                token: self.previous(),
            });
        }
        None
    }

    pub fn special_consume(&mut self, name: &str) -> Option<ast::TokenNode> {
        if self.special_check(name) {
            self.advance();
            return Some(ast::TokenNode {
                token: self.previous(),
            });
        }
        None
    }

    pub fn consume_newlines(&mut self) {
        while self.check_newline() {
            self.advance();
        }
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

    pub fn is_at_end(&mut self) -> bool {
        self.check(tokens::Token::EOF)
    }
}

impl<'a> Parser<'a> {
    // Node Functions
    pub fn parse_program(&mut self) -> ast::ProgramNode {
        self.next_level();

        let hai = self.special_consume("Word_HAI");
        if let None = hai {
            self.create_error(ParserError {
                message: "Expected HAI token to start program",
                token: self.peek(),
            });
            return ast::ProgramNode {
                statements: self.stmts.clone(),
            };
        }

        let version = self.parse_numbar_value();
        if let None = version {
            self.create_error(ParserError {
                message: "Expected valid version numbar",
                token: self.peek(),
            });
            return ast::ProgramNode {
                statements: self.stmts.clone(),
            };
        }

        if let Some(version) = version {
            if version.value() != 1.2 {
                self.create_error(ParserError {
                    message: "Expected version 1.2",
                    token: version.token.token,
                });
                return ast::ProgramNode {
                    statements: self.stmts.clone(),
                };
            }
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected comma or newline to end statement",
                token: self.peek(),
            });
            return ast::ProgramNode {
                statements: self.stmts.clone(),
            };
        }

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

        if self.stmts.len() == 0 {
            self.create_error(ParserError {
                message: "Expected KTHXBYE statement to end program",
                token: self.peek(),
            });
            return ast::ProgramNode {
                statements: self.stmts.clone(),
            };
        }
        match self.stmts[self.stmts.len() - 1].value {
            ast::StatementNodeValueOption::KTHXBYEStatement(_) => {}
            _ => {
                self.create_error(ParserError {
                    message: "Expected KTHXBYE statement to end program",
                    token: self.peek(),
                });
                return ast::ProgramNode {
                    statements: self.stmts.clone(),
                };
            }
        }

        self.prev_level();
        ast::ProgramNode {
            statements: self.stmts.clone(),
        }
    }

    pub fn parse_statement(&mut self) -> Option<ast::StatementNode> {
        self.next_level();

        let variable_declaration_statement = self.parse_variable_declaration_statement();
        if let Some(variable_declaration_statement) = variable_declaration_statement {
            if !self.check_ending() && !self.special_check("Word_R") {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::VariableDeclarationStatement(
                    variable_declaration_statement,
                ),
            });
        }

        let variable_assignment_statement = self.parse_variable_assignment_statement();
        if let Some(variable_assignment_statement) = variable_assignment_statement {
            if !self.check_ending() {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::VariableAssignmentStatement(
                    variable_assignment_statement,
                ),
            });
        }

        let kthxbye_statement = self.special_consume("Word_KTHXBYE");
        if let Some(kthxbye_statement) = kthxbye_statement {
            if !self.check_ending() && !self.is_at_end() {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::KTHXBYEStatement(kthxbye_statement),
            });
        }

        let visible_statement = self.parse_visible_statement();
        if let Some(visible_statement) = visible_statement {
            // visible checks for ending itself

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::VisibleStatement(visible_statement),
            });
        }

        let gimmeh_statement = self.parse_gimmeh_statement();
        if let Some(gimmeh_statement) = gimmeh_statement {
            if !self.check_ending() {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::GimmehStatement(gimmeh_statement),
            });
        }

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

    pub fn parse_variable_declaration_statement(
        &mut self,
    ) -> Option<ast::VariableDeclarationStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_I") {
            self.create_error(ParserError {
                message: "Expected I keyword to declare variable",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_HAS") {
            self.create_error(ParserError {
                message: "Expected HAS keyword to declare variable",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_A") {
            self.create_error(ParserError {
                message: "Expected A keyword to declare variable",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let identifier = self.special_consume("Identifier");
        if let None = identifier {
            self.create_error(ParserError {
                message: "Expected identifier for variable declaration",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_ITZ") {
            self.create_error(ParserError {
                message: "Expected ITZ keyword to declare variable",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let Some(type_) = self.special_consume("Word_NUMBER") {
            self.prev_level();
            return Some(ast::VariableDeclarationStatementNode {
                identifier: identifier.unwrap(),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_NUMBAR") {
            self.prev_level();
            return Some(ast::VariableDeclarationStatementNode {
                identifier: identifier.unwrap(),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_YARN") {
            self.prev_level();
            return Some(ast::VariableDeclarationStatementNode {
                identifier: identifier.unwrap(),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_TROOF") {
            self.prev_level();
            return Some(ast::VariableDeclarationStatementNode {
                identifier: identifier.unwrap(),
                type_,
            });
        }

        self.create_error(ParserError {
            message: "Expected valid type for variable declaration",
            token: self.peek(),
        });
        self.reset(start);
        None
    }

    pub fn parse_variable_assignment_statement(
        &mut self,
    ) -> Option<ast::VariableAssignmentStatementNode> {
        self.next_level();
        let start = self.current;

        let identifier = self.special_consume("Identifier");
        let mut var_dec: Option<ast::StatementNode> = None;

        if let None = identifier {
            if self.stmts.len() > 0 {
                match self.stmts[self.stmts.len() - 1].value {
                    ast::StatementNodeValueOption::VariableDeclarationStatement(_) => {
                        var_dec = Some(self.stmts.pop().unwrap());
                    }
                    _ => {
                        self.create_error(ParserError {
                            message: "Expected identifier or variable declaration for variable assignment",
                            token: self.peek(),
                        });
                        return None;
                    }
                }
            } else {
                self.create_error(ParserError {
                    message: "Expected identifier or variable declaration for variable assignment",
                    token: self.peek(),
                });
                return None;
            }
        }

        if let None = self.special_consume("Word_R") {
            self.create_error(ParserError {
                message: "Expected R keyword to assign variable",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression = self.parse_expression();
        if let None = expression {
            self.create_error(ParserError {
                message: "Expected valid expression for variable assignment",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let Some(dec) = var_dec {
            self.prev_level();
            match dec.value {
                ast::StatementNodeValueOption::VariableDeclarationStatement(node) => {
                    return Some(ast::VariableAssignmentStatementNode {
                        variable:
                            ast::VariableAssignmentNodeVariableOption::VariableDeclerationStatement(
                                node,
                            ),
                        expression: expression.unwrap(),
                    });
                }
                _ => {}
            }
        }

        self.prev_level();
        return Some(ast::VariableAssignmentStatementNode {
            variable: ast::VariableAssignmentNodeVariableOption::Identifier(identifier.unwrap()),
            expression: expression.unwrap(),
        });
    }

    pub fn parse_visible_statement(&mut self) -> Option<ast::VisibleStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_VISIBLE") {
            self.create_error(ParserError {
                message: "Expected VISIBLE keyword to output to console",
                token: self.peek(),
            });
            return None;
        }

        let mut expressions: Vec<ast::ExpressionNode> = Vec::new();
        while !self.is_at_end() {
            let expression = self.parse_expression();
            if let None = expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for VISIBLE statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            expressions.push(expression.unwrap());

            if self.check_ending() || self.check(tokens::Token::ExclamationMark) {
                break;
            }
        }

        let exclamation_mark = self.consume(tokens::Token::ExclamationMark);
        if let Some(exclamation_mark) = exclamation_mark {
            if !self.check_ending() {
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            self.prev_level();
            return Some(ast::VisibleStatementNode {
                expressions,
                exclamation: Some(exclamation_mark),
            });
        }

        self.prev_level();
        Some(ast::VisibleStatementNode {
            expressions,
            exclamation: None,
        })
    }

    pub fn parse_gimmeh_statement(&mut self) -> Option<ast::GimmehStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_GIMMEH") {
            self.create_error(ParserError {
                message: "Expected GIMMEH keyword to get input",
                token: self.peek(),
            });
            return None;
        }

        let identifier = self.special_consume("Identifier");
        if let None = identifier {
            self.create_error(ParserError {
                message: "Expected identifier for GIMMEH statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        return Some(ast::GimmehStatementNode {
            identifier: identifier.unwrap(),
        });
    }
}
