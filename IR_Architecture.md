# IR Rework
As of 7d144c336c251616402e3ebc1f56693f69952ce7, Hoyle implements a subset of most of its relevant features.
Although it lacks closures, trailing lambda syntax, enums, match expressions and traits,
it is beginning to enter a state where adding new features isn't that difficult, but results in an increasingly high static complexity cost because of the poorly defined IRs.

The current major pain points are: accessing struct metadata, accessing witness tables, and reference counting.

## Accessing Struct Metadata
Struct metadata should be stored inline in the struct, not in auxiallary `StructBuilder`s.
This is trivial to solve, but does force us to paramaterize `Struct` with the compiler pass where we didn't before.
In general, it seems like a good rule to never store data for multiple passes in an external map,
but instead always add data directly to the IR if we want to later reuse it.

## Accessing Witness Tables
This is a lot more complicated.
At the `sized` IR, each variable and expression is annotated with a `Witness` object (either an expression making the table or a trivial table flag),
this seems like a reasonable solution until we consider the lowering process.
Every place a lowered expression needs a witness table, it explicitly states it (with some exceptions that should be resolved).
The question becomes do we generate a unique witness table for each type,
or do we generate a fresh witness table for each instruction that requires one.
Although the second option seems far inferior, the generated output of this compiler is going to heavily rely on CSE anyways, and this seems like a similar operation.
If we can find a way to turn the unique witness table approach into a duplicated witness table + CSE approach, that seems superior.
This operation would be simple if witness tables were still trivial to copy, but since they're not, matters become complicated.

If we have a function like
```
func f[T](x: Vec<T>) {
  id(x);
  id(x);
}
```
lowering will proceed like:
```
func f(x: Vec<T>, T: Type) {
  let _0 = Vec(T);
  let _1 = Vec(T);
  id(copy x, copy _0)
  id(copy x, copy _1)
}
```
CSE would turn this into:
```
func f(x: Vec<T>, T: Type) {
  let _0 = Vec(T);
  id(copy x, copy _0)
  id(copy x, copy _0)
}
```
which is indeed what we want. On the other hand, if we had 
```
func f(x: Vec<T>, T: Type) {
  let _0 = Vec(T);
  let _1 = Vec(T);
  id(copy x, move _0)
  id(move x, move _1)
}
```
which is the move semantically optimal transformation, the resulting CSE transformation would yield:
```
func f(x: Vec<T>, T: Type) {
  let _0 = Vec(T);
  id(copy x, move _0)
  id(move x, move _0)
}
```
which is clearly not correct.
Since the only places we use witness tables are passing them to functions or using them directly,
as long as we always start by generating `copy`s for witness table arguments, we should be able to apply CSE to great effect.

There is one more optimisation that we need to ensure that this witness table approach is compatible with: redundant move elimination (RME).
The core idea is that if a visibly constructed variable is
1. only used as a move into a function call
2. not directly used as a function argument (being moved is okay) and gets copied at some point into another variable
then it can be constructed in place in either the relevant function call slot or the relevant variable, and any moves or copies relating to it can be adjusted.

The only difference that applying CSE and merging redundant witness tables will cause is that fewer witness tables will be able to be constructed in place in function calls.
While this might seem unideal, since witness table construction can allocate heap memory, its definitely better to favour a couple copies over more heap allocations.

The approach of generating redundant witness tables seems to be successful!
It should be relatively easy to rework the `bridge` IR so that each (non trivial ie arithmetic) use of a variable is actually a `VariableUse`
```rust
enum VariableUse {
  Copy(Variable, Witness),
  Move(Variable, Witness)
}
```
to ensure that we always have access to appropriate witnesses and that we always have control over mvoe semantics.

## Reference Counting
Maybe I should actually read counting immutable beans or perseus :)
I'm fairly sure that difficulties with this are all related to my crummy implementation, and that adjusting the IR won't help, since it's all self-contained.
