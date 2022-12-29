# Types
At a type level, Hoyle can be seen as a Hindley-Milner variant with extensions for structurally typed records and polymorphic variants. In addition, Hoyle plans to eventually support a system for polymorphic constraints similar to Haskell's type classes or Rust's traits.

## Polymorphism
To simplify the handling of generics, Hoyle only allows generics to be introduced by top level function definitions, not closures or let bindings.

## Existential Types
Although the details for Hoyle’s type class system have yet to be completed, one key detail is that Hoyle plans to support existential type class objects, similar to Rust’s dynamic trait objects.

