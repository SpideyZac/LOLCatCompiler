use std::collections::HashMap;

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

    pub fn free(&self) -> Vec<ir::IRStatement> {
        match self.type_ {
            Types::Yarn(size) => {
                vec![
                    ir::IRStatement::Push(size as f32),
                    ir::IRStatement::RefHook(self.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Free,
                ]
            }
            _ => vec![],
        }
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
                if size >= 0 {
                    vec![
                        ir::IRStatement::Push(size as f32),
                        ir::IRStatement::RefHook(self.value.hook),
                        ir::IRStatement::Copy,
                        ir::IRStatement::Free,
                    ]
                } else {
                    vec![]
                }
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

    pub fn get_statements(&self) -> Vec<ir::IRStatement> {
        let scope = self.get_scope();
        let name = scope.name.clone();

        if name == "main" {
            self.ir.entry.statements.clone()
        } else {
            for function in self.ir.functions.iter() {
                if function.name == name {
                    return function.statements.clone();
                }
            }

            panic!("Function not found");
        }
    }

    pub fn set_statements(&mut self, statements: Vec<ir::IRStatement>) {
        let scope = self.get_scope();
        let name = scope.name.clone();

        if name == "main" {
            self.ir.entry.statements = statements;
        } else {
            for function in self.ir.functions.iter_mut() {
                if function.name == name {
                    function.statements = statements;
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
                self.add_statements(var.free());

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
            ast::StatementNodeValueOption::VisibleStatement(visible_stmt) => {
                self.visit_visible_statement(visible_stmt);
            }
            ast::StatementNodeValueOption::GimmehStatement(gimmeh_stmt) => {
                self.visit_gimmeh_statement(gimmeh_stmt);
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
            ast::ExpressionNodeValueOption::VariableReference(var_ref) => {
                self.visit_variable_reference(var_ref.clone())
            }
            ast::ExpressionNodeValueOption::SumExpression(sum_expr) => {
                self.visit_sum_expression(sum_expr.clone())
            }
            ast::ExpressionNodeValueOption::DiffExpression(diff_expr) => {
                self.visit_difference_expression(diff_expr.clone())
            }
            ast::ExpressionNodeValueOption::ProduktExpression(prod_expr) => {
                self.visit_product_expression(prod_expr.clone())
            }
            ast::ExpressionNodeValueOption::QuoshuntExpression(quoshunt_expr) => {
                self.visit_quoshunt_expression(quoshunt_expr.clone())
            }
            ast::ExpressionNodeValueOption::ModExpression(mod_expr) => {
                self.visit_mod_expression(mod_expr.clone())
            }
            ast::ExpressionNodeValueOption::BiggrExpression(biggr_expr) => {
                self.visit_biggr_expression(biggr_expr.clone())
            }
            ast::ExpressionNodeValueOption::SmallrExpression(smallr_expr) => {
                self.visit_smallr_expression(smallr_expr.clone())
            }
            ast::ExpressionNodeValueOption::BothOfExpression(both_of_expr) => {
                self.visit_both_of_expression(both_of_expr.clone())
            }
            ast::ExpressionNodeValueOption::EitherOfExpression(either_of_expr) => {
                self.visit_either_of_expression(either_of_expr.clone())
            }
            ast::ExpressionNodeValueOption::WonOfExpression(won_of_expr) => {
                self.visit_won_of_expression(won_of_expr.clone())
            }
            ast::ExpressionNodeValueOption::NotExpression(not_expr) => {
                self.visit_not_expression(not_expr.clone())
            }
            ast::ExpressionNodeValueOption::AllOfExpression(all_of_expr) => {
                self.visit_all_of_expression(all_of_expr.clone())
            }
            ast::ExpressionNodeValueOption::AnyOfExpression(any_of_expr) => {
                self.visit_any_of_expression(any_of_expr.clone())
            }
            ast::ExpressionNodeValueOption::BothSaemExpression(both_saem_expr) => {
                self.visit_both_saem_expression(both_saem_expr.clone())
            }
            ast::ExpressionNodeValueOption::DiffrintExpression(diffrint_expr) => {
                self.visit_diffrint_expression(diffrint_expr.clone())
            }
            ast::ExpressionNodeValueOption::SmooshExpression(smoosh_expr) => {
                self.visit_smoosh_expression(smoosh_expr.clone())
            }
            ast::ExpressionNodeValueOption::MaekExpression(maek_expr) => {
                self.visit_maek_expression(maek_expr.clone())
            }
            ast::ExpressionNodeValueOption::ItReference(it_ref) => {
                self.visit_it_reference(it_ref.clone())
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

    pub fn visit_variable_reference(
        &mut self,
        var_ref: ast::VariableReferenceNode,
    ) -> (VariableValue, ast::TokenNode) {
        let name = match var_ref.identifier.value() {
            tokens::Token::Identifier(name) => name,
            _ => panic!("Expected Identifier token"),
        };

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = self.get_scope().get_variable(name);
        if let None = variable {
            self.errors.push(VisitorError {
                message: format!("Variable {} not found", name),
                token: var_ref.identifier.clone(),
            });
            return (
                VariableValue::new(-1, Types::Noob),
                var_ref.identifier.clone(),
            );
        }
        let (var, stmts) = variable.unwrap().copy(hook);
        self.add_statements(stmts);

        (var, var_ref.identifier)
    }

    pub fn visit_sum_expression(
        &mut self,
        sum_expr: ast::SumExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*sum_expr.left.clone());
        let (right, right_token) = self.visit_expression(*sum_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Add]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_difference_expression(
        &mut self,
        diff_expr: ast::DiffExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*diff_expr.left.clone());
        let (right, right_token) = self.visit_expression(*diff_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Subtract]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_product_expression(
        &mut self,
        prod_expr: ast::ProduktExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*prod_expr.left.clone());
        let (right, right_token) = self.visit_expression(*prod_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Multiply]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_quoshunt_expression(
        &mut self,
        quoshunt_expr: ast::QuoshuntExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*quoshunt_expr.left.clone());
        let (right, right_token) = self.visit_expression(*quoshunt_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Divide]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_mod_expression(
        &mut self,
        mod_expr: ast::ModExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*mod_expr.left.clone());
        let (right, right_token) = self.visit_expression(*mod_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Number) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Modulo]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Number);

        (variable, left_token)
    }

    pub fn visit_biggr_expression(
        &mut self,
        biggr_expr: ast::BiggrExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(0.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*biggr_expr.left.clone());
        let (right, right_token) = self.visit_expression(*biggr_expr.right.clone());

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![
            ir::IRStatement::RefHook(left.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Subtract,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Push(1.0),
            ir::IRStatement::Add,
            ir::IRStatement::Copy,
            ir::IRStatement::Sign,
            ir::IRStatement::Multiply,
            ir::IRStatement::RefHook(left.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Add,
            ir::IRStatement::Add,
            ir::IRStatement::Push(2.0),
            ir::IRStatement::Divide,
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Mov,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
        ]);

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_smallr_expression(
        &mut self,
        smallr_expr: ast::SmallrExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(0.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*smallr_expr.left.clone());
        let (right, right_token) = self.visit_expression(*smallr_expr.right.clone());

        if !left.type_.equals(&Types::Number) && !left.type_.equals(&Types::Numbar) {
            self.errors.push(VisitorError {
                message: "Expected NUMBER or NUMBAR type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&left.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![
            ir::IRStatement::RefHook(left.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Subtract,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Push(1.0),
            ir::IRStatement::Add,
            ir::IRStatement::Copy,
            ir::IRStatement::Sign,
            ir::IRStatement::Multiply,
            ir::IRStatement::RefHook(left.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::RefHook(right.hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Add,
            ir::IRStatement::Subtract,
            ir::IRStatement::Push(2.0),
            ir::IRStatement::Divide,
            ir::IRStatement::Push(-1.0),
            ir::IRStatement::Multiply,
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Mov,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
        ]);

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        let variable = VariableValue::new(hook, left.type_.clone());

        (variable, left_token)
    }

    pub fn visit_both_of_expression(
        &mut self,
        both_of_expr: ast::BothOfExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(0.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*both_of_expr.left.clone());
        let (right, right_token) = self.visit_expression(*both_of_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![
            ir::IRStatement::Multiply,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(1.0),
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Mov,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
        ]);

        let variable = VariableValue::new(hook, Types::Troof);
        (variable, left_token)
    }

    pub fn visit_either_of_expression(
        &mut self,
        either_of_expr: ast::EitherOfExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (left, left_token) = self.visit_expression(*either_of_expr.left.clone());
        let (right, right_token) = self.visit_expression(*either_of_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![ir::IRStatement::Add]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Troof);
        (variable, left_token)
    }

    pub fn visit_won_of_expression(
        &mut self,
        won_of_expr: ast::WonOfExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(0.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*won_of_expr.left.clone());
        let (right, right_token) = self.visit_expression(*won_of_expr.right.clone());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        if !left.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: left_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), left_token);
        }

        if !right.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        self.add_statements(vec![
            ir::IRStatement::Add,
            ir::IRStatement::Push(2.0),
            ir::IRStatement::Modulo,
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(1.0),
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Mov,
            ir::IRStatement::Push(0.0), // break out of loop
            ir::IRStatement::EndWhile,
        ]);

        let variable = VariableValue::new(hook, Types::Troof);
        (variable, left_token)
    }

    pub fn visit_not_expression(
        &mut self,
        not_expr: ast::NotExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (expression, token) = self.visit_expression(*not_expr.expression.clone());

        self.free_hook(expression.hook);

        if !expression.type_.equals(&Types::Troof) {
            self.errors.push(VisitorError {
                message: "Expected TROOF type".to_string(),
                token: token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), token);
        }

        self.add_statements(vec![
            ir::IRStatement::Push(1.0),
            ir::IRStatement::Add,
            ir::IRStatement::Push(2.0),
            ir::IRStatement::Modulo,
        ]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let variable = VariableValue::new(hook, Types::Troof);

        (variable, token)
    }

    pub fn visit_all_of_expression(
        &mut self,
        all_of_expr: ast::AllOfExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(1.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let mut t = None;
        self.add_statements(vec![ir::IRStatement::Push(1.0)]);
        for expression in all_of_expr.expressions.iter() {
            let (exp, token) = self.visit_expression(expression.clone());

            self.free_hook(exp.hook);

            if !exp.type_.equals(&Types::Troof) {
                self.errors.push(VisitorError {
                    message: "Expected TROOF type".to_string(),
                    token: token.clone(),
                });
                return (VariableValue::new(-1, Types::Noob), token);
            }
            t = Some(token);

            self.add_statements(vec![ir::IRStatement::Multiply]);
            let (hook_of_running_total, stmt) = self.get_hook();
            self.add_statements(vec![stmt]);

            self.add_statements(vec![
                ir::IRStatement::RefHook(hook_of_running_total),
                ir::IRStatement::Copy,
                ir::IRStatement::Push(1.0),
                ir::IRStatement::Add,
                ir::IRStatement::Push(2.0),
                ir::IRStatement::Modulo,
                ir::IRStatement::BeginWhile,
                ir::IRStatement::Push(0.0),
                ir::IRStatement::RefHook(hook),
                ir::IRStatement::Mov,
                ir::IRStatement::Push(0.0),
                ir::IRStatement::EndWhile,
            ]);

            self.free_hook(hook_of_running_total);
        }

        self.add_statements(vec![
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
        ]);

        (VariableValue::new(hook, Types::Troof), t.unwrap())
    }

    pub fn visit_any_of_expression(
        &mut self,
        any_of_expr: ast::AnyOfExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(0.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let mut t = None;
        for expression in any_of_expr.expressions.iter() {
            let (exp, token) = self.visit_expression(expression.clone());

            self.free_hook(exp.hook);

            if !exp.type_.equals(&Types::Troof) {
                self.errors.push(VisitorError {
                    message: "Expected TROOF type".to_string(),
                    token: token.clone(),
                });
                return (VariableValue::new(-1, Types::Noob), token);
            }
            t = Some(token);

            self.add_statements(vec![
                ir::IRStatement::BeginWhile,
                ir::IRStatement::Push(1.0),
                ir::IRStatement::RefHook(hook),
                ir::IRStatement::Mov,
                ir::IRStatement::Push(0.0),
                ir::IRStatement::EndWhile,
            ]);
        }

        (VariableValue::new(hook, Types::Troof), t.unwrap())
    }

    pub fn visit_both_saem_expression(
        &mut self,
        both_saem_expr: ast::BothSaemExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(1.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*both_saem_expr.left.clone());
        let (right, right_token) = self.visit_expression(*both_saem_expr.right.clone());

        if !left.type_.equals(&right.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        match left.type_ {
            Types::Number | Types::Numbar | Types::Troof => {
                self.add_statements(vec![
                    ir::IRStatement::Subtract,
                    ir::IRStatement::BeginWhile,
                    ir::IRStatement::Push(0.0),
                    ir::IRStatement::RefHook(hook),
                    ir::IRStatement::Mov,
                    ir::IRStatement::Push(0.0),
                    ir::IRStatement::EndWhile,
                ]);
            }
            Types::Yarn(size) => match right.type_ {
                Types::Yarn(size2) => {
                    if size != size2 {
                        self.add_statements(vec![
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::RefHook(hook),
                            ir::IRStatement::Mov,
                        ]);
                    } else {
                        for i in 0..size {
                            self.add_statements(vec![
                                ir::IRStatement::RefHook(left.hook),
                                ir::IRStatement::Copy,
                                ir::IRStatement::Push(i as f32 * 4.0),
                                ir::IRStatement::Add,
                                ir::IRStatement::Load(1),
                                ir::IRStatement::RefHook(right.hook),
                                ir::IRStatement::Copy,
                                ir::IRStatement::Push(i as f32 * 4.0),
                                ir::IRStatement::Add,
                                ir::IRStatement::Load(1),
                                ir::IRStatement::Subtract,
                                ir::IRStatement::BeginWhile,
                                ir::IRStatement::Push(0.0),
                                ir::IRStatement::RefHook(hook),
                                ir::IRStatement::Mov,
                                ir::IRStatement::Push(0.0),
                                ir::IRStatement::EndWhile,
                            ]);
                        }

                        self.add_statements(vec![
                            ir::IRStatement::BeginWhile,
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::EndWhile,
                            ir::IRStatement::BeginWhile,
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::EndWhile,
                        ]);
                    }
                }
                _ => {
                    panic!("Unexpected type");
                }
            },
            _ => {
                panic!("Unexpected type");
            }
        };

        self.add_statements(left.free());
        self.add_statements(right.free());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        (VariableValue::new(hook, Types::Troof), left_token)
    }

    pub fn visit_diffrint_expression(
        &mut self,
        diffrint_expr: ast::DiffrintExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        self.add_statements(vec![ir::IRStatement::Push(1.0)]); // return value
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let (left, left_token) = self.visit_expression(*diffrint_expr.left.clone());
        let (right, right_token) = self.visit_expression(*diffrint_expr.right.clone());

        if !left.type_.equals(&right.type_) {
            self.errors.push(VisitorError {
                message: format!(
                    "Expected {} type but got {}",
                    left.type_.to_string(),
                    right.type_.to_string()
                ),
                token: right_token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), right_token);
        }

        match left.type_ {
            Types::Number | Types::Numbar | Types::Troof => {
                self.add_statements(vec![
                    ir::IRStatement::Subtract,
                    ir::IRStatement::BeginWhile,
                    ir::IRStatement::Push(0.0),
                    ir::IRStatement::RefHook(hook),
                    ir::IRStatement::Mov,
                    ir::IRStatement::Push(0.0),
                    ir::IRStatement::EndWhile,
                ]);
            }
            Types::Yarn(size) => match right.type_ {
                Types::Yarn(size2) => {
                    if size != size2 {
                        self.add_statements(vec![
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::RefHook(hook),
                            ir::IRStatement::Mov,
                        ]);
                    } else {
                        for i in 0..size {
                            self.add_statements(vec![
                                ir::IRStatement::RefHook(left.hook),
                                ir::IRStatement::Copy,
                                ir::IRStatement::Push(i as f32 * 4.0),
                                ir::IRStatement::Add,
                                ir::IRStatement::Load(1),
                                ir::IRStatement::RefHook(right.hook),
                                ir::IRStatement::Copy,
                                ir::IRStatement::Push(i as f32 * 4.0),
                                ir::IRStatement::Add,
                                ir::IRStatement::Load(1),
                                ir::IRStatement::Subtract,
                                ir::IRStatement::BeginWhile,
                                ir::IRStatement::Push(0.0),
                                ir::IRStatement::RefHook(hook),
                                ir::IRStatement::Mov,
                                ir::IRStatement::Push(0.0),
                                ir::IRStatement::EndWhile,
                            ]);
                        }

                        self.add_statements(vec![
                            ir::IRStatement::BeginWhile,
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::EndWhile,
                            ir::IRStatement::BeginWhile,
                            ir::IRStatement::Push(0.0),
                            ir::IRStatement::EndWhile,
                        ]);
                    }
                }
                _ => {
                    panic!("Unexpected type");
                }
            },
            _ => {
                panic!("Unexpected type");
            }
        };

        self.add_statements(left.free());
        self.add_statements(right.free());

        self.free_hook(left.hook);
        self.free_hook(right.hook);

        self.add_statements(vec![
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Copy,
            ir::IRStatement::Push(1.0),
            ir::IRStatement::Add,
            ir::IRStatement::Push(2.0),
            ir::IRStatement::Modulo,
            ir::IRStatement::RefHook(hook),
            ir::IRStatement::Mov,
        ]);

        (VariableValue::new(hook, Types::Troof), left_token)
    }

    pub fn visit_smoosh_expression(
        &mut self,
        smoosh_expr: ast::SmooshExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let mut size = 0;
        let mut token = None;

        let old_scope = self.get_statements();

        for expression in smoosh_expr.expressions.iter() {
            let (exp, t) = self.visit_expression(expression.clone());

            if !exp.type_.equals(&Types::Yarn(-1)) {
                self.errors.push(VisitorError {
                    message: "Expected YARN type".to_string(),
                    token: t.clone(),
                });
                return (VariableValue::new(-1, Types::Noob), t);
            }

            token = Some(t);

            let size_local = match exp.type_ {
                Types::Yarn(size) => size,
                _ => panic!("Unexpected type"),
            };

            size += size_local;
        }

        self.set_statements(old_scope);

        self.add_statements(vec![
            ir::IRStatement::Push(size as f32),
            ir::IRStatement::Allocate,
        ]);

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);
        let mut size_passed = 0;

        for expression in smoosh_expr.expressions.iter() {
            let (exp, _) = self.visit_expression(expression.clone());

            let size_local = match exp.type_ {
                Types::Yarn(size) => size,
                _ => panic!("Unexpected type"),
            };

            self.add_statements(vec![
                ir::IRStatement::RefHook(exp.hook),
                ir::IRStatement::Copy,
                ir::IRStatement::Load(size_local),
                ir::IRStatement::RefHook(hook),
                ir::IRStatement::Copy,
                ir::IRStatement::Push(size_passed as f32 * 4.0),
                ir::IRStatement::Add,
                ir::IRStatement::Store(size_local),
            ]);

            self.add_statements(exp.free());
            self.free_hook(exp.hook);

            self.add_statements(vec![
                ir::IRStatement::BeginWhile,
                ir::IRStatement::Push(0.0),
                ir::IRStatement::EndWhile,
            ]);

            size_passed += size_local;
        }

        (VariableValue::new(hook, Types::Yarn(size)), token.unwrap())
    }

    pub fn visit_maek_expression(
        &mut self,
        maek_expr: ast::MaekExpressionNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (expression, token) = self.visit_expression(*maek_expr.expression.clone());

        self.free_hook(expression.hook);

        let mut type_ = match maek_expr.type_.token.token.to_name().as_str() {
            "Word_NUMBER" => Types::Number,
            "Word_NUMBAR" => Types::Numbar,
            "Word_TROOF" => Types::Troof,
            "Word_YARN" => Types::Yarn(-1), // unknown size
            _ => panic!("Unexpected type"),
        };

        match type_ {
            Types::Number => {
                match expression.type_ {
                    Types::Number => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Numbar => {
                        self.add_statements(vec![ir::IRStatement::CallForeign(
                            "float_to_int".to_string(),
                        )]);
                    }
                    Types::Troof => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Yarn(size) => {
                        self.add_statements(vec![
                            ir::IRStatement::Push(size as f32),
                            ir::IRStatement::CallForeign("string_to_int".to_string()),
                        ]);
                    }
                    Types::Noob => {
                        self.errors.push(VisitorError {
                            message: "Cannot convert type NOOB to NUMBER".to_string(),
                            token: token.clone(),
                        });
                        return (VariableValue::new(-1, Types::Noob), token);
                    }
                };
            }
            Types::Numbar => {
                match expression.type_ {
                    Types::Number => {
                        self.add_statements(vec![ir::IRStatement::CallForeign(
                            "int_to_float".to_string(),
                        )]);
                    }
                    Types::Numbar => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Troof => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Yarn(size) => {
                        self.add_statements(vec![
                            ir::IRStatement::Push(size as f32),
                            ir::IRStatement::CallForeign("string_to_float".to_string()),
                        ]);
                    }
                    Types::Noob => {
                        self.errors.push(VisitorError {
                            message: "Cannot convert type NOOB to NUMBAR".to_string(),
                            token: token.clone(),
                        });
                        return (VariableValue::new(-1, Types::Noob), token);
                    }
                };
            }
            Types::Troof => {
                match expression.type_ {
                    Types::Number => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Numbar => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Troof => {
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Yarn(size) => {
                        self.add_statements(vec![ir::IRStatement::Push(if size == 0 {
                            0.0
                        } else {
                            1.0
                        })]);
                    }
                    Types::Noob => {
                        self.errors.push(VisitorError {
                            message: "Cannot convert type NOOB to TROOF".to_string(),
                            token: token.clone(),
                        });
                        return (VariableValue::new(-1, Types::Noob), token);
                    }
                };
            }
            Types::Yarn(_) => {
                match expression.type_ {
                    Types::Number => {
                        type_ = Types::Yarn(32);
                        self.add_statements(vec![ir::IRStatement::CallForeign(
                            "int_to_string".to_string(),
                        )]);
                    }
                    Types::Numbar => {
                        type_ = Types::Yarn(32);
                        self.add_statements(vec![ir::IRStatement::CallForeign(
                            "float_to_string".to_string(),
                        )]);
                    }
                    Types::Troof => {
                        type_ = Types::Yarn(32);
                        self.add_statements(vec![ir::IRStatement::CallForeign(
                            "int_to_string".to_string(),
                        )]);
                    }
                    Types::Yarn(size) => {
                        type_ = Types::Yarn(size);
                        self.add_statements(vec![
                            ir::IRStatement::RefHook(expression.hook),
                            ir::IRStatement::Copy,
                        ]);
                    }
                    Types::Noob => {
                        self.errors.push(VisitorError {
                            message: "Cannot convert type NOOB to YARN".to_string(),
                            token: token.clone(),
                        });
                        return (VariableValue::new(-1, Types::Noob), token);
                    }
                };
            }
            _ => panic!("Unexpected type"),
        }

        self.add_statements(expression.free());

        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);
        (VariableValue::new(hook, type_), token)
    }

    pub fn visit_it_reference(
        &mut self,
        it_ref: ast::ItReferenceNode,
    ) -> (VariableValue, ast::TokenNode) {
        let (hook, stmt) = self.get_hook();
        self.add_statements(vec![stmt]);

        let scope = self.get_scope();
        let variable = scope.get_variable("IT");
        if let None = variable {
            self.errors.push(VisitorError {
                message: "IT variable not declared".to_string(),
                token: it_ref.token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), it_ref.token);
        }
        let variable = variable.unwrap();

        if variable.value.type_.equals(&Types::Noob) {
            self.errors.push(VisitorError {
                message: "IT variable not initialized".to_string(),
                token: it_ref.token.clone(),
            });
            return (VariableValue::new(-1, Types::Noob), it_ref.token);
        }
        let (var, stmts) = variable.copy(hook);
        self.add_statements(stmts);

        (var, it_ref.token)
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
            "Word_YARN" => Types::Yarn(1),
            _ => panic!("Unexpected type"),
        };

        if type_.equals(&Types::Yarn(1)) {
            self.add_statements(vec![ir::IRStatement::Push(1.0), ir::IRStatement::Allocate]);
        } else {
            self.add_statements(vec![ir::IRStatement::Push(0.0)]);
        }

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

                self.add_statements(variable.unwrap().free());

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

                self.add_statements(variable.unwrap().free());

                let scope_mut = self.get_scope_mut();
                let variable_mut = scope_mut.get_variable_mut(&name).unwrap();
                let stmts = variable_mut.assign(&expression.type_);
                self.add_statements(stmts);
            }
        }
    }

    pub fn visit_visible_statement(&mut self, visible: ast::VisibleStatementNode) {
        let (expr, _) = self.visit_smoosh_expression(ast::SmooshExpressionNode {
            expressions: visible.expressions.clone(),
        });

        self.free_hook(expr.hook);

        match expr.type_ {
            Types::Yarn(size) => {
                self.add_statements(vec![
                    ir::IRStatement::RefHook(expr.hook),
                    ir::IRStatement::Copy,
                    ir::IRStatement::Push(size as f32),
                    ir::IRStatement::CallForeign("print_string".to_string()),
                ]);
            }
            _ => panic!("Unexpected type"),
        }

        if let None = visible.exclamation {
            self.add_statements(vec![ir::IRStatement::CallForeign("prend".to_string())]);
        }

        self.add_statements(expr.free());
        self.add_statements(vec![
            ir::IRStatement::BeginWhile,
            ir::IRStatement::Push(0.0),
            ir::IRStatement::EndWhile,
        ]);
    }

    pub fn visit_gimmeh_statement(&mut self, gimmeh: ast::GimmehStatementNode) {
        let token = gimmeh.identifier;
        let name = match token.value() {
            tokens::Token::Identifier(name) => name,
            _ => panic!("Expected Identifier token"),
        };

        let scope = self.get_scope();
        let variable = scope.get_variable(&name);
        if let None = variable {
            self.errors.push(VisitorError {
                message: format!("Variable {} not declared", name),
                token,
            });
            return;
        }

        let variable = variable.unwrap();

        if !variable.value.type_.equals(&Types::Yarn(-1)) {
            self.errors.push(VisitorError {
                message: format!("Variable {} is not of type YARN", name),
                token,
            });
            return;
        }

        self.add_statements(variable.free());

        self.add_statements(vec![ir::IRStatement::CallForeign(
            "read_string".to_string(),
        )]);

        let scope_mut = self.get_scope_mut();
        let variable_mut = scope_mut.get_variable_mut(&name).unwrap();
        let stmts = variable_mut.assign(&variable_mut.value.type_.clone());
        self.add_statements(stmts);
    }
}
