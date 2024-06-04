use std::collections::HashMap;

use crate::compiler::ir;
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
            .insert("IT_NUMBER".to_string(), VariableTypes::Number);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_NUMBER".to_string(), -1);
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
            .insert("IT_NUMBAR".to_string(), -2);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_YARN".to_string(), VariableTypes::Yarn);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_YARN".to_string(), -3);
        visitor.program_state.entry_function_state.variables += 1;
        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);

        visitor
            .program_state
            .entry_function_state
            .variable_map
            .insert("IT_TROOF".to_string(), VariableTypes::Troof);
        visitor
            .program_state
            .entry_function_state
            .variable_addresses
            .insert("IT_TROOF".to_string(), -4);
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

                    match type_ {
                        VariableTypes::Number => {
                            self.add_statements(vec![
                                ir::IRStatement::Push(
                                    *scope.variable_addresses.get("IT_NUMBER").unwrap() as f32,
                                ),
                                ir::IRStatement::Mov,
                            ]);
                        }
                        _ => {
                            panic!("Unexpected type")
                        }
                    }
                }
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
            _ => {
                panic!("Unexpected expression")
            }
        }
    }

    pub fn visit_number_value(&mut self, number_value: ast::NumberValueNode) {
        self.add_statements(vec![ir::IRStatement::Push(number_value.value() as f32)]);
    }
}
