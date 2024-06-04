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
}

impl VariableTypes {
    pub fn to_string(&self) -> String {
        match self {
            VariableTypes::Number => "NUMBER".to_string(),
            VariableTypes::Numbar => "NUMBAR".to_string(),
            VariableTypes::Yarn => "YARN".to_string(),
            VariableTypes::Troof => "TROOF".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ScopeState {
    pub variables: i32,
    pub variable_map: HashMap<String, VariableTypes>,
    pub variable_addresses: HashMap<String, i32>,
    pub parent: Option<Box<ScopeState>>,
    pub arguments: i32,
    pub argument_map: HashMap<String, VariableTypes>,
    pub argument_addresses: HashMap<String, i32>,
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
}

pub struct ProgramState {
    pub is_inside_entry: bool,
    pub function_name: String,
    pub entry_function_state: ScopeState,
    pub function_states: Vec<ScopeState>,
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
                    parent: None,
                    arguments: 0,
                    argument_map: HashMap::new(),
                    argument_addresses: HashMap::new(),
                    sub_scopes: vec![],
                },
                function_states: vec![],
            },
        };

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_NUMBER".to_string(), VariableTypes::Number);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_NUMBER".to_string(), 1);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_NUMBAR".to_string(), VariableTypes::Number);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_NUMBAR".to_string(), 2);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_YARN".to_string(), VariableTypes::Number);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_YARN".to_string(), 3);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_TROOF".to_string(), VariableTypes::Number);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_TROOF".to_string(), 4);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
    }

    pub fn find_function_index_by_name(&self, name: String) -> Option<usize> {
        self.ir.functions.iter().position(|f| f.name == name)
    }

    pub fn add_statements(&mut self, statements: Vec<ir::IRStatement>) {
        if self.program_state.is_inside_entry {
            self.ir.entry.statements.extend(statements);
        } else {
            let function = self
                .ir
                .functions
                .iter_mut()
                .find(|f| f.name == self.program_state.function_name)
                .unwrap();
            function.statements.extend(statements);
        }
    }
}

impl<'a> Visitor<'a> {
    pub fn visit(&mut self) -> (ir::IR, Vec<VisitorError>) {
        self.visit_program(self.ast_tree.ast.clone());

        (self.ir.clone(), self.errors.clone())
    }

    pub fn visit_program(&mut self, program: ast::ProgramNode) {
        for statement in &program.statements {
            self.visit_statement(statement.clone());
        }
    }

    pub fn visit_statement(&mut self, statement: ast::StatementNode) {
        match statement.value {
            ast::StatementNodeValueOption::VariableDeclarationStatement(
                variable_declaration_statement,
            ) => {
                self.visit_variable_declaration_statement(variable_declaration_statement.clone());
            }
            ast::StatementNodeValueOption::VariableAssignmentStatement(
                variable_assignment_statement,
            ) => {
                self.visit_variable_assignment_statement(variable_assignment_statement.clone());
            }
            _ => {}
        }
    }

