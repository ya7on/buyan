# Buyan

Buyan is a small, statically typed, stack-oriented language inspired by Forth.

## Install

```sh
git clone https://github.com/ya7on/buyan.git
cd buyan
cargo build --release
```

The executable will be at `target/release/buyan`.

## Run

Run a program with the interpreter:

```sh
./target/release/buyan --path examples/hello_world.by
```

Generate Z80 CP/M assembly:

```sh
./target/release/buyan --path examples/hello_world.by --target z80-unknown-cpm > hello_world.asm
```

## Examples

- [Hello world](examples/hello_world.by)
- [Condition](examples/condition.by)
- [Loop](examples/loop.by)
- [Struct](examples/struct.by)

## Documentation

See the [documentation](docs/README.md) and the [tutorial](docs/tutorial.md).
