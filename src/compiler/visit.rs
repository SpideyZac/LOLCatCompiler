use std::collections::HashMap;
use std::panic;

use crate::compiler::ir;
use crate::lexer::tokens;
use crate::parser::ast;
use crate::parser::ast::VariableAssignmentNodeVariableOption;
use crate::parser::parser;

#[derive(Clone)]
pub enum Types {
    Number,
    Numbar,
    Yarn(i32), // size of the string
    Troof,
    Noob,
}

impl Types {
    pub fn to_string(&self) -> String {
        match self {
            Types::Number => "NUMBER".to_string(),
            Types::Numbar => "NUMBAR".to_string(),
            Types::Yarn(_) => "YARN".to_string(),
            Types::Troof => "TROOF".to_string(),
            Types::Noob => "NOOB".to_string(),
        }
    }

    pub fn equals(&self, other: &Types) -> bool {
        match self {
            Types::Number => match other {
                Types::Number => true,
                _ => false,
            },
            Types::Numbar => match other {
                Types::Numbar => true,
                _ => false,
            },
            Types::Yarn(_) => match other {
                Types::Yarn(_) => true,
                _ => false,
            },
            Types::Troof => match other {
                Types::Troof => true,
                _ => false,
            },
            Types::Noob => match other {
                Types::Noob => true,
                _ => false,
            },
        }
    }
}

pub struct VariableValue {
    pub hook: i32,
    pub type_: Types,
}

impl VariableValue {
    pub fn new(hook: i32, type_: Types) -> VariableValue {
        VariableValue { hook, type_ }
    }
}

pub struct VariableData {
    pub value: VariableValue,
}

impl VariableData {
    pub fn new(value: VariableValue) -> VariableData {
        VariableData { value }
    }

    pub fn free(&self) -> Vec<ir::IRStatement> {
        match self.value.type_ {
            Types::Yarn(size) => {
                vec![
                    ir::IRStatement::Push(size as f32),
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Free,
                ]
            }
            _ => vec![],
        }
    }

    pub fn copy(&self, hook: i32) -> (VariableValue, Vec<ir::IRStatement>) {
        match self.value.type_ {
            Types::Number => {
                let ir = vec![
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Hook(hook),
                ];

                (VariableValue::new(hook, Types::Number), ir)
            }
            Types::Numbar => {
                let ir = vec![
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Hook(hook),
                ];

                (VariableValue::new(hook, Types::Numbar), ir)
            }
            Types::Troof => {
                let ir = vec![
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Hook(hook),
                ];

                (VariableValue::new(hook, Types::Troof), ir)
            }
            Types::Yarn(size) => {
                let ir = vec![
                    ir::IRStatement::Push(size as f32),
                    ir::IRStatement::Allocate,
                    ir::IRStatement::Hook(hook),
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Load(size),
                    ir::IRStatement::RefHook(hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Store(size),
                ];

                (VariableValue::new(hook, Types::Yarn(size)), ir)
            }
            _ => panic!("Unexpected type"),
        }
    }

    pub fn assign(&mut self, type_: &Types) -> Vec<ir::IRStatement> {
        self.value.type_ = type_.clone();

        match type_ {
            Types::Number => vec![
                // assumes that the value is already on the stack
                ir::IRStatement::RefHook(self.value.hook),
                ir::IRStatement::Mov,
            ],
            Types::Numbar => vec![
                // assumes that the value is already on the stack
                ir::IRStatement::RefHook(self.value.hook),
                ir::IRStatement::Mov,
            ],
            Types::Troof => vec![
                // assumes that the value is already on the stack
                ir::IRStatement::RefHook(self.value.hook),
                ir::IRStatement::Mov,
            ],
            Types::Yarn(size) => {
                let ir = vec![
                    // assumes that the value is already on the stack
                    ir::IRStatement::RefHook(self.value.hook),
                    ir::IRStatement::Mov,
                ];

                self.value.type_ = Types::Yarn(*size);

                ir
            }
            _ => panic!("Unexpected type"),
        }
    }
}

pub struct Scope<'a> {
    pub name: String,
    pub variables: HashMap<String, VariableData>,
    pub parent: Option<&'a mut Scope<'a>>,
    pub sub_scopes: Vec<Scope<'a>>,
    pub used_hooks: Vec<i32>,
}

