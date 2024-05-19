use crate::lexer::lexer;
use crate::lexer::tokens;

#[derive(Debug, Clone)]
pub struct TokenNode {
    pub token: lexer::LexedToken,
}

impl TokenNode {
    pub fn value(&self) -> &tokens::Token {
        &self.token.token
    }
}

#[derive(Debug, Clone)]
pub struct ProgramNode {
    pub statements: Vec<StatementNode>,
}

#[derive(Debug, Clone)]
pub enum StatementNodeValueOption {
    Expression(ExpressionNode),
    VariableDeclarationStatement(VariableDeclarationStatementNode),
    VariableAssignmentStatement(VariableAssignmentStatementNode),
    KTHXBYEStatement(TokenNode),
    VisibleStatement(VisibleStatementNode),
    GimmehStatement(GimmehStatementNode),
}

#[derive(Debug, Clone)]
pub struct StatementNode {
    pub value: StatementNodeValueOption,
}

#[derive(Debug, Clone)]
pub enum ExpressionNodeValueOption {
    NumberValue(NumberValueNode),
    NumbarValue(NumbarValueNode),
    YarnValue(YarnValueNode),
    TroofValue(TroofValueNode),
}

#[derive(Debug, Clone)]
pub struct ExpressionNode {
    pub value: ExpressionNodeValueOption,
}

#[derive(Debug, Clone)]
pub struct NumberValueNode {
    pub token: TokenNode,
}

impl NumberValueNode {
    pub fn value(&self) -> i64 {
        if let tokens::Token::NumberValue(value) = self.token.value() {
            value.parse::<i64>().unwrap()
        } else {
            panic!("Expected NumberValue token")
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumbarValueNode {
    pub token: TokenNode,
}

impl NumbarValueNode {
    pub fn value(&self) -> f64 {
        if let tokens::Token::NumbarValue(value) = self.token.value() {
            value.parse::<f64>().unwrap()
        } else {
            panic!("Expected NumbarValue token")
        }
    }
}

#[derive(Debug, Clone)]
pub struct YarnValueNode {
    pub token: TokenNode,
}

impl YarnValueNode {
    pub fn value(&self) -> &String {
        if let tokens::Token::YarnValue(value) = self.token.value() {
            value
        } else {
            panic!("Expected YarnValue token")
        }
    }
}

#[derive(Debug, Clone)]
pub struct TroofValueNode {
    pub token: TokenNode,
}

impl TroofValueNode {
    pub fn value(&self) -> bool {
        if let tokens::Token::TroofValue(value) = self.token.value() {
            match value.as_str() {
                "WIN" => true,
                "FAIL" => false,
                _ => panic!("Invalid TroofValue token"),
            }
        } else {
            panic!("Expected TroofValue token")
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableDeclarationStatementNode {
    pub identifier: TokenNode,
    pub type_: TokenNode,
}

#[derive(Debug, Clone)]
pub enum VariableAssignmentNodeVariableOption {
    Identifier(TokenNode),
    VariableDeclerationStatement(VariableDeclarationStatementNode),
}

#[derive(Debug, Clone)]
pub struct VariableAssignmentStatementNode {
    pub variable: VariableAssignmentNodeVariableOption,
    pub expression: ExpressionNode,
}

#[derive(Debug, Clone)]
pub struct VisibleStatementNode {
    pub expressions: Vec<ExpressionNode>,
    pub exclamation: Option<TokenNode>,
}

#[derive(Debug, Clone)]
pub struct GimmehStatementNode {
    pub identifier: TokenNode,
}
