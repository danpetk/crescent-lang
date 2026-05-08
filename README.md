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

# Disclaimers

- The output assembly targets x86-64 (System V ABI), so only machines that support it can run the compiled output. Some supported machines are x86 Linux and macOS (Intel) machines. Native Windows is not supported.
- Generated assembly source files must be linked against libc to run properly.

# Prerequisites

- Rust 1.90 (to build the compiler)
- libc (to link compiled assembly files against)

# Building

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

# Basic usage

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
