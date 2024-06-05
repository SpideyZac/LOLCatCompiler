use std::collections::HashMap;

use crate::compiler::ir;
use crate::lexer::tokens;
use crate::parser::ast;
use crate::parser::parser;

#[derive(PartialEq, Clone)]
pub enum VariableTypes {
    Number,
    Numbar,
    Yarn,
    Troof,
    Noob,
}

impl VariableTypes {
    pub fn to_string(&self) -> String {
        match self {
            VariableTypes::Number => "NUMBER".to_string(),
            VariableTypes::Numbar => "NUMBAR".to_string(),
            VariableTypes::Yarn => "YARN".to_string(),
            VariableTypes::Troof => "TROOF".to_string(),
            VariableTypes::Noob => "NOOB".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ScopeState {
    pub variables: i32,
    pub variable_map: HashMap<String, VariableTypes>,
    pub variable_addresses: HashMap<String, i32>,
    pub is_argument: HashMap<String, bool>,
    pub arguments: i32,
    pub parent: Option<Box<ScopeState>>,
    pub sub_scopes: Vec<ScopeState>,
}

impl ScopeState {
    pub fn get_variable(&self, name: String) -> Option<ScopeState> {
        if self.variable_map.contains_key(&name) {
            return Some(self.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_variable(name);
        }

        None
    }

    pub fn get_variable_mut(&mut self, name: String) -> Option<&mut ScopeState> {
        if self.variable_map.contains_key(&name) {
            return Some(self);
        }

        if let Some(parent) = &mut self.parent {
            return parent.get_variable_mut(name);
        }

        None
    }
}

pub struct ProgramState {
    pub is_inside_entry: bool,
    pub function_name: String,
    pub entry_function_state: ScopeState,
    pub function_states: HashMap<String, ScopeState>,
}

impl ProgramState {
    pub fn get_scope(&self) -> ScopeState {
        if self.is_inside_entry {
            return self.entry_function_state.clone();
        }

        self.function_states
            .get(&self.function_name)
            .unwrap()
            .clone()
    }

    pub fn get_mut_scope(&mut self) -> &mut ScopeState {
        if self.is_inside_entry {
            return &mut self.entry_function_state;
        }

        self.function_states.get_mut(&self.function_name).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct VisitorError {
    pub message: String,
    pub token: ast::TokenNode,
}

pub struct Visitor<'a> {
    ast_tree: parser::ParserReturn<'a>,
    ir: ir::IR,
    errors: Vec<VisitorError>,
    program_state: ProgramState,
}

impl<'a> Visitor<'a> {
    pub fn add_statements(&mut self, statements: Vec<ir::IRStatement>) {
        if self.program_state.is_inside_entry {
            self.ir.entry.statements.extend(statements);
        } else {
            self.ir
                .functions
                .iter_mut()
                .find(|f| f.name == self.program_state.function_name)
                .unwrap()
                .statements
                .extend(statements);
        }
    }

    pub fn new(ast_tree: parser::ParserReturn<'a>, stack_size: i32, heap_size: i32) -> Self {
        let entry = ir::IRFunctionEntry::new(stack_size, heap_size, vec![]);
        let mut visitor = Self {
            ast_tree,
            ir: ir::IR::new(vec![], entry),
            errors: vec![],
            program_state: ProgramState {
                is_inside_entry: true,
                function_name: "".to_string(),
                entry_function_state: ScopeState {
                    variables: 0,
                    variable_map: HashMap::new(),
                    variable_addresses: HashMap::new(),
                    is_argument: HashMap::new(),
                    arguments: 0,
                    parent: None,
                    sub_scopes: vec![],
                },
                function_states: HashMap::new(),
            },
        };

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT".to_string(), VariableTypes::Noob);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT".to_string(), -1);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
    }
}

impl<'a> Visitor<'a> {
    pub fn visit(&mut self) -> (ir::IR, Vec<VisitorError>) {
        self.visit_program(self.ast_tree.ast.clone());

        (self.ir.clone(), self.errors.clone())
    }

    pub fn visit_program(&mut self, program: ast::ProgramNode) {
        for statement in program.statements {
            self.visit_statement(statement.clone());
        }
    }

    pub fn visit_statement(&mut self, statement: ast::StatementNode) {
        match statement.value {
            ast::StatementNodeValueOption::Expression(expr) => {
                let (type_, _) = self.visit_expression(expr.clone());
                // save to IT with type_
                if type_ != VariableTypes::Noob {
                    let scope = self.program_state.get_scope();

                    if *scope.variable_map.get("IT").unwrap() == VariableTypes::Yarn {
                        self.add_statements(vec![
                            ir::IRStatement::Push(
                                *scope.variable_addresses.get("IT").unwrap() as f32,
                            ),
                            ir::IRStatement::Copy,
                            ir::IRStatement::Load(1), // get the size
                            ir::IRStatement::Push(1.0),
                            ir::IRStatement::Add,
                            ir::IRStatement::Push(4.0),
                            ir::IRStatement::Multiply,
                            ir::IRStatement::Push(
                                *scope.variable_addresses.get("IT").unwrap() as f32,
                            ),
                            ir::IRStatement::Copy,
                            ir::IRStatement::Free,
                        ]);
                    }

                    match type_ {
                        VariableTypes::Number => {
                            self.add_statements(vec![
                                ir::IRStatement::Push(
                                    *scope.variable_addresses.get("IT").unwrap() as f32
                                ),
                                ir::IRStatement::Mov,
                            ]);
                            let mut_scope = self.program_state.get_mut_scope();
                            mut_scope
                                .variable_map
                                .get_mut("IT")
                                .unwrap()
                                .clone_from(&VariableTypes::Number);
                        }
                        VariableTypes::Numbar => {
                            self.add_statements(vec![
                                ir::IRStatement::Push(
                                    *scope.variable_addresses.get("IT").unwrap() as f32
                                ),
                                ir::IRStatement::Mov,
                            ]);
                            let mut_scope = self.program_state.get_mut_scope();
                            mut_scope
                                .variable_map
                                .get_mut("IT")
                                .unwrap()
                                .clone_from(&VariableTypes::Numbar);
                        }
                        VariableTypes::Yarn => {
                            self.add_statements(vec![
                                ir::IRStatement::Push(
                                    *scope.variable_addresses.get("IT").unwrap() as f32
                                ),
                                ir::IRStatement::Mov,
                            ]);
                            let mut_scope = self.program_state.get_mut_scope();
                            mut_scope
                                .variable_map
                                .get_mut("IT")
                                .unwrap()
                                .clone_from(&VariableTypes::Yarn);
                        }
                        VariableTypes::Troof => {
                            self.add_statements(vec![
                                ir::IRStatement::Push(
                                    *scope.variable_addresses.get("IT").unwrap() as f32
                                ),
                                ir::IRStatement::Mov,
                            ]);
                            let mut_scope = self.program_state.get_mut_scope();
                            mut_scope
                                .variable_map
                                .get_mut("IT")
                                .unwrap()
                                .clone_from(&VariableTypes::Troof);
                        }
                        _ => {
                            panic!("Unexpected type")
                        }
                    }
                }
            }
            ast::StatementNodeValueOption::VariableDeclarationStatement(var_decl) => {
                self.visit_variable_declaration(var_decl.clone());
            }
            ast::StatementNodeValueOption::VariableAssignmentStatement(var_assign) => {
                self.visit_variable_assignment(var_assign.clone());
            }
            ast::StatementNodeValueOption::KTHXBYEStatement(_) => {
                self.add_statements(vec![ir::IRStatement::Halt]);
            }
            _ => {
                panic!("Unexpected statement")
            }
        }
    }

    pub fn visit_expression(
        &mut self,
        expression: ast::ExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        match expression.value {
            ast::ExpressionNodeValueOption::NumberValue(number_value) => {
                self.visit_number_value(number_value.clone());
                (VariableTypes::Number, number_value.token.clone())
            }
            ast::ExpressionNodeValueOption::NumbarValue(numbar_value) => {
                self.visit_numbar_value(numbar_value.clone());
                (VariableTypes::Numbar, numbar_value.token.clone())
            }
            ast::ExpressionNodeValueOption::YarnValue(yarn_value) => {
                self.visit_yarn_value(yarn_value.clone());
                (VariableTypes::Yarn, yarn_value.token.clone())
            }
            ast::ExpressionNodeValueOption::TroofValue(troof_value) => {
                self.visit_troof_value(troof_value.clone());
                (VariableTypes::Troof, troof_value.token.clone())
            }
            ast::ExpressionNodeValueOption::VariableReference(variable_ref) => {
                let type_ = self.visit_variable_reference(variable_ref.clone());
                (type_, variable_ref.identifier.clone())
            }
            ast::ExpressionNodeValueOption::SumExpression(sum_expr) => {
                let (type_, token) = self.visit_sum_expression(sum_expr.clone());
                (type_, token)
            }
            ast::ExpressionNodeValueOption::DiffExpression(diff_expr) => {
                let (type_, token) = self.visit_difference_expression(diff_expr.clone());
                (type_, token)
            }
            ast::ExpressionNodeValueOption::ProduktExpression(prod_expr) => {
                let (type_, token) = self.visit_product_expression(prod_expr.clone());
                (type_, token)
            }
            ast::ExpressionNodeValueOption::QuoshuntExpression(quoshunt_expr) => {
                let (type_, token) = self.visit_quotient_expression(quoshunt_expr.clone());
                (type_, token)
            }
            ast::ExpressionNodeValueOption::ModExpression(mod_expr) => {
                let (type_, token) = self.visit_mod_expression(mod_expr.clone());
                (type_, token)
            }
            _ => {
                panic!("Unexpected expression")
            }
        }
    }

    pub fn visit_number_value(&mut self, number_value: ast::NumberValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(number_value.value() as f32)]);
    }

    pub fn visit_numbar_value(&mut self, numbar_value: ast::NumbarValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(numbar_value.value())]);
    }

