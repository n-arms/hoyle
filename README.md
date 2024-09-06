# hoyle
A high performance opinionated functional-adjacent programming language.

## project goals
- high performance functional programming
- syntax that will be familiar to JavaScript, C++, Rust users
- complete type inference in function bodies
- seperate compilation without uniform representation

## example program
```
func id[t](x: t): t = x
func four(): F64 = 4
func five(): F64 = four() + id(1)
func mul_by(x: F64): (F64) => F64 = a => a * x
func main(): F64 = {
  let f = mul_by(five());
  f(3)
}
```
This program makes use of generics (`id` is generic over the type `t`), closures (`mul_by` returns a function that multiplies its argument), and statement blocks (`main` looks a lot like code in an imperative language).

## compilation stages
The Hoyle compiler goes through a number of stages to compile programs.

### lexing
The program text is broken up into a series of chunks called tokens such as `=>`, `func`, `(`, and `mul_by`.

### parsing
A list of tokens are parsed into a tree representing the program. Hoyle uses parser combinators to do its parsing, which are essentially a nice wrapper around top-down recursive parsing.

### type inference
Hoyle has full type inference in function bodies. This means that while Hoyle is a fully statically typed language (like Java or C++), you never need to write the types of variables in function bodies, the compiler can always figure it out. You only ever need to write the types of function arguments and return values, similar to Rust.

Hoyle uses a bidirectional Hindley-Milner type inference algorithm with in place unification. This enables it to fully infer all types, while getting the kind of good error messages that bidirectional type checking enables.

### type passing
This is where dynamic type information is explicitly inserted into the program (only as necessary, see Hoyle's generic compilation strategy).

### lowering
The type-passing Hoyle program is lowered into an IR called `bridge`. `bridge` is a list of instructions suitable for compiling into C or further lowering to assembly. `bridge` programs don't have closures and contain explicit reference counting operations. 

### emitting
At the moment this is the end of the pipeline; a program written in the `bridge` IR is converted into C to be compiled by the user.

## memory management
Hoyle allocates very little dynamic memory: structs and enums are all stored on the stack and don't require heap allocations. When Hoyle does need dynamic memory (ie when dealing with closures), it uses reference counting to collect garbage. The end goal is for Hoyle to implement the [counting immutable beans](https://arxiv.org/pdf/1908.05647) paper's approach to elliding reference counts, but as of right now Hoyle doesn't do this.

## generics
Hoyle uses an unusual scheme to compile generics. It allows Hoyle to have the same in-memory representation of data as a monomorphized language, but maintain seperate compilation. The Swift language uses a similar approach.

Hoyle accomplishes this by passing a piece of auxillary type information around at runtime, called a witness table. Wherever the types of a program are fully static (ie there are no generics) no witness tables are generated. When you do call generic functions, the Hoyle compiler extends the function arguments to also pass the witness table for the generics types of your function. For example the compiler transforms
```
func id[t](x: t): t = x
func main(): F64 = id(3)
```
into
```
func id(x: t, t: WitnessTable, result: t) = move x into result using witness table t
```
The `x` parameter is the same as in the original program, but two new arguments have been added. The `t` parameter stores the witness table that describes how to deal with a value of type `t`. It contains the size of a value of type `t`, as well as information about how to copy, move, and destroy it. Finally, the `result` parameter is used to hold the return value of the function.
