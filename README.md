# Buyan

Buyan is a small, statically typed, stack-oriented language inspired by Forth.

## Installation

To install Buyan using [Cargo](https://doc.rust-lang.org/cargo/), run:

```sh
cargo install --git https://github.com/ya7on/buyan.git
```

## Run

Run a program with the interpreter:

```sh
buyan examples/hello_world.by
```

Generate Z80 CP/M assembly:

```sh
buyan examples/hello_world.by --target z80-unknown-cpm > hello_world.asm
```

## Examples

- [Hello world](examples/hello_world.by)
- [Condition](examples/condition.by)
- [Loop](examples/loop.by)
- [Struct](examples/struct.by)

## Documentation

See the [documentation](https://ya7on.github.io/buyan/).
