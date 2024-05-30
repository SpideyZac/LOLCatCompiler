use crate::compiler::target::Target;

pub enum IRStatement {
    Push(f32),
    Add,
    Subtract,
    Multiply,
    Divide,
    Sign,
    Allocate,
    Free,
    Store(i32),
    Load(i32),
    Copy,
    Call(String),
    CallForeign(String),
    BeginWhile,
    EndWhile,
    LoadBasePtr,
    EstablishStackFrame(i32),
    EndStackFrame(i32, i32),
    SetReturnRegister,
    AccessReturnRegister,
}

impl IRStatement {
    pub fn assemble(&self, target: &impl Target) -> String {
        match self {
            IRStatement::Push(n) => target.push(*n),
            IRStatement::Add => target.add(),
            IRStatement::Subtract => target.subtract(),
            IRStatement::Multiply => target.multiply(),
            IRStatement::Divide => target.divide(),
            IRStatement::Sign => target.sign(),
            IRStatement::Allocate => target.allocate(),
            IRStatement::Free => target.free(),
            IRStatement::Store(size) => target.store(*size),
            IRStatement::Load(size) => target.load(*size),
            IRStatement::Copy => target.f_copy(),
            IRStatement::Call(name) => target.call_fn(name.clone()),
            IRStatement::CallForeign(name) => target.call_foreign_fn(name.clone()),
            IRStatement::BeginWhile => target.begin_while(),
            IRStatement::EndWhile => target.end_while(),
            IRStatement::LoadBasePtr => target.load_base_ptr(),
            IRStatement::EstablishStackFrame(local_scope_size) => {
                target.establish_stack_frame(*local_scope_size)
            }
            IRStatement::EndStackFrame(arg_size, local_scope_size) => {
                target.end_stack_frame(*arg_size, *local_scope_size)
            }
            IRStatement::SetReturnRegister => target.set_return_register(),
            IRStatement::AccessReturnRegister => target.access_return_register(),
        }
    }
}

pub struct IRFunction {
    pub name: String,
    pub statements: Vec<IRStatement>,
}

impl IRFunction {
    pub fn new(name: String, statements: Vec<IRStatement>) -> Self {
        IRFunction { name, statements }
    }

    pub fn assemble(&self, target: &impl Target) -> String {
        let mut code = String::new();
        let mut body = String::new();

        for statement in self.statements.iter() {
            let assembly = statement.assemble(target);

            body.push_str(&assembly);
        }

        code.push_str(&target.fn_definition(self.name.clone(), body));

        code
    }
}

pub struct IRFunctionEntry {
    pub stack_size: i32,
    pub heap_size: i32,
    pub statements: Vec<IRStatement>,
}

impl IRFunctionEntry {
    pub fn new(stack_size: i32, heap_size: i32, statements: Vec<IRStatement>) -> Self {
        IRFunctionEntry {
            stack_size,
            heap_size,
            statements,
        }
    }

    pub fn assemble(&self, target: &impl Target) -> String {
        let mut code = String::new();
        let mut body = String::new();

        for statement in self.statements.iter() {
            let assembly = statement.assemble(target);

            body.push_str(&assembly);
        }

        code.push_str(&target.begin_entry_point(self.stack_size, self.heap_size));
        code.push_str(&body);
        code.push_str(&target.end_entry_point());

        code
    }
}

pub struct IR {
    pub functions: Vec<IRFunction>,
    pub entry: IRFunctionEntry,
}

impl IR {
    pub fn new(functions: Vec<IRFunction>, entry: IRFunctionEntry) -> Self {
        IR { functions, entry }
    }

    pub fn assemble(&self, target: &impl Target) -> String {
        let mut code = String::new();

        for function in self.functions.iter() {
            let assembly = function.assemble(target);

            code.push_str(&assembly);
        }

        let entry = self.entry.assemble(target);

        code.push_str(&entry);

        code
    }
}
