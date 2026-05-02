# Buyan

Buyan is a simple Forth-like language written in Rust.

# Hello world

[source](./examples/hello_world.by)

```
$ cargo run --bin cli -- --path ./examples/hello_world.by
Hello, world!
```

# Syntax example

```
import std.io;

module hello_world;

def main( -- )
    "Hello, world!"
    std.io.println
end
```
