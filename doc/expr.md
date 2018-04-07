# Expressions

## Literals

- `123`, `0b111010`, `0o137`, `0x4bf86`  
Integer literals. Bit-widths are derived automatically from used
radix and digits (except decimal literals).

## Operators

The following operators are listed in the order of the lowest precedence
to the highest.

- `?`, `? :` Binary and Ternary Conditional
- `=` Assignment
- `@` Concatenation
- `||` Lazy Or
- `&&` Lazy And
- `==`, `!=`, `<`, `<=`, `>`, `>=` Relational
- `|` Binary Or
- `^` Binary Xor
- `&` Binary And
- `<<`, `>>` Binary Shifts
- `+`, `-` Addition and Subtraction
- `*`, `/`, `%` Multiplication, Division, and Modulo
- `[hi:lo]` Bit-slice
- `!`, `-` Unary Not and Unary Negation

## Blocks

You can evaluate multiple sub-expressions by using blocks. The last
sub-expression is returned. You can separate sub-expressions with
commas or linebreaks:

```c
{ x = 123, y = 456, x + y }
```

```c
{
    x = 123
    y = 456
    x + y
}
```

## Predefined Variables

- `pc`  
The address of the current instruction or the current expression in a
data directive.

## Predefined Functions

- `assert(condition)`  
Generates an error when `condition` is false. Useful to check for
the validity of instruction arguments, and also for instruction cascading.