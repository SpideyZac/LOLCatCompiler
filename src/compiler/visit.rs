use std::collections::HashMap;

use crate::compiler::ir;
use crate::lexer::tokens;
use crate::parser::ast;
use crate::parser::parser;

pub enum VariableTypes {
    Number,
}

pub struct ScopeState {
    pub variables: i32,
    pub variable_map: HashMap<String, VariableTypes>,
    pub variable_addresses: HashMap<String, i32>,
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
        Self {
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
                },
                function_states: vec![],
            },
        }
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
            ast::StatementNodeValueOption::Expression(expression) => {
                self.visit_expression(expression.clone());
                // TODO: setup IT variable from exp
            }
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

    pub fn visit_expression(&mut self, expression: ast::ExpressionNode) {
        match expression.value {
            ast::ExpressionNodeValueOption::NumberValue(number_value) => {
                self.visit_number_value(number_value.clone());
            }
            _ => {}
        }
    }

    pub fn visit_number_value(&mut self, number_value: ast::NumberValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(number_value.value() as f32)]);
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
            if function.variable_map.contains_key(&name) {
                self.errors.push(VisitorError {
                    message: format!("Variable {} already declared", name),
                    token: variable_declaration_statement.identifier.clone(),
                });
                return;
            }
            function.variables += 1;
            function.variable_map.insert(name.clone(), variable_type);
            function.variable_addresses.insert(name, -function.variables);
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
                if !self
                    .program_state
                    .entry_function_state
                    .variable_map
                    .contains_key(&name)
                {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token: ident.clone(),
                    });
                    return;
                }

                self.visit_expression(variable_assignment_statement.expression.clone());
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
                if !function.variable_map.contains_key(&name) {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token: ident.clone(),
                    });
                    return;
                }

                let address = *function.variable_addresses.get(&name).unwrap() as f32;

                self.visit_expression(variable_assignment_statement.expression.clone());
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
                self.add_statements(vec![ir::IRStatement::Push(
                    *self
                        .program_state
                        .entry_function_state
                        .variable_addresses
                        .get(&name)
                        .unwrap() as f32,
                )]);
                self.visit_expression(variable_assignment_statement.expression.clone());
                self.add_statements(vec![ir::IRStatement::Mov]);
            } else {
                let index = self
                    .find_function_index_by_name(self.program_state.function_name.clone())
                    .unwrap();
                let function = &mut self.program_state.function_states[index];

                let address = *function.variable_addresses.get(&name).unwrap() as f32;

                self.visit_expression(variable_assignment_statement.expression.clone());
                self.add_statements(vec![ir::IRStatement::Push(address), ir::IRStatement::Mov]);
            }
        }
    }
}
