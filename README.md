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

(MORE DOCS SOON)