    pub fn visit_yarn_value(&mut self, yarn_value: ast::YarnValueNode) {
        // yarn stores a pointer to the string on the heap
        let chars = yarn_value.value().chars().collect::<Vec<char>>();
        self.add_statements(vec![
            ir::IRStatement::Push((chars.len() as i32 as f32 + 1.0) * 4.0), // store length + 1
            ir::IRStatement::Allocate, // allocate space on the heap
        ]);
        self.add_statements(vec![ir::IRStatement::Push(chars.len() as i32 as f32)]); // store length
        for char in chars.iter() {
            self.add_statements(vec![ir::IRStatement::Push(*char as i32 as f32)]);
            // store char
        }
        self.add_statements(vec![
            ir::IRStatement::Push(
                -(self.program_state.get_scope().variables as f32
                    - self.program_state.get_scope().arguments as f32
                    + 1.0),
            ), // This is the address of the heap_ptr for the string
            ir::IRStatement::Copy, // duplicate this value
            ir::IRStatement::Store(chars.len() as i32 + 1), // store the string at the address
        ]);
    }

    pub fn visit_troof_value(&mut self, troof_value: ast::TroofValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(if troof_value.value() {
            1.0
        } else {
            0.0
        })]);
    }

    pub fn visit_variable_reference(
        &mut self,
        variable_ref: ast::VariableReferenceNode,
    ) -> VariableTypes {
        let scope = self.program_state.get_scope();
        let identifier = match &variable_ref.identifier.token.token {
            tokens::Token::Identifier(ident) => ident.clone(),
            _ => {
                panic!("Expected identifier token");
            }
        };

        let sub_scope = scope.get_variable(identifier.clone());
        if let None = sub_scope {
            self.errors.push(VisitorError {
                message: format!("Variable {} not declared", identifier),
                token: variable_ref.identifier.clone(),
            });
            return VariableTypes::Noob;
        }

        let sub_scope = sub_scope.unwrap();
        let type_ = sub_scope.variable_map.get(&identifier).unwrap().clone();
        let address = sub_scope
            .variable_addresses
            .get(&identifier)
            .unwrap()
            .clone();
        self.add_statements(vec![
            ir::IRStatement::Push(address as f32),
            ir::IRStatement::Copy,
        ]);

        type_
    }

    pub fn visit_sum_expression(
        &mut self,
        sum_expr: ast::SumExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        let (left_type, left_token) = self.visit_expression(*sum_expr.left.clone());
        if left_type != VariableTypes::Number && left_type != VariableTypes::Numbar {
            self.errors.push(VisitorError {
                message: format!("Expected NUMBER or NUMBAR, got {}", left_type.to_string()),
                token: left_token.clone(),
            });
            return (VariableTypes::Noob, left_token);
        }
        let (right_type, right_token) = self.visit_expression(*sum_expr.right.clone());
        if right_type != left_type {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} got {}",
                    left_type.to_string(),
                    right_type.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableTypes::Noob, right_token);
        }
        self.add_statements(vec![ir::IRStatement::Add]);
        (left_type, left_token)
    }

    pub fn visit_difference_expression(
        &mut self,
        diff_expr: ast::DiffExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        let (left_type, left_token) = self.visit_expression(*diff_expr.left.clone());
        if left_type != VariableTypes::Number && left_type != VariableTypes::Numbar {
            self.errors.push(VisitorError {
                message: format!("Expected NUMBER or NUMBAR, got {}", left_type.to_string()),
                token: left_token.clone(),
            });
            return (VariableTypes::Noob, left_token);
        }
        let (right_type, right_token) = self.visit_expression(*diff_expr.right.clone());
        if right_type != left_type {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} got {}",
                    left_type.to_string(),
                    right_type.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableTypes::Noob, right_token);
        }
        self.add_statements(vec![ir::IRStatement::Subtract]);
        (left_type, left_token)
    }

    pub fn visit_product_expression(
        &mut self,
        prod_expr: ast::ProduktExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        let (left_type, left_token) = self.visit_expression(*prod_expr.left.clone());
        if left_type != VariableTypes::Number && left_type != VariableTypes::Numbar {
            self.errors.push(VisitorError {
                message: format!("Expected NUMBER or NUMBAR, got {}", left_type.to_string()),
                token: left_token.clone(),
            });
            return (VariableTypes::Noob, left_token);
        }
        let (right_type, right_token) = self.visit_expression(*prod_expr.right.clone());
        if right_type != left_type {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} got {}",
                    left_type.to_string(),
                    right_type.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableTypes::Noob, right_token);
        }
        self.add_statements(vec![ir::IRStatement::Multiply]);
        (left_type, left_token)
    }

    pub fn visit_quotient_expression(
        &mut self,
        quoshunt_expr: ast::QuoshuntExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        let (left_type, left_token) = self.visit_expression(*quoshunt_expr.left.clone());
        if left_type != VariableTypes::Number && left_type != VariableTypes::Numbar {
            self.errors.push(VisitorError {
                message: format!("Expected NUMBER or NUMBAR, got {}", left_type.to_string()),
                token: left_token.clone(),
            });
            return (VariableTypes::Noob, left_token);
        }
        let (right_type, right_token) = self.visit_expression(*quoshunt_expr.right.clone());
        if right_type != left_type {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} got {}",
                    left_type.to_string(),
                    right_type.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableTypes::Noob, right_token);
        }
        self.add_statements(vec![ir::IRStatement::Divide]);
        (left_type, left_token)
    }

    pub fn visit_mod_expression(
        &mut self,
        mod_expr: ast::ModExpressionNode,
    ) -> (VariableTypes, ast::TokenNode) {
        let (left_type, left_token) = self.visit_expression(*mod_expr.left.clone());
        if left_type != VariableTypes::Number {
            self.errors.push(VisitorError {
                message: format!("Expected NUMBER, got {}", left_type.to_string()),
                token: left_token.clone(),
            });
            return (VariableTypes::Noob, left_token);
        }
        let (right_type, right_token) = self.visit_expression(*mod_expr.right.clone());
        if right_type != left_type {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} got {}",
                    left_type.to_string(),
                    right_type.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableTypes::Noob, right_token);
        }
        self.add_statements(vec![ir::IRStatement::Modulo]);
        (left_type, left_token)
    }

    pub fn visit_variable_declaration(&mut self, var_decl: ast::VariableDeclarationStatementNode) {
        let scope = self.program_state.get_mut_scope();
        let identifier = match &var_decl.identifier.token.token {
            tokens::Token::Identifier(ident) => ident.clone(),
            _ => {
                panic!("Expected identifier token");
            }
        };
        let type_ = match var_decl.type_.token.token.to_name().as_str() {
            "Word_NUMBER" => VariableTypes::Number,
            "Word_NUMBAR" => VariableTypes::Numbar,
            "Word_YARN" => VariableTypes::Yarn,
            "Word_TROOF" => VariableTypes::Troof,
            _ => {
                panic!("Unexpected type");
            }
        };

        if scope.variable_map.contains_key(&identifier) {
            self.errors.push(VisitorError {
                message: format!("Variable {} already declared", identifier),
                token: var_decl.identifier.clone(),
            });
            return;
        }

        scope.variables += 1;
        scope.variable_map.insert(identifier.clone(), type_);
        scope
            .variable_addresses
            .insert(identifier.clone(), -(scope.variables - scope.arguments));
        self.add_statements(vec![ir::IRStatement::Push(0.0)]);
    }

    pub fn visit_variable_assignment(&mut self, var_assign: ast::VariableAssignmentStatementNode) {
        if let ast::VariableAssignmentNodeVariableOption::Identifier(ident) = var_assign.variable {
            let mut scope = self.program_state.get_scope();
            let identifier = match &ident.token.token {
                tokens::Token::Identifier(identi) => identi.clone(),
                _ => {
                    panic!("Expected identifier token");
                }
            };

            let sub_scope = scope.get_variable_mut(identifier.clone());
            if let None = sub_scope {
                self.errors.push(VisitorError {
                    message: format!("Variable {} not declared", identifier),
                    token: ident.clone(),
                });
                return;
            }

            let sub_scope = sub_scope.unwrap();
            let type_ = sub_scope.variable_map.get(&identifier).unwrap().clone();

            let (expr_type, token) = self.visit_expression(var_assign.expression.clone());
            if type_ != expr_type {
                self.errors.push(VisitorError {
                    message: format!(
                        "Variable {} is of type {}, but expression is of type {}",
                        identifier,
                        type_.to_string(),
                        expr_type.to_string()
                    ),
                    token,
                });
                return;
            }

            let address = sub_scope
                .variable_addresses
                .get(&identifier)
                .unwrap()
                .clone();
            self.add_statements(vec![
                ir::IRStatement::Push(address as f32),
                ir::IRStatement::Mov,
            ]);
        } else {
            let var_dec = match var_assign.variable {
                ast::VariableAssignmentNodeVariableOption::VariableDeclerationStatement(va_dec) => {
                    va_dec
                }
                _ => {
                    panic!("Expected function definition");
                }
            };
            self.visit_variable_declaration(var_dec.clone());
            let scope = self.program_state.get_scope();

            let identifier = match &var_dec.identifier.token.token {
                tokens::Token::Identifier(ident) => ident.clone(),
                _ => {
                    panic!("Expected identifier token");
                }
            };

            let type_ = match var_dec.type_.token.token.to_name().as_str() {
                "Word_NUMBER" => VariableTypes::Number,
                "Word_NUMBAR" => VariableTypes::Numbar,
                "Word_YARN" => VariableTypes::Yarn,
                "Word_TROOF" => VariableTypes::Troof,
                _ => {
                    panic!("Unexpected type");
                }
            };

            let (expr_type, token) = self.visit_expression(var_assign.expression.clone());
            if type_ != expr_type {
                self.errors.push(VisitorError {
                    message: format!(
                        "Variable {} is of type {}, but expression is of type {}",
                        identifier,
                        type_.to_string(),
                        expr_type.to_string()
                    ),
                    token,
                });
                return;
            }

            let address = scope.variable_addresses.get(&identifier).unwrap().clone();
            self.add_statements(vec![
                ir::IRStatement::Push(address as f32),
                ir::IRStatement::Mov,
            ]);
        }
    }
}
