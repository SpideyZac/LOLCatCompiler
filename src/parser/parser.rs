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

    pub fn special_check_amount(&self, name: &str, amount: usize) -> bool {
        if self.peek_amount(amount).token.to_name() == name.to_string() {
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
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
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
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
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
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
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
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::GimmehStatement(gimmeh_statement),
            });
        }

        let if_statement = self.parse_if_statement();
        if let Some(if_statement) = if_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::IfStatement(if_statement),
            });
        }

        let switch_statement = self.parse_switch_statement();
        if let Some(switch_statement) = switch_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::SwitchStatement(switch_statement),
            });
        }

        let gtfo_statement = self.special_consume("Word_GTFO");
        if let Some(gtfo_statement) = gtfo_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::GTFOStatement(gtfo_statement),
            });
        }

        let loop_statement = self.parse_loop_statement();
        if let Some(loop_statement) = loop_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::LoopStatement(loop_statement),
            });
        }

        let return_statement = self.parse_return_statement();
        if let Some(return_statement) = return_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::ReturnStatement(return_statement),
            });
        }

        let function_definition_statement = self.parse_function_definition_statement();
        if let Some(function_definition_statement) = function_definition_statement {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
                return None;
            }

            self.prev_level();
            return Some(ast::StatementNode {
                value: ast::StatementNodeValueOption::FunctionDefinitionStatement(
                    function_definition_statement,
                ),
            });
        }

        let expression = self.parse_expression();
        if let Some(expression) = expression {
            if !self.check_ending() {
                self.next_level();
                self.create_error(ParserError {
                    message: "Expected comma or newline to end statement",
                    token: self.peek(),
                });
                self.prev_level();
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
        if self.special_check("NumberValue") {
            if let Some(number_value) = self.parse_number_value() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::NumberValue(number_value),
                });
            }
        }

        if self.special_check("NumbarValue") {
            if let Some(numbar_value) = self.parse_numbar_value() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::NumbarValue(numbar_value),
                });
            }
        }

        if self.special_check("YarnValue") {
            if let Some(yarn_value) = self.parse_yarn_value() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::YarnValue(yarn_value),
                });
            }
        }

        if self.special_check("TroofValue") {
            if let Some(troof_value) = self.parse_troof_value() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::TroofValue(troof_value),
                });
            }
        }

        if self.special_check("Identifier") {
            if let Some(variable_reference) = self.parse_variable_reference_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::VariableReference(variable_reference),
                });
            }
        }

        if self.special_check("Word_SUM") {
            if let Some(sum_expression) = self.parse_sum_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::SumExpression(sum_expression),
                });
            }
        }

        if self.special_check("Word_DIFF") {
            if let Some(diff_expression) = self.parse_diff_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::DiffExpression(diff_expression),
                });
            }
        }

        if self.special_check("Word_PRODUKT") {
            if let Some(produkt_expression) = self.parse_produkt_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::ProduktExpression(produkt_expression),
                });
            }
        }

        if self.special_check("Word_QUOSHUNT") {
            if let Some(quoshunt_expression) = self.parse_quoshunt_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::QuoshuntExpression(quoshunt_expression),
                });
            }
        }

        if self.special_check("Word_MOD") {
            if let Some(mod_expression) = self.parse_mod_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::ModExpression(mod_expression),
                });
            }
        }

        if self.special_check("Word_BIGGR") {
            if let Some(biggr_expression) = self.parse_biggr_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::BiggrExpression(biggr_expression),
                });
            }
        }

        if self.special_check("Word_SMALLR") {
            if let Some(smallr_expression) = self.parse_smallr_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::SmallrExpression(smallr_expression),
                });
            }
        }

        if self.special_check("Word_BOTH") && self.special_check_amount("Word_OF", 1) {
            if let Some(both_of_expression) = self.parse_both_of_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::BothOfExpression(both_of_expression),
                });
            }
        }

        if self.special_check("Word_EITHER") {
            if let Some(either_expression) = self.parse_either_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::EitherOfExpression(either_expression),
                });
            }
        }

        if self.special_check("Word_WON") {
            if let Some(won_expression) = self.parse_won_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::WonOfExpression(won_expression),
                });
            }
        }

        if self.special_check("Word_NOT") {
            if let Some(not_expression) = self.parse_not_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::NotExpression(not_expression),
                });
            }
        }

        if self.special_check("Word_ALL") {
            if let Some(all_of_expression) = self.parse_all_of_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::AllOfExpression(all_of_expression),
                });
            }
        }

        if self.special_check("Word_ANY") {
            if let Some(any_of_expression) = self.parse_any_of_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::AnyOfExpression(any_of_expression),
                });
            }
        }

        if self.special_check("Word_BOTH") && self.special_check_amount("Word_SAEM", 1) {
            if let Some(both_saem_expression) = self.parse_both_saem_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::BothSaemExpression(both_saem_expression),
                });
            }
        }

        if self.special_check("Word_DIFFRINT") {
            if let Some(diffrint_expression) = self.parse_diffrint_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::DiffrintExpression(diffrint_expression),
                });
            }
        }

        if self.special_check("Word_SMOOSH") {
            if let Some(smoosh_expression) = self.parse_smoosh_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::SmooshExpression(smoosh_expression),
                });
            }
        }

        if self.special_check("Word_MAEK") {
            if let Some(maek_expression) = self.parse_maek_expression() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::MaekExpression(maek_expression),
                });
            }
        }

        if self.special_check("Word_IT") {
            if let Some(it_reference) = self.parse_it_reference() {
                return Some(ast::ExpressionNode {
                    value: ast::ExpressionNodeValueOption::ItReference(it_reference),
                });
            }
        }

        self.create_error(ParserError {
            message: "Expected valid expression",
            token: self.peek(),
        });
        self.next_level(); // prevent level from changing
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

    pub fn parse_variable_reference_expression(&mut self) -> Option<ast::VariableReferenceNode> {
        self.next_level();

        let identifier = self.special_consume("Identifier");
        if let Some(identifier) = identifier {
            self.prev_level();
            return Some(ast::VariableReferenceNode { identifier });
        }

        self.create_error(ParserError {
            message: "Expected identifier for variable reference",
            token: self.peek(),
        });
        None
    }

    pub fn parse_sum_expression(&mut self) -> Option<ast::SumExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_SUM") {
            self.create_error(ParserError {
                message: "Expected SUM keyword for sum expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for sum expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for sum expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for sum expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for sum expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::SumExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_diff_expression(&mut self) -> Option<ast::DiffExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_DIFF") {
            self.create_error(ParserError {
                message: "Expected DIFF keyword for diff expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for diff expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for diff expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for diff expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for diff expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::DiffExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_produkt_expression(&mut self) -> Option<ast::ProduktExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_PRODUKT") {
            self.create_error(ParserError {
                message: "Expected PRODUKT keyword for product expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for product expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for product expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for product expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for product expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::ProduktExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_quoshunt_expression(&mut self) -> Option<ast::QuoshuntExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_QUOSHUNT") {
            self.create_error(ParserError {
                message: "Expected QUOSHUNT keyword for quotient expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for quotient expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for quotient expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for quotient expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for quotient expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::QuoshuntExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_mod_expression(&mut self) -> Option<ast::ModExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_MOD") {
            self.create_error(ParserError {
                message: "Expected MOD keyword for modulo expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for modulo expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for modulo expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for modulo expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for modulo expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::ModExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_biggr_expression(&mut self) -> Option<ast::BiggrExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_BIGGR") {
            self.create_error(ParserError {
                message: "Expected BIGGR keyword for greater expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for greater expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for greater expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for greater expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for greater expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::BiggrExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_smallr_expression(&mut self) -> Option<ast::SmallrExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_SMALLR") {
            self.create_error(ParserError {
                message: "Expected SMALLR keyword for lesser expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for lesser expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for lesser expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for lesser expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for lesser expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::SmallrExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_both_of_expression(&mut self) -> Option<ast::BothOfExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_BOTH") {
            self.create_error(ParserError {
                message: "Expected BOTH keyword for both of expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for both of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for both of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for both of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for both of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::BothOfExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_either_expression(&mut self) -> Option<ast::EitherOfExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_EITHER") {
            self.create_error(ParserError {
                message: "Expected EITHER keyword for either of expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for either of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for either of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for either of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for either of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::EitherOfExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_won_expression(&mut self) -> Option<ast::WonOfExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_WON") {
            self.create_error(ParserError {
                message: "Expected WON keyword for won of expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for won of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for won of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for won of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for won of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::WonOfExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_not_expression(&mut self) -> Option<ast::NotExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_NOT") {
            self.create_error(ParserError {
                message: "Expected NOT keyword for not expression",
                token: self.peek(),
            });
            return None;
        }

        let expression = self.parse_expression();
        if let None = expression {
            self.create_error(ParserError {
                message: "Expected valid expression for not expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::NotExpressionNode {
            expression: Box::new(expression.unwrap()),
        })
    }

    pub fn parse_all_of_expression(&mut self) -> Option<ast::AllOfExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_ALL") {
            self.create_error(ParserError {
                message: "Expected ALL keyword for all of expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for all of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut expressions = Vec::new();
        while !self.is_at_end() {
            let expression = self.parse_expression();
            if let None = expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for all of expression",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
            expressions.push(expression.unwrap());

            if self.special_check("Word_AN") {
                self.special_consume("Word_AN");
            } else {
                break;
            }
        }

        if let None = self.special_consume("Word_MKAY") {
            self.create_error(ParserError {
                message: "Expected MKAY keyword for all of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::AllOfExpressionNode { expressions })
    }

    pub fn parse_any_of_expression(&mut self) -> Option<ast::AnyOfExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_ANY") {
            self.create_error(ParserError {
                message: "Expected ANY keyword for any of expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_OF") {
            self.create_error(ParserError {
                message: "Expected OF keyword for any of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut expressions = Vec::new();
        while !self.is_at_end() {
            let expression = self.parse_expression();
            if let None = expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for any of expression",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
            expressions.push(expression.unwrap());

            if self.special_check("Word_AN") {
                self.special_consume("Word_AN");
            } else {
                break;
            }
        }

        if let None = self.special_consume("Word_MKAY") {
            self.create_error(ParserError {
                message: "Expected MKAY keyword for any of expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::AnyOfExpressionNode { expressions })
    }

    pub fn parse_both_saem_expression(&mut self) -> Option<ast::BothSaemExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_BOTH") {
            self.create_error(ParserError {
                message: "Expected BOTH keyword for both saem expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_SAEM") {
            self.create_error(ParserError {
                message: "Expected SAEM keyword for both saem expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for both saem expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for both saem expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for both saem expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::BothSaemExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_diffrint_expression(&mut self) -> Option<ast::DiffrintExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_DIFFRINT") {
            self.create_error(ParserError {
                message: "Expected DIFFRINT keyword for different expression",
                token: self.peek(),
            });
            return None;
        }

        let expression1 = self.parse_expression();
        if let None = expression1 {
            self.create_error(ParserError {
                message: "Expected valid expression for different expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_AN") {
            self.create_error(ParserError {
                message: "Expected AN keyword for different expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression2 = self.parse_expression();
        if let None = expression2 {
            self.create_error(ParserError {
                message: "Expected valid expression for different expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::DiffrintExpressionNode {
            left: Box::new(expression1.unwrap()),
            right: Box::new(expression2.unwrap()),
        })
    }

    pub fn parse_smoosh_expression(&mut self) -> Option<ast::SmooshExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_SMOOSH") {
            self.create_error(ParserError {
                message: "Expected SMOOSH keyword for smoosh expression",
                token: self.peek(),
            });
            return None;
        }

        let mut expressions = Vec::new();
        while !self.is_at_end() {
            let expression = self.parse_expression();
            if let None = expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for smoosh expression",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
            expressions.push(expression.unwrap());

            if self.special_check("Word_AN") {
                self.special_consume("Word_AN");
            } else {
                break;
            }
        }

        if let None = self.special_consume("Word_MKAY") {
            self.create_error(ParserError {
                message: "Expected MKAY keyword for smoosh expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::SmooshExpressionNode { expressions })
    }

    pub fn parse_maek_expression(&mut self) -> Option<ast::MaekExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_MAEK") {
            self.create_error(ParserError {
                message: "Expected MAEK keyword for type conversion expression",
                token: self.peek(),
            });
            return None;
        }

        let expression = self.parse_expression();
        if let None = expression {
            self.create_error(ParserError {
                message: "Expected valid expression for type conversion expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_A") {
            self.create_error(ParserError {
                message: "Expected A keyword for type conversion expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let Some(type_) = self.special_consume("Word_NUMBER") {
            self.prev_level();
            return Some(ast::MaekExpressionNode {
                expression: Box::new(expression.unwrap()),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_NUMBAR") {
            self.prev_level();
            return Some(ast::MaekExpressionNode {
                expression: Box::new(expression.unwrap()),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_YARN") {
            self.prev_level();
            return Some(ast::MaekExpressionNode {
                expression: Box::new(expression.unwrap()),
                type_,
            });
        }

        if let Some(type_) = self.special_consume("Word_TROOF") {
            self.prev_level();
            return Some(ast::MaekExpressionNode {
                expression: Box::new(expression.unwrap()),
                type_,
            });
        }

        self.create_error(ParserError {
            message: "Expected valid type for type conversion expression",
            token: self.peek(),
        });
        self.reset(start);
        None
    }

    pub fn parse_it_reference(&mut self) -> Option<ast::ItReferenceNode> {
        self.next_level();

        let token = self.special_consume("Word_IT");
        if let None = token {
            self.create_error(ParserError {
                message: "Expected IT keyword for it number reference",
                token: self.peek(),
            });
            return None;
        }

        self.prev_level();
        Some(ast::ItReferenceNode {
            token: token.unwrap(),
        })
    }

    pub fn parse_function_call_expression(&mut self) -> Option<ast::FunctionCallExpressionNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_I") {
            self.create_error(ParserError {
                message: "Expected I keyword for function call expression",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_IZ") {
            self.create_error(ParserError {
                message: "Expected IZ keyword for function call expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let identifier = self.special_consume("Identifier");
        if let None = identifier {
            self.create_error(ParserError {
                message: "Expected identifier for function call expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut arguments = Vec::new();
        let mut has_args = false;
        while !self.is_at_end() {
            if let None = self.special_consume("Word_YR") {
                if !has_args {
                    break;
                }
                self.create_error(ParserError {
                    message: "Expected YR keyword for function call expression",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            has_args = true;

            let expression = self.parse_expression();
            if let None = expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for function call expression",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
            arguments.push(expression.unwrap());

            if self.special_check("Word_AN") {
                self.special_consume("Word_AN");
            } else {
                break;
            }
        }

        if let None = self.special_consume("Word_MKAY") {
            self.create_error(ParserError {
                message: "Expected MKAY keyword for function call expression",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        Some(ast::FunctionCallExpressionNode {
            identifier: identifier.unwrap(),
            arguments,
        })
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
            if let Some(dec) = var_dec {
                self.stmts.push(dec);
            }
            self.reset(start);
            return None;
        }

        let expression = self.parse_expression();
        if let None = expression {
            self.create_error(ParserError {
                message: "Expected valid expression for variable assignment",
                token: self.peek(),
            });
            if let Some(dec) = var_dec {
                self.stmts.push(dec);
            }
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

    pub fn parse_if_statement(&mut self) -> Option<ast::IfStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_O") {
            self.create_error(ParserError {
                message: "Expected O keyword to start if statement",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_RLY") {
            self.create_error(ParserError {
                message: "Expected RLY keyword to start if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.consume(tokens::Token::QuestionMark) {
            self.create_error(ParserError {
                message: "Expected ? to start if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_YA") {
            self.create_error(ParserError {
                message: "Expected YA keyword to start if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_RLY") {
            self.create_error(ParserError {
                message: "Expected RLY keyword to start if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected newline or comma to end if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut statements = Vec::new();
        while !self.is_at_end() {
            let statement = self.parse_statement();
            if let None = statement {
                self.create_error(ParserError {
                    message: "Expected valid statement for if statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            statements.push(statement.unwrap());

            if self.special_check("Word_OIC")
                || (self.special_check("Word_NO") && self.special_check_amount("Word_WAI", 1))
                || self.special_check("Word_MEBBE")
            {
                break;
            }
        }

        let mut else_if_nodes: Vec<ast::ElseIfStatementNode> = Vec::new();
        while !self.is_at_end() {
            if self.special_check("Word_OIC")
                || (self.special_check("Word_NO") && self.special_check_amount("Word_WAI", 1))
            {
                break;
            }

            let statement = self.parse_statement();
            if let Some(s) = statement {
                if else_if_nodes.len() == 0 {
                    self.create_error(ParserError {
                        message: "Expected MEBBE keyword to start else if statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }

                let last = else_if_nodes.len() - 1;
                else_if_nodes[last].statements.push(s);
                continue;
            } else if else_if_nodes.len() > 0 {
                self.create_error(ParserError {
                    message: "Expected valid statement for else if statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            if let None = self.special_consume("Word_MEBBE") {
                self.create_error(ParserError {
                    message: "Expected MEBBE keyword to start else if statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            } else {
                let expression = self.parse_expression();
                if let None = expression {
                    self.create_error(ParserError {
                        message: "Expected valid expression for else if statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }

                else_if_nodes.push(ast::ElseIfStatementNode {
                    expression: expression.unwrap(),
                    statements: Vec::new(),
                });

                if !self.check_ending() {
                    self.create_error(ParserError {
                        message: "Expected newline or comma to end else if statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }
            }
        }

        let mut else_statements = Vec::new();
        if self.special_check("Word_NO") && self.special_check_amount("Word_WAI", 1) {
            self.special_consume("Word_NO");
            self.special_consume("Word_WAI");

            if !self.check_ending() {
                self.create_error(ParserError {
                    message: "Expected newline or comma to end else statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            while !self.is_at_end() {
                let statement = self.parse_statement();
                if let None = statement {
                    self.create_error(ParserError {
                        message: "Expected valid statement for else statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }

                else_statements.push(statement.unwrap());

                if self.special_check("Word_OIC") {
                    break;
                }
            }
        }

        if let None = self.special_consume("Word_OIC") {
            self.create_error(ParserError {
                message: "Expected OIC keyword to end if statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        if else_statements.len() > 0 {
            return Some(ast::IfStatementNode {
                statements,
                else_ifs: else_if_nodes,
                else_: Some(else_statements),
            });
        }
        Some(ast::IfStatementNode {
            statements,
            else_ifs: else_if_nodes,
            else_: None,
        })
    }

    pub fn parse_switch_statement(&mut self) -> Option<ast::SwitchStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_WTF") {
            self.create_error(ParserError {
                message: "Expected WTF keyword to start switch statement",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.consume(tokens::Token::QuestionMark) {
            self.create_error(ParserError {
                message: "Expected ? to start switch statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected newline or comma to end switch statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut cases: Vec<ast::SwitchCaseStatementNode> = Vec::new();

        while !self.is_at_end() {
            if self.special_check("Word_OIC") || self.special_check("Word_OMGWTF") {
                break;
            }

            let statement = self.parse_statement();
            if let Some(s) = statement {
                if cases.len() == 0 {
                    self.create_error(ParserError {
                        message: "Expected OMGWTF keyword to start case statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }

                let last = cases.len() - 1;
                cases[last].statements.push(s);
                continue;
            } else if cases.len() > 0 {
                self.create_error(ParserError {
                    message: "Expected valid statement for case statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            if let None = self.special_consume("Word_OMG") {
                self.create_error(ParserError {
                    message: "Expected OMG keyword to start case statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            } else {
                let expression = self.parse_expression();
                if let None = expression {
                    self.create_error(ParserError {
                        message: "Expected valid expression for case statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }

                cases.push(ast::SwitchCaseStatementNode {
                    expression: expression.unwrap(),
                    statements: Vec::new(),
                });

                if !self.check_ending() {
                    self.create_error(ParserError {
                        message: "Expected newline or comma to end case statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }
            }
        }

        if let None = self.special_consume("Word_OMGWTF") {
            self.create_error(ParserError {
                message: "Expected OMGWTF keyword to start default case statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected newline or comma to end default case statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut default_case = Some(Vec::new());
        while !self.is_at_end() {
            let statement = self.parse_statement();
            if let None = statement {
                self.create_error(ParserError {
                    message: "Expected valid statement for default case statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            default_case.as_mut().unwrap().push(statement.unwrap());

            if self.special_check("Word_OIC") {
                break;
            }
        }

        if let None = self.special_consume("Word_OIC") {
            self.create_error(ParserError {
                message: "Expected OIC keyword to end switch statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::SwitchStatementNode {
            cases,
            default: default_case,
        })
    }

    pub fn parse_loop_statement(&mut self) -> Option<ast::LoopStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_IM") {
            self.create_error(ParserError {
                message: "Expected IM keyword to start loop statement",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_IN") {
            self.create_error(ParserError {
                message: "Expected IN keyword to start loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let label = self.special_consume("Identifier");
        if let None = label {
            self.create_error(ParserError {
                message: "Expected identifier for loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_UPPIN") {
            if let None = self.special_consume("Word_NERFIN") {
                self.create_error(ParserError {
                    message: "Expected UPPIN or NERFIN keyword to start loop statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
        }
        let operation = self.previous();

        if let None = self.special_consume("Word_YR") {
            self.create_error(ParserError {
                message: "Expected YR keyword to start loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let variable = self.special_consume("Identifier");
        if let None = variable {
            self.create_error(ParserError {
                message: "Expected identifier for loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut condition = None;
        let mut condition_expression = None;
        if let None = self.special_consume("Word_TIL") {
            if let Some(t) = self.special_consume("Word_WILE") {
                condition = Some(t);

                condition_expression = self.parse_expression();
                if let None = condition_expression {
                    self.create_error(ParserError {
                        message: "Expected valid expression for loop statement",
                        token: self.peek(),
                    });
                    self.reset(start);
                    return None;
                }
            }
        } else {
            condition = Some(ast::TokenNode {
                token: self.previous(),
            });

            condition_expression = self.parse_expression();
            if let None = condition_expression {
                self.create_error(ParserError {
                    message: "Expected valid expression for loop statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected newline or comma to end loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.special_check("Word_IM")
                && self.special_check_amount("Word_OUTTA", 1)
                && self.special_check_amount("Word_YR", 2)
                && self.special_check_amount("Identifier", 3)
            {
                break;
            }

            let statement = self.parse_statement();
            if let None = statement {
                self.create_error(ParserError {
                    message: "Expected valid statement for loop statement",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            statements.push(statement.unwrap());
        }

        if let None = self.special_consume("Word_IM") {
            self.create_error(ParserError {
                message: "Expected IM keyword to end loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_OUTTA") {
            self.create_error(ParserError {
                message: "Expected OUTTA keyword to end loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_YR") {
            self.create_error(ParserError {
                message: "Expected YR keyword to end loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let outta_label = self.special_consume("Identifier");
        if let None = outta_label {
            self.create_error(ParserError {
                message: "Expected identifier to end loop statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        match label.clone().unwrap().token.token {
            tokens::Token::Identifier(label) => match outta_label.unwrap().token.token {
                tokens::Token::Identifier(outta_label) => {
                    if label != outta_label {
                        self.create_error(ParserError {
                            message: "Expected same label to end loop statement",
                            token: self.peek(),
                        });
                        self.reset(start);
                        return None;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        self.prev_level();
        Some(ast::LoopStatementNode {
            label: label.unwrap(),
            operation: ast::TokenNode { token: operation },
            variable: variable.unwrap(),
            condition,
            condition_expression,
            statements,
        })
    }

    pub fn parse_return_statement(&mut self) -> Option<ast::ReturnStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_FOUND") {
            self.create_error(ParserError {
                message: "Expected FOUND keyword to start return statement",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_YR") {
            self.create_error(ParserError {
                message: "Expected YR keyword to start return statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let expression = self.parse_expression();
        if let None = expression {
            self.create_error(ParserError {
                message: "Expected valid expression for return statement",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::ReturnStatementNode {
            expression: expression.unwrap(),
        })
    }

    pub fn parse_function_definition_statement(
        &mut self,
    ) -> Option<ast::FunctionDefinitionStatementNode> {
        self.next_level();
        let start = self.current;

        if let None = self.special_consume("Word_HOW") {
            self.create_error(ParserError {
                message: "Expected HOW keyword to start function definition",
                token: self.peek(),
            });
            return None;
        }

        if let None = self.special_consume("Word_IZ") {
            self.create_error(ParserError {
                message: "Expected IZ keyword to start function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_I") {
            self.create_error(ParserError {
                message: "Expected I keyword to start function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let identifier = self.special_consume("Identifier");
        if let None = identifier {
            self.create_error(ParserError {
                message: "Expected identifier for function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_ITZ") {
            self.create_error(ParserError {
                message: "Expected ITZ keyword to start function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let return_type: ast::TokenNode;
        if let Some(type_) = self.special_consume("Word_NUMBER") {
            return_type = type_;
        } else if let Some(type_) = self.special_consume("Word_NUMBAR") {
            return_type = type_;
        } else if let Some(type_) = self.special_consume("Word_YARN") {
            return_type = type_;
        } else if let Some(type_) = self.special_consume("Word_TROOF") {
            return_type = type_;
        } else if let Some(type_) = self.special_consume("Word_NOOB") {
            return_type = type_;
        } else {
            self.create_error(ParserError {
                message: "Expected valid return type for function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut arguments = Vec::new();
        while !self.is_at_end() {
            if let None = self.special_consume("Word_YR") {
                self.create_error(ParserError {
                    message: "Expected YR keyword for function definition",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            let identifier = self.special_consume("Identifier");
            if let None = identifier {
                self.create_error(ParserError {
                    message: "Expected identifier for function definition",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            if let None = self.special_consume("Word_ITZ") {
                self.create_error(ParserError {
                    message: "Expected ITZ keyword to start function definition",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            let type_: ast::TokenNode;
            if let Some(type__) = self.special_consume("Word_NUMBER") {
                type_ = type__;
            } else if let Some(type__) = self.special_consume("Word_NUMBAR") {
                type_ = type__;
            } else if let Some(type__) = self.special_consume("Word_YARN") {
                type_ = type__;
            } else if let Some(type__) = self.special_consume("Word_TROOF") {
                type_ = type__;
            } else {
                self.create_error(ParserError {
                    message: "Expected valid type for function definition",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            arguments.push((identifier.unwrap(), type_));

            if self.special_check("Word_AN") {
                self.special_consume("Word_AN");
            } else {
                break;
            }
        }

        if !self.check_ending() {
            self.create_error(ParserError {
                message: "Expected newline or comma to end function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.special_check("Word_IF")
                && self.special_check_amount("Word_U", 1)
                && self.special_check_amount("Word_SAY", 2)
                && self.special_check_amount("Word_SO", 3)
            {
                break;
            }

            let statement = self.parse_statement();
            if let None = statement {
                self.create_error(ParserError {
                    message: "Expected valid statement for function definition",
                    token: self.peek(),
                });
                self.reset(start);
                return None;
            }

            statements.push(statement.unwrap());
        }

        if let None = self.special_consume("Word_IF") {
            self.create_error(ParserError {
                message: "Expected IF keyword to end function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_U") {
            self.create_error(ParserError {
                message: "Expected U keyword to end function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_SAY") {
            self.create_error(ParserError {
                message: "Expected SAY keyword to end function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        if let None = self.special_consume("Word_SO") {
            self.create_error(ParserError {
                message: "Expected SO keyword to end function definition",
                token: self.peek(),
            });
            self.reset(start);
            return None;
        }

        self.prev_level();
        Some(ast::FunctionDefinitionStatementNode {
            arguments,
            identifier: identifier.unwrap(),
            return_type,
            statements,
        })
    }
}