    pub fn visit_expression(
        &mut self,
        expression: ast::ExpressionNode,
        expected_type: VariableTypes,
    ) {
        match expression.value {
            ast::ExpressionNodeValueOption::NumberValue(number_value) => {
                self.visit_number_value(number_value.clone());
                if expected_type != VariableTypes::Number {
                    self.errors.push(VisitorError {
                        message: format!("Expected type {}, got NUMBER", expected_type.to_string()),
                        token: number_value.token.clone(),
                    });
                }
            }
            ast::ExpressionNodeValueOption::NumbarValue(numbar_value) => {
                self.visit_numbar_value(numbar_value.clone());
                if expected_type != VariableTypes::Numbar {
                    self.errors.push(VisitorError {
                        message: format!("Expected type {}, got NUMBAR", expected_type.to_string()),
                        token: numbar_value.token.clone(),
                    });
                }
            }
            ast::ExpressionNodeValueOption::YarnValue(yarn_value) => {
                self.visit_yarn_value(yarn_value.clone());
                if expected_type != VariableTypes::Yarn {
                    self.errors.push(VisitorError {
                        message: format!("Expected type {}, got YARN", expected_type.to_string()),
                        token: yarn_value.token.clone(),
                    });
                }
            }
            ast::ExpressionNodeValueOption::TroofValue(troof_value) => {
                self.visit_troof_value(troof_value.clone());
                if expected_type != VariableTypes::Troof {
                    self.errors.push(VisitorError {
                        message: format!("Expected type {}, got TROOF", expected_type.to_string()),
                        token: troof_value.token.clone(),
                    });
                }
            }
            ast::ExpressionNodeValueOption::VariableReference(variable_reference) => {
                let return_type = self.visit_variable_reference(variable_reference.clone());
                if return_type != expected_type {
                    self.errors.push(VisitorError {
                        message: format!(
                            "Expected type {}, got {}",
                            expected_type.to_string(),
                            return_type.to_string()
                        ),
                        token: variable_reference.identifier.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    pub fn visit_number_value(&mut self, number_value: ast::NumberValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(number_value.value() as f32)]);
    }

    pub fn visit_numbar_value(&mut self, numbar_value: ast::NumbarValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(numbar_value.value())]);
    }

    pub fn visit_yarn_value(&mut self, yarn_value: ast::YarnValueNode) {
        for char in yarn_value.value().chars() {
            self.add_statements(vec![ir::IRStatement::Push(char as i32 as f32)]);
        }

        let last_address = -(self.program_state.entry_function_state.variables
            + 1
            + yarn_value.value().len() as i32
            + 1); // 1 for the next address, len for the length of the string, 1 for the stored length

        self.add_statements(vec![
            ir::IRStatement::Push((4 * yarn_value.value().len()) as f32),
            ir::IRStatement::Push((4 * yarn_value.value().len()) as f32 + 1.0),
            ir::IRStatement::Allocate,
            ir::IRStatement::Push(last_address as f32),
            ir::IRStatement::Copy,
            ir::IRStatement::Store(yarn_value.value().len() as i32 + 1),
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
        variable_reference: ast::VariableReferenceNode,
    ) -> VariableTypes {
        let name = match variable_reference.identifier.value() {
            tokens::Token::Identifier(name) => name,
            _ => panic!("Unexpected token"),
        }
        .clone();

        if self.program_state.is_inside_entry {
            if let None = self
                .program_state
                .entry_function_state
                .get_variable(name.clone())
            {
                self.errors.push(VisitorError {
                    message: format!("Variable {} not declared", name),
                    token: variable_reference.identifier.clone(),
                });
                return VariableTypes::Number;
            }

            let scope = self
                .program_state
                .entry_function_state
                .get_variable(name.clone())
                .unwrap();

            let address = *scope.variable_addresses.get(&name).unwrap() as f32;

            self.add_statements(vec![ir::IRStatement::Push(address), ir::IRStatement::Copy]);
            return scope.variable_map.get(&name).unwrap().clone();
        } else {
            let index = self
                .find_function_index_by_name(self.program_state.function_name.clone())
                .unwrap();
            let function = &mut self.program_state.function_states[index];
            let clone = function.clone();

            if let None = clone.get_variable(name.clone()) {
                self.errors.push(VisitorError {
                    message: format!("Variable {} not declared", name),
                    token: variable_reference.identifier.clone(),
                });
                return VariableTypes::Number;
            }

            let scope = clone.get_variable(name.clone()).unwrap();

            let address = *scope.variable_addresses.get(&name).unwrap() as f32;

            self.add_statements(vec![ir::IRStatement::Push(address), ir::IRStatement::Copy]);
            return scope.variable_map.get(&name).unwrap().clone();
        }
    }

    pub fn visit_variable_declaration_statement(
        &mut self,
        variable_declaration_statement: ast::VariableDeclarationStatementNode,
    ) {
        let name = match variable_declaration_statement.identifier.value() {
            tokens::Token::Identifier(name) => name,
            _ => panic!("Unexpected token"),
        }
        .clone();

        let variable_type = match variable_declaration_statement.type_.value() {
            tokens::Token::Word(word) => match word.as_str() {
                "NUMBER" => VariableTypes::Number,
                "NUMBAR" => VariableTypes::Numbar,
                "YARN" => VariableTypes::Yarn,
                "TROOF" => VariableTypes::Troof,
                _ => panic!("Unexpected variable type"),
            },
            _ => panic!("Unexpected token"),
        };

        if self.program_state.is_inside_entry {
            if self
                .program_state
                .entry_function_state
                .variable_map
                .contains_key(&name)
                || self
                    .program_state
                    .entry_function_state
                    .argument_map
                    .contains_key(&name)
            {
                self.errors.push(VisitorError {
                    message: format!("Variable {} already declared", name),
                    token: variable_declaration_statement.identifier.clone(),
                });
                return;
            }
            self.program_state.entry_function_state.variables += 1;
            self.program_state
                .entry_function_state
                .variable_map
                .insert(name.clone(), variable_type);
            self.program_state
                .entry_function_state
                .variable_addresses
                .insert(name, -self.program_state.entry_function_state.variables);
            self.add_statements(vec![ir::IRStatement::Push(0.0)]);
        } else {
            let index = self
                .find_function_index_by_name(self.program_state.function_name.clone())
                .unwrap();
            let function = &mut self.program_state.function_states[index];
            if function.variable_map.contains_key(&name)
                || function.argument_map.contains_key(&name)
            {
                self.errors.push(VisitorError {
                    message: format!("Variable {} already declared", name),
                    token: variable_declaration_statement.identifier.clone(),
                });
                return;
            }
            function.variables += 1;
            function.variable_map.insert(name.clone(), variable_type);
            function
                .variable_addresses
                .insert(name, -function.variables);
            self.add_statements(vec![ir::IRStatement::Push(0.0)]);
        }
    }

    pub fn visit_variable_assignment_statement(
        &mut self,
        variable_assignment_statement: ast::VariableAssignmentStatementNode,
    ) {
        if let ast::VariableAssignmentNodeVariableOption::Identifier(ident) =
            &variable_assignment_statement.variable
        {
            let name = match ident.value() {
                tokens::Token::Identifier(name) => name,
                _ => panic!("Unexpected token"),
            }
            .clone();

            if self.program_state.is_inside_entry {
                if let None = self
                    .program_state
                    .entry_function_state
                    .get_variable(name.clone())
                {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token: ident.clone(),
                    });
                    return;
                }

                let scope = self
                    .program_state
                    .entry_function_state
                    .get_variable(name.clone())
                    .unwrap();

                self.visit_expression(
                    variable_assignment_statement.expression.clone(),
                    scope.variable_map.get(&name).unwrap().clone(),
                );
                self.add_statements(vec![
                    ir::IRStatement::Push(
                        *self
                            .program_state
                            .entry_function_state
                            .variable_addresses
                            .get(&name)
                            .unwrap() as f32,
                    ),
                    ir::IRStatement::Mov,
                ]);
            } else {
                let index = self
                    .find_function_index_by_name(self.program_state.function_name.clone())
                    .unwrap();
                let function_states = &mut self.program_state.function_states;
                let function = &mut function_states[index];
                let clone = function.clone();
                if let None = clone.get_variable(name.clone()) {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token: ident.clone(),
                    });
                    return;
                }

                let scope = clone.get_variable(name.clone()).unwrap();

                let address = *scope.variable_addresses.get(&name).unwrap() as f32;

                self.visit_expression(
                    variable_assignment_statement.expression.clone(),
                    clone.variable_map.get(&name).unwrap().clone(),
                );
                self.add_statements(vec![ir::IRStatement::Push(address), ir::IRStatement::Mov]);
            }
        }

        if let ast::VariableAssignmentNodeVariableOption::VariableDeclerationStatement(var_dec) =
            variable_assignment_statement.variable
        {
            self.visit_variable_declaration_statement(var_dec.clone());

            let name = match var_dec.identifier.value() {
                tokens::Token::Identifier(name) => name,
                _ => panic!("Unexpected token"),
            }
            .clone();

            if self.program_state.is_inside_entry {
                self.visit_expression(
                    variable_assignment_statement.expression.clone(),
                    self.program_state
                        .entry_function_state
                        .variable_map
                        .get(&name)
                        .unwrap()
                        .clone(),
                );
                self.add_statements(vec![
                    ir::IRStatement::Push(
                        *self
                            .program_state
                            .entry_function_state
                            .variable_addresses
                            .get(&name)
                            .unwrap() as f32,
                    ),
                    ir::IRStatement::Mov,
                ]);
            } else {
                let index = self
                    .find_function_index_by_name(self.program_state.function_name.clone())
                    .unwrap();
                let function = &mut self.program_state.function_states[index];
                let clone = function.clone();

                let address = *function.variable_addresses.get(&name).unwrap() as f32;

                self.visit_expression(
                    variable_assignment_statement.expression.clone(),
                    clone.variable_map.get(&name).unwrap().clone(),
                );
                self.add_statements(vec![ir::IRStatement::Push(address), ir::IRStatement::Mov]);
            }
        }
    }
}