impl<'a> Scope<'a> {
    pub fn new(name: String, parent: Option<&'a mut Scope<'a>>) -> Scope<'a> {
        Scope {
            name,
            variables: HashMap::new(),
            parent,
            sub_scopes: vec![],
            used_hooks: vec![],
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableData> {
        match self.variables.get(name) {
            Some(data) => Some(data),
            None => match &self.parent {
                Some(parent) => parent.get_variable(name),
                None => None,
            },
        }
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VariableData> {
        match self.variables.get_mut(name) {
            Some(data) => Some(data),
            None => match &mut self.parent {
                Some(parent) => parent.get_variable_mut(name),
                None => None,
            },
        }
    }

    pub fn add_variable(&mut self, name: String, value: VariableData) {
        self.variables.insert(name, value);
    }

    pub fn add_sub_scope(&mut self, scope: Scope<'a>) {
        self.sub_scopes.push(scope);
    }

    pub fn add_hook(&mut self, hook: i32) {
        self.used_hooks.push(hook);
    }

    pub fn free(&self) -> Vec<ir::IRStatement> {
        let mut ir = vec![];

        for (_, variable) in self.variables.iter() {
            ir.append(&mut variable.free());
        }

        for scope in self.sub_scopes.iter() {
            ir.append(&mut scope.free());
        }

        ir
    }
}

#[derive(Clone)]
pub struct VisitorError {
    pub message: String,
    pub token: ast::TokenNode,
}

pub struct Visitor<'a> {
    pub ast_tree: parser::ParserReturn<'a>,
    pub scopes: Vec<Scope<'a>>,
    pub current_scope_index: usize,
    pub max_hook: i32,
    pub used_hooks: Vec<i32>,
    pub ir: ir::IR,
    pub errors: Vec<VisitorError>,
}

impl<'a> Visitor<'a> {
    pub fn get_scope(&self) -> &Scope<'a> {
        &self.scopes[self.current_scope_index]
    }

    pub fn get_scope_mut(&mut self) -> &mut Scope<'a> {
        &mut self.scopes[self.current_scope_index]
    }

    pub fn add_statements(&mut self, statements: Vec<ir::IRStatement>) {
        let scope = self.get_scope();
        let name = scope.name.clone();

        if name == "main" {
            self.ir.entry.statements.extend(statements);
        } else {
            for function in self.ir.functions.iter_mut() {
                if function.name == name {
                    function.statements.extend(statements);
                    return;
                }
            }

            panic!("Function not found");
        }
    }

    pub fn get_hook(&mut self) -> (i32, ir::IRStatement) {
        for hook in 0..self.max_hook {
            if !self.used_hooks.contains(&hook) {
                self.used_hooks.push(hook);
                let scope = self.get_scope_mut();
                scope.add_hook(hook);
                return (hook, ir::IRStatement::Hook(hook));
            }
        }

        let hook = self.max_hook;
        self.used_hooks.push(hook);
        let stmt = ir::IRStatement::Hook(hook);
        let scope = self.get_scope_mut();
        scope.add_hook(hook);
        self.max_hook += 1;
        return (hook, stmt);
    }

    pub fn free_scope(&mut self) {
        let scope = self.get_scope();
        let ir = scope.free();
        for hook in scope.used_hooks.clone().iter() {
            self.used_hooks.retain(|&x| x != *hook);
        }
        self.add_statements(ir);
    }

    pub fn free_hook(&mut self, hook: i32) {
        self.used_hooks.retain(|&x| x != hook);
    }

    pub fn new(ast_tree: parser::ParserReturn<'a>, stack_size: i32, heap_size: i32) -> Self {
        let entry = ir::IRFunctionEntry::new(stack_size, heap_size, vec![]);
        let mut visitor = Self {
            ast_tree,
            errors: vec![],
            scopes: vec![Scope::new("main".to_string(), None)],
            current_scope_index: 0,
            max_hook: 0,
            used_hooks: vec![],
            ir: ir::IR::new(vec![], entry),
        };

        visitor.add_statements(vec![ir::IRStatement::Push(0.0)]);
        let (hook, stmt) = visitor.get_hook();
        let main_scope = visitor.get_scope_mut();
        main_scope.add_variable(
            "IT".to_string(),
            VariableData::new(VariableValue::new(hook, Types::Noob)),
        );
        visitor.add_statements(vec![stmt]);

        visitor
    }
}

