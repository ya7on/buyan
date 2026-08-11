# `std.stack`

# Structs

# Words

## drop

**Type vars**: `T`

**Signature**

```buyan
T --
```

## dup

**Type vars**: `T`

**Signature**

```buyan
T -- T, T
```

## swap

**Type vars**: `A`, `B`

**Signature**

```buyan
A, B -- B, A
```

## over

**Type vars**: `A`, `B`

**Signature**

```buyan
A, B -- A, B, A
```

## rotate_left

**Type vars**: `A`, `B`, `C`

**Signature**

```buyan
A, B, C -- B, C, A
```

## rotate_right

**Type vars**: `A`, `B`, `C`

**Signature**

```buyan
A, B, C -- C, A, B
```

## call

**Stack vars**: `...S`, `...R`

**Signature**

```buyan
...S, |...S -- ...R| -- ...R
```
