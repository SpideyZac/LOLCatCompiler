use super::Target;

use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    io::{Error, ErrorKind, Result, Write},
    process::{Command, Stdio},
};

pub struct VM;
impl Target for VM {
    fn get_name(&self) -> char {
        'c'
    }

    fn is_standard(&self) -> bool {
        true
    }

    fn std(&self) -> String {
        String::from(include_str!("std.c"))
    }

    fn core_prelude(&self) -> String {
        String::from(include_str!("core.c"))
    }

    fn core_postlude(&self) -> String {
        String::new()
    }

    fn begin_entry_point(&self, stack_size: i32, heap_size: i32) -> String {
        format!(
            "int main() {{\nmachine *vm = machine_new({}, {});\n",
            stack_size, heap_size,
        )
    }

    fn end_entry_point(&self) -> String {
        String::from("\nmachine_drop(vm);\nreturn 0;\n}")
    }

    fn establish_stack_frame(&self) -> String {
        format!("machine_establish_stack_frame(vm);\n")
    }

    fn end_stack_frame(&self, arg_size: i32, local_scope_size: i32) -> String {
        format!(
            "machine_end_stack_frame(vm, {}, {});\n",
            arg_size, local_scope_size
        )
    }

    fn set_return_register(&self) -> String {
        String::from("machine_set_return_register(vm);\n")
    }

    fn access_return_register(&self) -> String {
        String::from("machine_access_return_register(vm);\n")
    }

    fn load_base_ptr(&self) -> String {
        String::from("machine_load_base_ptr(vm);\n")
    }

    fn push(&self, n: f32) -> String {
        format!("machine_push(vm, {});\n", n)
    }

    fn add(&self) -> String {
        String::from("machine_add(vm);\n")
    }

    fn subtract(&self) -> String {
        String::from("machine_subtract(vm);\n")
    }

    fn multiply(&self) -> String {
        String::from("machine_multiply(vm);\n")
    }

    fn divide(&self) -> String {
        String::from("machine_divide(vm);\n")
    }

    fn modulo(&self) -> String {
        String::from("machine_modulo(vm);\n")
    }

    fn sign(&self) -> String {
        String::from("machine_sign(vm);\n")
    }

    fn allocate(&self) -> String {
        String::from("machine_allocate(vm);\n")
    }

    fn free(&self) -> String {
        String::from("machine_free(vm);\n")
    }

    fn store(&self, size: i32) -> String {
        format!("machine_store(vm, {});\n", size)
    }

    fn load(&self, size: i32) -> String {
        format!("machine_load(vm, {});\n", size)
    }

    fn f_copy(&self) -> String {
        String::from("machine_copy(vm);\n")
    }

    fn mov(&self) -> String {
        String::from("machine_mov(vm);\n")
    }

    fn hook(&self, index: i32) -> String {
        format!("machine_hook(vm, {});\n", index)
    }

    fn ref_hook(&self, index: i32) -> String {
        format!("machine_ref_hook(vm, {});\n", index)
    }

    fn fn_header(&self, name: String) -> String {
        format!("void {}(machine* vm);\n", name)
    }

    fn fn_definition(&self, name: String, body: String) -> String {
        format!("void {}(machine* vm) {{ {}}}\n", name, body)
    }

    fn call_fn(&self, name: String) -> String {
        format!("machine_push(vm, 1);\n{}(vm);\n", name) // we push 1 as a temp value for a return pointer
    }

    fn call_foreign_fn(&self, name: String) -> String {
        format!("{}(vm);\n", name)
    }

    fn begin_while(&self) -> String {
        String::from("while (machine_pop(vm)) {\n")
    }

    fn end_while(&self) -> String {
        String::from("}\n")
    }

    fn halt(&self) -> String {
        String::from("machine_halt(vm);\n")
    }

    fn compile(&self, code: String) -> Result<()> {
        let exe_path = current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        let deps_path = exe_dir.join("dep");
        let mut tcc_path = deps_path.join("tcc");
        let mut cc = "gcc";
        tcc_path = tcc_path.join(("tcc".to_string() + EXE_SUFFIX).as_str());
        if tcc_path.exists() {
            cc = tcc_path.to_str().unwrap();
        }

        let child = Command::new(cc)
            .arg("-O2")
            .args(&["-o", &format!("main{}", EXE_SUFFIX)[..]])
            .args(&["-x", "c", "-"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = child {
            match child.stdin.as_mut() {
                Some(stdin) => {
                    if let Err(_) = stdin.write_all(code.as_bytes()) {
                        return Result::Err(Error::new(
                            ErrorKind::Other,
                            "unable to open write to child stdin",
                        ));
                    }
                }
                None => {
                    return Result::Err(Error::new(ErrorKind::Other, "unable to open child stdin"))
                }
            }

            match child.wait_with_output() {
                Ok(_) => return Result::Ok(()),
                Err(_) => {
                    return Result::Err(Error::new(ErrorKind::Other, "unable to read child output"))
                }
            }
        } else {
            // child failed to execute
            Result::Err(Error::new(
                ErrorKind::Other,
                "unable to spawn child gcc proccess",
            ))
        }
    }
}
