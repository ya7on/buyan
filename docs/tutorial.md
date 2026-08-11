# Tutorial

## What Is a Stack-Oriented Programming Language?

A **stack** is a data structure that follows the LIFO rule: Last In, First Out.

A **stack-oriented programming language** is a language that uses a stack to store data and perform operations.

Buyan is a stack-oriented language. Every operation works with values stored on the stack.

Let us look at a simple program step by step. It adds two numbers and leaves the result on the stack. This example uses pseudocode:

```
2 2 +
```

There are three stack operations:

- `2` puts the number 2 on the stack. The stack is now `[2]`.
- `2` puts another number 2 on the stack. The stack is now `[2, 2]`.
- `+` takes the top two numbers, adds them, and puts the result on the stack. The stack is now `[4]`.

## Stack Signature

In Buyan, a stack signature describes the types on the stack. Every word describes two stack states: the state it expects before it runs and the state it produces after it runs.

Here is a word that adds two numbers:

```
def add(usize, usize -- usize) ... end
```

The `--` separates the two stack states. The signature `usize, usize -- usize` means that the word takes two `usize` values from the top of the stack and leaves one `usize` value on top.

## Stack Polymorphism

Buyan supports stack polymorphism. It lets one word work with different types or different stack states. There are two syntax features for this:

- **Type variable.** A type variable lets a stack signature use different types. Add the variable to the word declaration. For example: `def example<T>(T, T -- T)`. This word takes two values of type `T` and leaves one value of type `T`. If you call it with `usize` values, it leaves a `usize`. If you call it with `u8` values, it leaves a `u8`.
- **Stack variable.** A stack variable represents a stack state. Add it to the word declaration. For example: `def example<...T>(...T, usize -- ...T)`. This word takes a stack described by `...T` with a `usize` value on top. It removes the `usize` and leaves the rest of the stack unchanged.

## Structure of a Buyan Program

A Buyan program can contain:

- **Imports** — words or structures from other modules.
- **A module name** — every module has a unique name.
- **Structures** — custom data types.
- **Words** — operations similar to functions in other programming languages.

## Control Flow

Control flow uses special words. Like everything else in Buyan, these words work through the stack.

### Conditions

Conditions in Buyan work like conditions in other programming languages. They run different code depending on a Boolean value.

The word `std.cfg.if` expects these values on the stack:

- A `bool` condition.
- A lambda to run when the condition is true.
- A lambda to run when the condition is false.

Example:

```
2u8 2u8 std.u8.add
4u8 std.u8.eq

| -- | { then }
| -- | { else }

std.cfg.if
```

First, the program adds `2 + 2` and puts `4u8` on the stack. It then compares this value with `4u8` and puts the result on the stack. In this example, the result is `true`.

Next, it puts the `then` lambda on the stack. This lambda runs when the condition is true. It then puts the `else` lambda on the stack. This lambda runs when the condition is false.

After `std.cfg.if` runs, the stack contains the result of the `then` or `else` lambda.

Important: the `then` and `else` lambdas must produce the same stack state.

### Loops

The word `std.cfg.while` creates a loop. It expects these values on the stack:

- A condition lambda. It leaves a `bool` on top of the stack and runs before every loop iteration.
- A body lambda. It runs on every iteration while the condition is true.

Here is a loop that runs 10 times:

```
0u8

| u8 -- u8, bool | {
  std.stack.dup 10u8 std.u8.lt
}
| u8 -- u8 | {
  1u8 std.stack.add
}
std.cfg.while
```

First, the program puts `0u8` on the stack. This value is the counter.

Next, it puts a lambda on the stack. This lambda checks whether the counter is less than 10. It returns `true` when the counter is less than 10 and `false` otherwise.

It then puts another lambda on the stack. This lambda adds 1 to the counter and returns the new value.

The loop runs 10 times and leaves `10u8` on the stack.

# Data Types

## Integers

Buyan has several unsigned integer types:

- `usize` — an unsigned integer. Its size depends on the target platform.
- `u8` — an 8-bit unsigned integer.
- `u16` — a 16-bit unsigned integer.

You can put integer values on the stack with this syntax:

- `42` puts the `usize` value `42` on the stack.
- `0x2A` puts the hexadecimal `usize` value `42` on the stack.
- `42u8` puts the `u8` value `42` on the stack.
- `0x2Au8` puts the hexadecimal `u8` value `42` on the stack.
- `'*'` puts the character `*` on the stack as the `u8` value `42`.
- `42u16` puts the `u16` value `42` on the stack.
- `0x2Au16` puts the hexadecimal `u16` value `42` on the stack.

You can learn more about each type in the standard library documentation:

- [u8](stdlib/u8.md)
- [u16](stdlib/u16.md)
- [usize](stdlib/usize.md)

## Str

The `Str` type stores a sequence of characters. At this time, strings can contain only ASCII characters.

Use this syntax to put a string on the stack:

```
"Hello, World!"
```

When you create a string, the compiler reserves space in static memory. It stores the string as a packed array of ASCII bytes with the string length at the beginning. The string stays in memory until the program ends.

You can learn more in the standard library documentation for [Str](stdlib/str.md).

## Structures

Structures are custom data types with one or more fields. They group stack values into one logical value.

Use this syntax to define a structure:

```
struct Point(u8, u8);
```

Use `Point<` to pack values into a structure. The required field values must be on top of the stack. A `Point` needs two `u8` values. Its pack operation has this signature: `u8, u8 -- Point`.

Use `Point>` to unpack a structure. It removes the structure from the stack and puts its fields on the stack. Its signature is `Point -- u8, u8`.

Use `Point.field` to get a structure field. For example, use `Point.0` to get the first field of `Point`. This operation removes the `Point` structure from the stack and puts its first field on top. Its signature is `Point -- u8`.

## Lambda

A lambda is an anonymous word. You can pass it to another word or return it from another word.

Use this syntax to define a lambda:

```
| A -- B | { ops }
```

Here, `A` is the stack state before the lambda runs, `B` is the stack state after it runs, and `ops` is the list of operations in the lambda body.

A lambda signature looks like this:

```
| A -- B |
```

It looks like a lambda declaration, but it has no body.
