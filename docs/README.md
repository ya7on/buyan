# Buyan Documentation

Buyan is a compiled, [strictly statically typed](https://en.wikipedia.org/wiki/Type_safety), [stack-oriented](https://en.wikipedia.org/wiki/Stack-oriented_programming), and [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) programming language. It is inspired by [Forth](https://en.wikipedia.org/wiki/Forth_(programming_language)).

You can learn more about concatenative programming languages [here](https://concatenative.org/).

# Installation

To install Buyan, clone the repository and build it using [Cargo](https://doc.rust-lang.org/cargo/):

```
git clone https://github.com/ya7on/buyan.git
cd buyan
cargo build --release
```

After building, you can find the executable in `target/release/buyan`.

# Usage

To run a Buyan program, simply pass the file path to the `buyan` executable:

```
./target/release/buyan --path <path>.by --target <target>
```

Where `<target>` is the target architecture to compile for.

# Target Architectures

At the moment, Buyan supports the following target architectures:

- `interpreter` - instantly executes the program using an interpreter
- `z80-unknown-cpm` - compiles the program for the Z80 CP/M target, printing the assembled code to stdout
