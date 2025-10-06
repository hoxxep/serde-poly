use crate::{DeserializePoly, SerializePoly};

macro_rules! impl_poly_owned {
    ($name:ty) => {
        impl DeserializePoly for $name {
            type Out<'de> = Self;
        }

        impl SerializePoly for $name {
            type Out = Self;
        }
    };

    ($generic:ident, $name:ty) => {
        impl<$generic> DeserializePoly for $name
        where
            $name: for <'de> serde::Deserialize<'de>,
        {
            type Out<'de> = Self;
        }

        impl<$generic> SerializePoly for $name
        where
            $name: serde::Serialize,
        {
            type Out = Self;
        }
    };
}

macro_rules! impl_poly_borrowed {
    ($name:ty, $poly:ident) => {
        pub struct $poly {}

        impl DeserializePoly for $poly {
            type Out<'de> = $name;
        }

        impl<'de> SerializePoly for $name {
            type Out = $poly;
        }
    };

    ($generic:ident, $name:ty, $named:ty, $poly:ident) => {
        pub struct $poly<$generic>(core::marker::PhantomData<$generic>);

        impl<'d, $generic> DeserializePoly for $poly<$generic>
        where
            $named: serde::Deserialize<'d>,
            $generic: 'd,
        {
            type Out<'de> = $name;
        }

        impl<'d, $generic> SerializePoly for $named where $named: serde::Serialize {
            type Out = $poly<$generic>;
        }
    };
}

impl_poly_owned!(String);
impl_poly_owned!(bool);
impl_poly_owned!(char);
impl_poly_owned!(u8);
impl_poly_owned!(u16);
impl_poly_owned!(u32);
impl_poly_owned!(u64);
impl_poly_owned!(u128);
impl_poly_owned!(usize);
impl_poly_owned!(i8);
impl_poly_owned!(i16);
impl_poly_owned!(i32);
impl_poly_owned!(i64);
impl_poly_owned!(i128);
impl_poly_owned!(isize);
impl_poly_owned!(f32);
impl_poly_owned!(f64);
impl_poly_owned!(T, Vec<T>);

impl_poly_borrowed!(&'de str, StrPoly);

#[cfg(feature = "uuid")]
impl_poly_owned!(uuid::Uuid);
