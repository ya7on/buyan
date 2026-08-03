# Common Concepts

## Program Structure

- **Imports** make words and structs from another module available.
- **Module name** identifies the current module.
- **Structs** declare data types.
- **Words** define executable operations.

```buyan
import std.io;
import std.str;

module app;

struct Person(std.str.Str, u8);

def main( -- )
    "Semyon" 67u8 >Person
    Person.0
    std.io.println
end
```

## Stack

Buyan uses a last-in, first-out stack. Values are pushed onto the top; words consume their inputs from the top and push their outputs back.

## Data Types

Buyan has the following built-in data types:

- **Integers** are represented by `u8` and `u16`. In `1u8`, `1` is the value and `u8` is its type.
- **Strings** are text values written in quotes, such as `"Hello".
- **Booleans** are `bool` values representing `true` or `false`.

## Typing

Stack effects are described with concrete types, type variables, or stack variables.

- A concrete type, such as `u8`, describes an exact value type.
- A type variable, such as `A`, describes one generic type.
- A stack variable, such as `...S`, describes a generic sequence of stack values.

## Lambdas

Lambdas are anonymous words stored as values on the stack. Their stack effect is written between `|` characters, followed by their body in braces.

```buyan
| -- std.str.Str| { "Hello" }
```

## Words

Words are the unit used to store code. A word accepts and returns a stack in the form described by its `( -- )` stack effect. Calling a word requires the stack shape on the left and leaves the stack shape on the right.

```buyan
def one( -- u8)
    1u8
end
```

## Structs

Structs are containers for multiple values stored under one name. Values are packed with `>StructName`, unpacked with `StructName>`, and a field is selected from a struct on the stack with `StructName.0`.

```buyan
struct Person(std.str.Str, u8);
```
