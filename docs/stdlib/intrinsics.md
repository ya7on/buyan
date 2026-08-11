# `std.intrinsics`

# Structs

# Words

## add

**Type vars**: `T`

**Signature**

```buyan
T, T -- T
```

## sub

**Type vars**: `T`

**Signature**

```buyan
T, T -- T
```

## mul

**Type vars**: `T`

**Signature**

```buyan
T, T -- T
```

## div

**Type vars**: `T`

**Signature**

```buyan
T, T -- T
```

## eq

**Type vars**: `T`

**Signature**

```buyan
T, T -- bool
```

## gt

**Type vars**: `T`

**Signature**

```buyan
T, T -- bool
```

## lt

**Type vars**: `T`

**Signature**

```buyan
T, T -- bool
```

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

## u8_to_u16

**Signature**

```buyan
u8 -- u16
```

## u16_to_u8

**Signature**

```buyan
u16 -- u8
```

## u8_to_usize

**Signature**

```buyan
u8 -- usize
```

## u16_to_usize

**Signature**

```buyan
u16 -- usize
```

## usize_to_u8

**Signature**

```buyan
usize -- u8
```

## usize_to_u16

**Signature**

```buyan
usize -- u16
```

## offset

**Signature**

```buyan
ptr, u16 -- ptr
```

## load

**Signature**

```buyan
ptr -- u8
```

## store

**Signature**

```buyan
ptr, u8 --
```
