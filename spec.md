Katlang is a very simple language, and for the most is executed exactly left-to-right one character at a time. Some constructs alter the order during parsing, which is reflected in the parsed code structure.

## Value types

- Integer: the simplest type, a plain 64-bit signed integer
- String: a UTF-8 encoded string
- Stack/list: a list of values. Called a stack because it can be converted to and from a stack frame.
- Function: a primitive function

## Grammar

```
program := stmt program
stmt := literal | command | block | variable
literal := string | number
string := <whitespace> | " ... " | 'x
number := [0-9]+ <whitespace>?
command := <ident of any primitive function>
block := ( program ) | [ program ] | { program } variable
variable := <any UTF-8 character>
```

## Parsing

The parser tries the following in order, for each character, applying the first match:

1. The character is whitespace
   - Treated as a string literal, so a space becomes `" "`
2. The character is a double quote
   - Reads the until the next quote as-is, respecting `\"` if you need to type a doublequote. Kat doesn't treat newlines specially, so you can write out anything in a string.
3. The character is a single quote
   - Reads the next character as a string literal.
4. The character is a digit `[0-9]`
   - Reads all following digits as a single integer. Consumes directly following whitespace, so that numbers can be separated easily (`10 20` pushes 10, then 20)
5. The character is a builtin command
   - Adds it to the program.
   - Commands may impose special parsing rules, which are explained for each command separately.
6. The character starts a block
   - `(...)`: Executes the contents in a separate context and collects them to a list. Eg. `(1 2 3)` creates a list `[1 2 3]`
   - `[...]`: Collects the contained commands as a list (aka a block). Does not execute the contents like the previous type. Used for defining unnamed functions.
   - `{...}v`: Same as previous, except also assigns it to the variable `v` (which can be any UTF-8 character). The value is _not_ preserved on the stack. The variable is marked as known.
7. The character is an unknown variable
   - Reads following code until a `}`. Assigns the block defined by that code to the variable _at the start of the program_. The point of definition fetches the variable, but does not execute it. The variable is marked as known.
8. The character is a known variable
   - Reads the variable. If it's a function or a block, executes it. If not, pushes it to the stack.

### Implicit block

Some commands may impose a rule for an implicit block. This means that the command is followed by a block of code, terminated with a `$`. The `$` is optional if the current block ends anyways. If a command that expects an implicit block is immediately followed by a `$`, no block is created.

Eg. `(1 2 3)M1+$` == `(1 2 3)[1+]M$`

## Commands

Whenever a function is mentioned, it can mean either a quoted builtin or a block (a list of builtins and blocks).

- `` ` `` (backtick) (CreateCommand): quotes the builtin directly after this command.
  - Eg. `` `+ `` pushes the actual function``+` to the stack, instead of executing it
- `+` (Add): pops two values and pushes their sum on the stack.
  - If either value is a list, it's looped over.
  - If either value is a string, it's concatenated with the other value (which is coerced to a string implicitly).
  - If both values are integers, they're summed.
  - Eg. `(1 2 3)1+` -> `(2 3 4)`
  - Eg. `"hi"1+` -> `"hi1"`
  - Eg. `20 31+` -> `51`
- `*` (Multiply): Pops two values and multiplies them. Errors if one of the values isn't an integer.
- `R` (ReadLine): Reads a line from stdin (without the newline) and pushes it to the stack. Errors on EOF.
- `W` (WriteLine): Pops a value, coerces it to a string and writes it to stdout (with a following newline).
- `w` (Write): Pops a value, coerces it to a string and writes it to stdout (without a following newline).
- `M` (Map): Pops a function, and then a value. The value must be a list. Applies the function to each item separately, collecting the top item of the stack after each iteration.
  - Parsing rule: implicit block
  - Eg. `(1 2 3)M1+2*` -> `(4 6 8)`
- `F` (ForEach): Same as map, but does not collect the values.
  - Parsing rule: implicit block
  - Eg. `(1 2 3)FW` -> empty stack, but prints each value on separate lines
  - Eg. `0(1 2 3)F+` -> `6`
- `#` (Repeat): Pops a function and a value. Coerces the value to an integer and repeats the function that many times.
  - Parsing rule: implicit block
  - Eg: `1 10#2*` -> `1024`
- `!` (Execute): pops a function and executes it
- `S` (Split): Pops a separator (string) and a string. Splits the string with the separator and collects the pieces to a list.
- `I` (ToInteger): Pops a value and coerces it to an integer.
- `r` (Range): Pops a number n. Produces a list `(1..n)` (inclusive).
- `:` (Duplicate): Duplicates the top element.
- `;` (DuplicateSecond): Duplicates the second element, placing the result below the top item.
  - Eg. `2 3;` -> `2 2 3`
- `_` (Drop): Drops the top item.
- `x` (Rotate(2)): Swaps the top 2 items.
- `X` (Rotate(3)): Rotates the top 3 items.
  - Eg. `1 2 3X` -> `3 1 2`
- `p` (PushSide): Pushes the top value to the side stack. Does not consume the value.
- `P` (PopSide): Pops the top item from the side stack.
- `~` (ConsumeSide): Consumes the entire side stack and pushes it as a list to the main stack.
- `J` (Join): Pops a separator and a list. Coerces each item of the list to strings and joins them using the separator.
- `>` (PushVariable): Pops the top item and writes it to the variable.
  - Parsing rule: reads the next character as the variable name.
  - Eg. `10>x` writes `10` to the variable `x`.
- `<` (PopVariable): Read the value of the variable and push it to the stack. Does not consume the variable.
  - Parsing rule: reads the next character as the variable name.
  - Eg. `10>x<x` leaves the stack with `10`, and still has `10` in the variable `x`.
