mod impls;

use serde::Deserialize;
pub use serde_poly_macro::Poly;

/// A disjoint marker trait to hide the lifetimes of the deserializable types. All types must
/// implement this trait to be used as type parameters in the serialization wrappers.
pub trait DeserializePoly {
    type Out<'de>: serde::Deserialize<'de>;
}

/// A helper trait for types that implement [`DeserializePoly`], akin to [`serde::de::DeserializeOwned`].
pub trait DeserializePolyOwned:
    for<'de> DeserializePoly<Out<'de> = Self> + for<'de> Deserialize<'de>
{
}

impl<T> DeserializePolyOwned for T where
    T: 'static + for<'de> DeserializePoly<Out<'de> = Self> + for<'de> Deserialize<'de>
{
}

/// A disjoint marker trait to hide the lifetimes of the serializable types.
pub trait SerializePoly: serde::Serialize {
    type Out;
}

/// A disjoint marker trait for types that implement both [`DeserializePoly`] and [`SerializePoly`].
pub trait SerdePoly: DeserializePoly + SerializePoly {}
impl<T> SerdePoly for T where T: DeserializePoly + SerializePoly {}
