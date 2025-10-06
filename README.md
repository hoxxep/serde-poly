# serde-poly

Polymorphic lifetime traits for serde serialization and deserialization.

## Overview

`serde-poly` solves the problem of working with types that have lifetime parameters in generic contexts. When you have types like `MyType<'a, T>` that borrow data during deserialization, you need a way to express them in trait bounds without exposing the lifetime parameter.

This library provides marker traits `DeserializePoly` and `SerializePoly` that create a bidirectional mapping between lifetime-parameterized types and their lifetime-less "Poly" companions.

## Use Case

Consider a type like `EncryptedItem<'a, MyData<'a>>` where `MyData<'a>` uses zero-copy deserialization from borrowed bytes. Deriving `serde_poly::Poly` for `MyData<'a>` will:
- Implement `MyDataPoly` as a zero-sized companion/marker type.
- Implement the `DeserializePoly` trait for `MyDataPoly` that deserializes into `MyData<'de>`.
- Implement the `SerializePoly` trait for `MyData<'a>` that serializes from `MyDataPoly`.

EncryptedItem can now hold generic types `T: DeserializePoly + SerializePoly`, avoiding many lifetime issues in zerocopy contexts as inner types are slowly deserialized.

## Usage

### Owned types (types without lifetimes)

For types that don't borrow data, the derive macro implements both traits with `Self`:

```rust
use serde::{Deserialize, Serialize};
use serde_poly::Poly;

#[derive(Debug, Serialize, Deserialize, Poly)]
struct Owned {
    value: String,
}

// Implements:
// - DeserializePoly with Out<'de> = Self
// - SerializePoly with Out = Self
```

### Types with lifetimes

For types with a lifetime parameter, the macro generates a companion "Poly" type:

```rust
use serde::{Deserialize, Serialize};
use serde_poly::Poly;

#[derive(Debug, Serialize, Deserialize, Poly)]
struct Borrowed<'a> {
    value: &'a str,
}

// Generates:
// - struct BorrowedPoly(PhantomData<fn() -> ()>)
// - impl SerializePoly for Borrowed<'a> with Out = BorrowedPoly
// - impl DeserializePoly for BorrowedPoly with Out<'de> = Borrowed<'de>
```

### Custom Poly type names

You can customize the generated Poly type name:

```rust
#[derive(Debug, Serialize, Deserialize, Poly)]
#[poly(name = "MyCustomName")]
struct Borrowed<'a> {
    data: &'a str,
}

// Generates: struct MyCustomName(...)
```

## Traits

### `DeserializePoly`

Maps a type to its deserializable form with a generic lifetime:

```rust
pub trait DeserializePoly {
    type Out<'de>: serde::Deserialize<'de>;
}
```

### `SerializePoly`

Maps a type to its serializable form without lifetimes:

```rust
pub trait SerializePoly: serde::Serialize {
    type Out;
}
```

### `DeserializePolyOwned`

A helper trait for types that own their data (similar to `DeserializeOwned`):

```rust
pub trait DeserializePolyOwned:
    for<'de> DeserializePoly<Out<'de> = Self> + for<'de> Deserialize<'de>
{}
```

## License

Licensed under either of MIT license or Apache License Version 2.0, at your option.
