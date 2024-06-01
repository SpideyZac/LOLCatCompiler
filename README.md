# LOLCatCompiler

A simple & performant LOLCode compiler written in Rust.

# Compilation Modes

## C Virtual Machine
LOLCatCompiler comes with a built in C Runtime to allow it to be ran on any machine. Optionally, you can compile to assembly, but not many targets are supported so you will have to implement any new ones yourself.

### Dependencies
An optional cc arg can be passed to the program to specify the c compiler to use. By default, LOLCatCompiler will look for a `dep` folder located in the same directory as the executable and will look for a `tcc` or tiny c compiler folder inside.
If the tiny c compiler is not found, it will default to use `gcc`.

### Compilation Steps

* Lex & Parse the input script
* Generate IR code for the AST
* Create a temp c file containing all of the VM instructions
* Import the std.c and core.c
* Compile the script using the `dependencies`

# IR (Intermediate Representation)
LOLCatCompiler's IR takes inspiration from [oakc's intermediate representation](https://github.com/adam-mcdaniel/oakc?tab=readme-ov-file#intermediate-representation)

In fact, we have only added 3 instructions (and changed some) for compatability with assembly! (**MAY CHANGE**)
| Instruction | Side Effect |
|-|-|
| `push(n: f32);` | Push a number onto the stack. |
| `add();` | Pop two numbers off of the stack, and push their sum. |
| `subtract();` | Pop two numbers off of the stack. Subtract the first from the second, and push the result. |
| `multiply();` | Pop two numbers off of the stack, and push their product. |
| `divide();` | Pop two numbers off of the stack. Divide the second by the first, and push the result. |
| `sign();` | Pop a number off of the stack. If it is greater or equal to zero, push `1`, otherwise push `-1`. |
| `allocate();` | Pop a number off of the stack, and return a pointer to that number of free bytes on the heap. |
| `free();` | Pop a number off of the stack, and go to where this number points in the heap. Pop another number off of the stack, and free that many cells at this location in the heap. |
| `store(size: i32);` | Pop a number off of the stack, and go to where this number points in the heap. Then, pop `size` numbers off of the stack. Store these numbers in reverse order at this location in the heap. |
| `load(size: i32);` | Pop a number off of the stack, and go to where this number points in the heap. Then, push `size` number of consecutive memory cells onto the stack. |
| `copy();` | Pop a number off of the stack, and go to where base_ptr - this number points in the stack. Then push the value in the stack back onto the stack again.  |
| `call(fn: i32);` | Call a user defined function by it's compiler assigned ID. |
| `call_foreign_fn(name: String);` | Call a foreign function by its name in source. |
| `begin_while();` | Start a while loop. For each iteration, pop a number off of the stack. If the number is not zero, continue the loop. |
| `end_while();` | Mark the end of a while loop. |
| `load_base_ptr();` | Load the base pointer of the established stack frame, which is always less than or equal to the stack pointer. |
| `establish_stack_frame();` | Calls `load_base_ptr` and sets the base_ptr to the current stack address |
| `end_stack_frame(arg_size: i32, local_scope_size: i32);` | Pop `local_scope_size` numbers off of the stack. Then, restore the base_ptr by popping another number off the stack. Next, pop the return address (next instruction address) of off the stack. Finally, pop `arg_size` numbers off of the stack. |
| `set_return_register();` | Pop a number off of the stack, and set the return register to its value |
| `access_return_register();` | Push return register's value to the stack |

Here is how the base_ptr works:

```
; ebp = base_pointer
; ebp + 3 = arg2
; ebp + 2 = arg1
; ebp + 1 = next address (after func return)
; ebp     = previous base pointer
; ebp - 1 = local_variable_1
; ebp - 2 = local_variable_2
```

Example Program (which adds 1 + 2)

```rust
use compiler::target::Target;

use crate::compiler::ir::{IRFunction, IRFunctionEntry, IRStatement, IR};
use crate::compiler::target::vm::VM;

fn main() {
    let t = VM {}; // Create an instance of the C Virtual Machine for the IR

    let ir = IR::new(
        vec![IRFunction::new(
            "sum".to_string(), // Creates a function called "sum"
            vec![
                IRStatement::EstablishStackFrame,
                IRStatement::Push(2.0),
                IRStatement::Copy, // Copies arg1 (num1) to the front of the stack (base_ptr + 2) (recall the structure for base_ptr)
                IRStatement::Push(3.0),
                IRStatement::Copy, // Copies arg2 (num2) to the front of the stack (base_ptr + 3)
                IRStatement::Add, // Add num1 and num2 together
                IRStatement::SetReturnRegister, // Set the value of the return register to equal the result of num1 + num2
                // The following section is to display heap capabilities (not necessary)
                IRStatement::AccessReturnRegister, // Copies the value in the return register to the front of the stack
                IRStatement::Push(4.0),
                IRStatement::Copy, // Copies arg3 (heap_ptr) to the front of the stack (base_ptr + 4)
                IRStatement::Store(1), // Store 1 float (return register access) to heap_ptr
                IRStatement::EndStackFrame(2, 0), // All local variables are cleared/eaten, hence the 0, and we do not want to delete the heap_ptr from the stack as it is used later in main
            ],
        )],
        IRFunctionEntry::new(
            100, // stack_size (400 bytes)
            400, // heap_size (400 bytes)
            vec![
                // Establish stack frame is automatically called
                IRStatement::Push(4.0), // Store the size (in bytes) of this allocation for later use
                IRStatement::Push(4.0), // We do the same thing, but Allocate will eat this value
                IRStatement::Allocate, // Push a heap_ptr where 4 bytes are allocated (acts as arg3 for sum)
                IRStatement::Push(2.0), // arg2 for sum
                IRStatement::Push(1.0), // arg1 for sum
                IRStatement::Call("sum".to_string()), // call sum
                IRStatement::AccessReturnRegister, // add the return register's value to the stack
                IRStatement::CallForeign("prn".to_string()), // print its value (in number form)
                IRStatement::CallForeign("prend".to_string()), // print a new line
                IRStatement::Push(-2.0), // Access the first local variable (the heap_ptr - not cleared because sum only clears args 1 and 2)
                IRStatement::Copy, // Copies its value to the front of the stack (base_ptr - 2)
                IRStatement::Load(1), // Copies 1 number from heap[heap_ptr] to the stack
                IRStatement::CallForeign("prn".to_string()), // print its value (in number form)
                IRStatement::CallForeign("prend".to_string()), // print a new line
                IRStatement::Free, // Free the heap_ptr (the only 2 things left on the stack are the original arg3 and the duplicate 4.0 size in bytes) - frees the heap memory at heap_ptr with a size of 4.0 bytes
                // do not need to end the stack frame as the program is done anyways
            ],
        ),
    );

    let code = ir.assemble(&t); // assemble with the given target
    t.compile(code).expect("Failed to compile"); // then we compile it
}
```