impl<'a> Visitor<'a> {
    pub fn visit(&mut self) -> (ir::IR, Vec<VisitorError>, i32) {
        self.visit_program(self.ast_tree.ast.clone());

        (self.ir.clone(), self.errors.clone(), self.max_hook)
    }

    pub fn visit_program(&mut self, program: ast::ProgramNode) {
        for statement in program.statements {
            self.visit_statement(statement.clone());
        }
    }

    pub fn visit_statement(&mut self, statement: ast::StatementNode) {
        match statement.value {
            ast::StatementNodeValueOption::Expression(expression) => {
                let var = self.get_scope().get_variable("IT").unwrap();
                if var.value.type_.equals(&Types::Yarn(0)) {
                    self.add_statements(var.free());
                }

                let (variable_value, _) = self.visit_expression(expression);
                self.free_hook(variable_value.hook);

                match variable_value.type_ {
                    Types::Number => {
                        let it = self.get_scope_mut().get_variable_mut("IT").unwrap();
                        let stmts = it.assign(&Types::Number);
                        self.add_statements(stmts);
                    }
                    Types::Numbar => {
                        let it = self.get_scope_mut().get_variable_mut("IT").unwrap();
                        let stmts = it.assign(&Types::Numbar);
                        self.add_statements(stmts);
                    }
                    Types::Troof => {
                        let it = self.get_scope_mut().get_variable_mut("IT").unwrap();
                        let stmts = it.assign(&Types::Troof);
                        self.add_statements(stmts);
                    }
                    Types::Yarn(size) => {
                        let it = self.get_scope_mut().get_variable_mut("IT").unwrap();
                        let stmts = it.assign(&Types::Yarn(size));
                        self.add_statements(stmts);
                    }
                    _ => {
                        panic!("Unexpected type");
                    }
                }
            }
            ast::StatementNodeValueOption::VariableDeclarationStatement(var_dec) => {
                self.visit_variable_declaration(var_dec);
            }
            ast::StatementNodeValueOption::VariableAssignmentStatement(var_assign) => {
                self.visit_variable_assignment(var_assign);
            }
            ast::StatementNodeValueOption::KTHXBYEStatement(_) => {
                self.add_statements(vec![ir::IRStatement::Halt]);
            }
            _ => {
                panic!("Unexpected statement");
            }
        }
    }

    pub fn visit_expression(
        &mut self,
        expression: ast::ExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        match expression.value {
            ast::ExpressionNodeValueOption::NumberValue(number) => {
                self.visit_number_value(number.clone())
            }
            ast::ExpressionNodeValueOption::NumbarValue(numbar) => {
                self.visit_numbar_value(numbar.clone())
            }
            ast::ExpressionNodeValueOption::TroofValue(troof) => {
                self.visit_troof_value(troof.clone())
            }
            ast::ExpressionNodeValueOption::YarnValue(yarn) => self.visit_yarn_value(yarn.clone()),
            _ => {
                panic!("Unexpected expression");
            }
        }
    }

