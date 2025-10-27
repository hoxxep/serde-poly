//! A derive macro for `#[derive(Poly)]` that implements:
//! - For types without lifetimes:
//!   - impl [`DeserializePoly`] for `Self` with `type Out<'de> = Self`
//!   - impl [`SerializePoly`] for `Self` with `type Out = Self`
//! - For types with lifetimes, such as `MyType<'a, T>`:
//!   - A tuple struct `MyTypePoly<T>(PhantomData<fn() -> T>)`, without lifetimes.
//!   - impl [`SerializePoly`] for `MyType<'a, T>` with `type Out = MyTypePoly<T>`
//!   - impl [`DeserializePoly`] for `MyTypePoly<T>` with `type Out<'de> = MyType<'de, T>`
//!
//! Supports `#[poly(name = "CustomName")]` attributes to customize the name of the
//! generated Poly type.
//!
//! For types with multiple lifetime parameters, the derive macro fails with a clear
//! error message.

mod expand_ownable_poly;
mod expand_poly;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    DeriveInput,
};
use crate::expand_poly::expand_poly;

#[proc_macro_derive(Poly, attributes(poly))]
pub fn derive_poly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_poly(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(OwnablePoly)]
pub fn derive_ownable_poly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_ownable_poly::expand_ownable_poly(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
