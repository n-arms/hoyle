# Types
At a type level, Hoyle can be seen as a Hindley-Milner variant with extensions for algebraic data types and a mechanism for polymorphism similar to Haskell’s type classes.

## Uniform Function Call Syntax
One of Hoyle’s goals is to act like a functional language, but still be very familiar to people who aren’t used to functional languages. An important part of this is UFCS. A common pattern in Hoyle is to use dot notation, enabling code like `map.get(key)`. In a traditional OOP language, such syntax requires the user to have defined a method `get` for items of type `Map`. Hoyle doesn’t have the same notion of a method, so `map.get(key)` is just syntactic sugar for `get(map, key)`.

## Protocols
Protocols are Hoyle’s tool for constrained polymorphism. They allow you to specify that your `print` function doesn’t just take lists of type `[a]` for any old `a`, rather `a` has to implement the `Stringable` protocol. A protocol is a lot like a trait in Rust, except protocols only contain functions that refer to the type the protocol is being implemented on using Self, not allowing for self, &self, &mut self.

## Existential Types
Hoyle supports dynamic dispatch. The type checker allows you to have types that resemble “any type that implements a given protocol” in addition to “this concrete type” or “this generic type”. This means you can have, for example, a list of values of different types that all implement the Stringable protocol.