    pub fn visit_number_value(
        &mut self,
        number: ast::NumberValueNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(number.value() as f32)]);
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Number);

        (variable, number.token)
    }

    pub fn visit_numbar_value(
        &mut self,
        numbar: ast::NumbarValueNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(numbar.value())]);
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Numbar);

        (variable, numbar.token)
    }

    pub fn visit_troof_value(
        &mut self,
        troof: ast::TroofValueNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(if troof.value() {
            1.0
        } else {
            0.0
        })]);
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Troof);

        (variable, troof.token)
    }

    pub fn visit_yarn_value(
        &mut self,
        yarn: ast::YarnValueNode,
    ) -> (VariableValue, ast::TokenNode) {
        let string = yarn.value();
        let size = string.len() as i32;
        self.add_statements(vec![
            ir::IRStatement::Push(size as f32),
            ir::IRStatement::Allocate,
        ]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        for c in string.chars() {
            self.add_statements(vec![ir::IRStatement::Push(c as i32 as f32)]);
        }

        self.add_statements(vec![
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Store(size),
        ]);

        let variable = VariableValue::new(hook, Types::Yarn(size));

        (variable, yarn.token)
    }

    pub fn visit_variable_declaration(&mut self, var_dec: ast::VariableDeclarationStatementNode) {
        let token = var_dec.identifier;
        let name = match token.value() {
            tokens::Token::Identifier(name) => name,
            _ => panic!("Expected Identifier token"),
        };

        let scope = self.get_scope();
        let variable = scope.get_variable(&name);
        if let Some(_) = variable {
            self.errors.push(VisitorError {
                message: format!("Variable {} already declared", name),
                token,
            });
            return;
        }

        let type_ = match var_dec.type_.token.token.to_name().as_str() {
            "Word_NUMBER" => Types::Number,
            "Word_NUMBAR" => Types::Numbar,
            "Word_TROOF" => Types::Troof,
            "Word_YARN" => Types::Yarn(-1), // unknown size
            _ => panic!("Unexpected type"),
        };

        self.add_statements(vec![ir::IRStatement::Push(0.0)]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableData::new(VariableValue::new(hook, type_));
        let scope_mut = self.get_scope_mut();
        scope_mut.add_variable(name.clone(), variable);
    }

    pub fn visit_variable_assignment(&mut self, var_assign: ast::VariableAssignmentStatementNode) {
        match var_assign.variable {
            VariableAssignmentNodeVariableOption::Identifier(token) => {
                let name = match token.value() {
                    tokens::Token::Identifier(name) => name,
                    _ => panic!("Expected Identifier token"),
                };

                let (expression, t) = self.visit_expression(var_assign.expression.clone());
                self.free_hook(expression.hook);

                let scope = self.get_scope();
                let variable = scope.get_variable(&name);
                if let None = variable {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token,
                    });
                    return;
                }

                if !expression.type_.equals(&variable.unwrap().value.type_) {
                    self.errors.push(VisitorError {
                        message: format!(
                            "Variable {} is of type {} but expression is of type {}",
                            name,
                            variable.unwrap().value.type_.to_string(),
                            expression.type_.to_string()
                        ),
                        token: t,
                    });
                    return;
                }

                let scope_mut = self.get_scope_mut();
                let variable_mut = scope_mut.get_variable_mut(&name).unwrap();
                let stmts = variable_mut.assign(&expression.type_);
                self.add_statements(stmts);
            }
            ast::VariableAssignmentNodeVariableOption::VariableDeclerationStatement(var_dec) => {
                self.visit_variable_declaration(var_dec.clone());

                let token = var_dec.identifier;

                let name = match token.value() {
                    tokens::Token::Identifier(name) => name,
                    _ => panic!("Expected Identifier token"),
                };

                let (expression, t) = self.visit_expression(var_assign.expression.clone());
                self.free_hook(expression.hook);

                let scope = self.get_scope();
                let variable = scope.get_variable(&name);
                if let None = variable {
                    self.errors.push(VisitorError {
                        message: format!("Variable {} not declared", name),
                        token,
                    });
                    return;
                }

                if !expression.type_.equals(&variable.unwrap().value.type_) {
                    self.errors.push(VisitorError {
                        message: format!(
                            "Variable {} is of type {} but expression is of type {}",
                            name,
                            variable.unwrap().value.type_.to_string(),
                            expression.type_.to_string()
                        ),
                        token: t,
                    });
                    return;
                }

                let scope_mut = self.get_scope_mut();
                let variable_mut = scope_mut.get_variable_mut(&name).unwrap();
                let stmts = variable_mut.assign(&expression.type_);
                self.add_statements(stmts);
            }
        }
    }
}
