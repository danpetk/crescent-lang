# Crescent

Crescent is a compiled programming language with a hand-written compiler that targets x86-64 assembly.

<img src="/assets/demo.png" width="600" alt="Image demonstrating an example program"/>

## Features

- Compiles to native x86-64 assembly (System V ABI)

- Hand-written lexer, parser, semantic analyzer, and code generation

- Variables, functions, loops, conditionals, formatted printing

- Informative error messages

- Statically typed

- No dependencies

## About

I was inspired to write a compiler after I took a compilers class at university and ended up really liking the content. We compiled to WebAssembly in that class, so I wanted to write my own compiler to target a more traditional assembly language. I finished reading the [Rust handbook](https://doc.rust-lang.org/book/) (which I often read during lectures :) ) at around the same time so I decided to choose this compiler as my first big Rust project.

I wanted to write everything from scratch, so there are no dependencies for this project. I did, however, take some inspiration from other compilers to model some of my structures, especially [rustc](https://github.com/rust-lang/rust/tree/main) since its also written in Rust.

## Development & Challenges 

I am actually pretty satisfied with how the compiler frontend came out. I feel like I modeled everything well and there wasn't much friction when developing. The backend, however, is a different story. After finishing semantic analysis, I was tired that after 3 phases of compilation I still didn't have any code generation. I decided to go straight from the analyzed AST to code generation, skipping a linear IR. This worked well enough with the interpreters and the WebAssembly compilers that i've written before.

This was not a good idea. Problems like register allocation that are solved much more naturally with a pass over some IR in [SSA form](https://en.wikipedia.org/wiki/Static_single-assignment_form) instead involved more cumbersome and error-prone code. I also commmited to doing codegen in one pass, which made emitting stuff like stack offsets where the whole function needs to be known beforehand especially awkward. I ended up storing offset information in the symbol table, which works since I am only targeting one architecture, but is not ideal.

In hindsight, I learned a lot but I also made some mistakes. If I return to this project, I will rewrite the backend to use a proper IR. 

The compiler still works as is right now! 

## Disclaimers

- The output assembly targets x86-64 (System V ABI), so only machines that support it can run the compiled output. Some supported machines are x86 Linux and macOS (Intel) machines. Native Windows is not supported.
- Generated assembly source files must be linked against libc to run properly.

## Prerequisites

- Rust 1.90 (to build the compiler)
- libc (to link compiled assembly files against)

## Building

Follow the instructions to clone and build the compiler executable.

```bash
git clone https://github.com/danpetk/crescent-lang
cd crescent-lang
cargo build --release
```
When completed, the compiler executable will be at `crescent-lang/target/release/crescent_lang`. Alternatively, doing 
```bash
cargo run
``` 
will build and run the executable in one go. Any command-line arguments after will be forwarded to the executable.

## Basic usage

To compile a source file, do 
```bash
./crescent_lang {file}
```
The extension I've chose is `.crsnt`. By default, the compiled assembly is placed in `out.s`. To choose the output file name, pass it in as the second argument.
For example, to compile `source.crsnt` into `result.s`, you would do 
```bash
./crescent_lang source.crsnt result.s
```
I've provided a few example programs in `crescent-lang/examples/` that you can try to compile.

To run the generated assembly, you will need to assemble and link it against libc into an executable. The easiest way to do this is to use a c compiler like `gcc` or `clang`. Run
```bash
gcc out.s
```
and run the final binary using
```bash
./a.out
```

I've provided a helper script `compile_and_run.sh` that compiles a source file, assembles it, and runs it in one go as well.

## Language Specs

The following is an overview of the language's features and syntax.

### Statements vs Expressions

Things in the language are categorized into statements and expressions. Statements do things, while expressions produce values. This should be very similar to languages like C++. An example of a statement is a `while` loop, while an example expression is `x + 3`. 

### Function Declarations

All function declarations must be in the global scope. They are also the only things allowed in the global scope. An example function declaration is
```
func example(x : i64, y: i64) : i64 { ... }
```
Functions are declared with the `func` keyword, followed by an identifier and a paranthesized parameter list. Each element of the list is a parameter name followed by a colon and a type annotation. The only type in the language currently is a 64-bit signed integer, `i64` (see Development & Challenges above). After the list, a colon and a return type annotation is required. Finally, a statement block is required for the function body.

### Statement Blocks 

Statement blocks are enclosed in `{ }` and contain any number of statements separated by semicolons (not all statements require semicolons, like `if` statements and `while` loops). Each block creates a new scope.

### Variable bindings

Variable bindings are used to assign an identifier to a value that can be accessed and modified. An example variable binding is 
```
let x : i64 = 5;
```
Variables are declared with the `let` keyword, followed by an indentifier, colon, and a type annotation. After the equal sign, an expression is required to set the initial value of the varable. Shadowing is allowed for nested scopes, but not for two identifiers in the same scope. Both functions and variables share the same namespace.

### `if` statement

Identical to C-style `if` statements, except the condition does not need to be in parenthesis. The condition is truthy if it is not zero.
```
if cond { 
    ... 
} else if cond { 
    ... 
} else { 
    ... 
} 
```

### `while` statement

Identical to C-style `while` statements, except the condition does not need to be in parenthesis. Again the condition is truthy if it is not zero.
```
while cond {
    ...
} 
```

### `continue` & `break` statements

`continue` jumps to the loop condition check of the innermost loop. `break` jumps out of the innermost loop.

### `return` statement

`return` followed by an expression breaks out of the current function and yields the value to the caller (i.e. `return 5+6;`). Every possible branch of a function needs a `return` statement, otherwise it is undefinded behavior.

### `print` statement

Despite looking like a function call, a call to built-in `print` is a statement since it does not produce a value. An example usage of print is 
```
print("Hello, {}!\n", 3); // prints "Hello, 3!"
```
The first argument to `print` must be a string literal (this is the only place a string literal is valid). Inside the string literal, format placeholders `{}` can be used to specify where additional arguments will be inserted in the output. After the string literal, additional arguments can be provided to fill in those format placeholders. The number of placeholders and extra arguments must match exactly.

### Expressions

Expressions are pretty much identical to most C-style languages. At their simplest, they are integer literals (i.e. `135`) and variable references (i.e. `x`). You can combine values using arithmetic operators (`+`, `-`, `*`, `/`, `%`) and compare them with `==`, `!=`, `<`, `<=`, `>`, and `>=`. The unary operators `!` and `-` handle logical negation and numeric negation respectively. Functions can be called as expressions (i.e. `x + func(3,4)`), allowing their return values to be used in other expressions. The binary operator `=` is also an expression that updates an existing value (i.e. `x = 7`) and returns the value of the right expression.