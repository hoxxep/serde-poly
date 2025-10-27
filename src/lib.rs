mod impl_poly;
mod impl_ownable_poly;

use serde::Deserialize;
pub use serde_poly_macro::{OwnablePoly, Poly};

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

/// A type that can be converted from `Type<'a>` to `Type<'static>` by cloning or owning its data.
///
/// Mostly useful as a helper method for coercing types with lifetimes into their `'static`
/// variants, it _is not_ intended to otherwise change the type.
pub trait OwnablePoly {
    type Owned: OwnablePoly;

    fn into_owned(self) -> Self::Owned;
}

struct Example<'a> {
    data: std::borrow::Cow<'a, str>,
}

/// `#[derive(OwnablePoly)]` on `Example` should generate code similar to this.
impl<'a> OwnablePoly for Example<'a> {
    type Owned = Example<'static>;

    fn into_owned(self) -> Self::Owned {
        Example {
            data: std::borrow::Cow::Owned(self.data.into_owned()),
        }
    }
}
