pub mod vm;

pub trait Target {
    fn get_name(&self) -> char;
    fn is_standard(&self) -> bool;

    fn std(&self) -> String;
    fn core_prelude(&self) -> String;
    fn core_postlude(&self) -> String;

    fn begin_entry_point(&self, stack_size: i32, heap_size: i32) -> String;
    fn end_entry_point(&self) -> String;

    fn establish_stack_frame(&self) -> String;
    fn end_stack_frame(&self, arg_size: i32, local_scope_size: i32) -> String;
    fn set_return_register(&self) -> String;
    fn access_return_register(&self) -> String;
    fn load_base_ptr(&self) -> String;

    fn push(&self, n: f32) -> String;

    fn add(&self) -> String;
    fn subtract(&self) -> String;
    fn multiply(&self) -> String;
    fn divide(&self) -> String;
    fn modulo(&self) -> String;
    fn sign(&self) -> String;

    fn allocate(&self) -> String;
    fn free(&self) -> String;
    fn store(&self, floats: i32) -> String;
    fn load(&self, floats: i32) -> String;
    fn f_copy(&self) -> String;
    fn mov(&self) -> String;

    fn hook(&self, index: i32) -> String;
    fn ref_hook(&self, index: i32) -> String;

    fn fn_header(&self, name: String) -> String;
    fn fn_definition(&self, name: String, body: String) -> String;
    fn call_fn(&self, name: String) -> String;
    fn call_foreign_fn(&self, name: String) -> String;

    fn begin_while(&self) -> String;
    fn end_while(&self) -> String;

    fn halt(&self) -> String;

    fn compile(&self, code: String) -> std::io::Result<()>;
}
