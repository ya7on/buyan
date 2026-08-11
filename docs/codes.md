## B0001

**Unknown Error**

The compiler encountered an internal condition that does not have a more specific diagnostic.

**Example**

```text
[B0001] Error: Unknown Error
  Invalid module for word
```

**How to fix**

This normally indicates a compiler bug rather than an error in Buyan source code. Save the source that triggered the error and the complete compiler output, then report them at [the Buyan issue tracker](https://github.com/ya7on/buyan/issues).

```text
Include the smallest source file that reproduces the error.
```

## B0002

**File Not Found**

The entrypoint passed to the compiler does not exist or cannot be read.

**Example**

```console
$ cargo run --bin buyan -- --path ./missing.by
```

**How to fix**

Pass the path to an existing Buyan source file.

```console
$ cargo run --bin buyan -- --path ./examples/hello_world.by
```

## B0003

**Import Error**

An imported module could not be found. Standard-library imports must also name a supported module.

**Example**

```buyan
import missing;

module app;
```

**How to fix**

Create the imported module at the corresponding path, remove the import, or correct its name.

```buyan
import std.io;

module app;
```

## B0004

**Unexpected Token**

The lexer found a character or token that is not valid in Buyan source code.

**Example**

```buyan
module app;
!
```

**How to fix**

Remove the invalid token or replace it with valid Buyan syntax.

```buyan
module app;
```

## B0005

**Parse Error**

The tokens are individually valid, but they do not form a valid Buyan program.

**Example**

```buyan
module app;

def main( -- )
    1u8
```

**How to fix**

Complete the construct indicated by the diagnostic. This word is missing its closing `end`.

```buyan
module app;

def main( -- u8)
    1u8
end
```

## B0006

**Invalid Attribute**

The attribute attached to a word is not supported.

**Example**

```buyan
module app;

#[inline]
def main( -- u8) 1u8 end
```

**How to fix**

Remove the unsupported attribute.

```buyan
module app;

def main( -- u8) 1u8 end
```

## B0007

**Symbol Already Exists**

Two modules, words, structs, or generic variables resolve to the same symbol name.

**Example**

```buyan
module app;

def value( -- u8) 1u8 end
def value( -- u8) 2u8 end
```

**How to fix**

Give every symbol a unique name in its scope.

```buyan
module app;

def first( -- u8) 1u8 end
def second( -- u8) 2u8 end
```

## B0008

**Symbol Not Found**

The compiler could not resolve a referenced type, word, struct, or module.

**Example**

```buyan
module app;

def main(Missing -- Missing) end
```

**How to fix**

Correct the symbol name, define it, or import the module that contains it.

```buyan
module app;

def main( -- u8) 1u8 end
```

## B0009

**Invalid Symbol**

A known symbol was used where the compiler expected a different kind of symbol. If valid Buyan source triggers this error, it may indicate an internal compiler inconsistency.

**Example**

```text
[B0009] Error: Invalid Symbol
  symbol 'app.main' cannot be used here
```

**How to fix**

Check that calls refer to words and type positions refer to types or structs.

```text
Include the source file and complete compiler output in the issue.
```

## B0010

**Recursive Struct**

A struct contains itself directly or through another struct, so its size cannot be determined.

**Example**

```buyan
module app;

struct A(B);
struct B(A);
```

**How to fix**

Break the recursive chain by replacing one of the recursive fields with a non-recursive type.

```buyan
module app;

struct A(B);
struct B(u8);
```

## B0011

**Invalid Field Index**

A struct field access uses an index outside the struct's field range. Field indexes start at zero.

**Example**

```buyan
module app;

struct Pair(u8, u16);

def first(Pair -- u8) Pair.2 end
```

**How to fix**

Use an index that exists on the struct.

```buyan
module app;

struct Pair(u8, u16);

def first(Pair -- u8) Pair.0 end
```

## B0012

**Invalid Stack**

The current stack does not match the inputs required by an instruction or the output declared by a word.

**Example**

```buyan
import std.u8;
import std.str;

module app;

def main( -- u8)
    1u8
    "two"
    std.u8.add
end
```

**How to fix**

Make the produced values match the required stack types and order.

```buyan
import std.u8;

module app;

def main( -- u8)
    1u8
    2u8
    std.u8.add
end
```

## B0013

**Empty Word**

A non-builtin word has no instructions in its body. This is reported as a warning.

**Example**

```buyan
module app;

def main( -- ) end
```

**How to fix**

Add the intended implementation or remove the unused word.

```buyan
module app;

def main( -- u8)
    1u8
end
```

## B0014

**Unused Import**

A module is imported but none of its words or structs are used. This is reported as a warning.

**Example**

```buyan
import std.io;

module app;

def main( -- u8) 1u8 end
```

**How to fix**

Remove the import or use a symbol from the imported module.

```buyan
module app;

def main( -- u8) 1u8 end
```

## B0015

**Runtime Error**

The interpreter encountered an invalid runtime condition.

**How to fix**

Read the diagnostic message for the exact cause.

## B0016

**Cannot Infer Type**

The compiler could not infer all polymorphic parameters from the available inputs.

**Example**

```buyan
module app;

#[builtin]
def make<T>( -- T) end

def main( -- u8)
    make
end
```

**How to fix**

Provide an input or another type constraint from which the generic parameter can be inferred.

```buyan
module app;

#[builtin]
def identity<T>(T -- T) end

def main( -- u8)
    1u8
    identity
end
```

## B0017

**Invalid String Literal**

String literals may contain only ASCII characters.

**Example**

```buyan
import std.str;

module app;

def main( -- std.str.Str)
    "Привет"
end
```

**How to fix**

Replace non-ASCII characters with an ASCII representation.

```buyan
import std.str;

module app;

def main( -- std.str.Str)
    "Hello"
end
```

## B0018

**Data Overflow**

The data is too large.

**Example**

```buyan
import std.str;

module app;

def main( -- std.str.Str)
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
end
```

**How to fix**

Reduce or split the data that exceeded the limit.
