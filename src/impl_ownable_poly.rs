use crate::OwnablePoly;
use std::borrow::Cow;

macro_rules! impl_ownable_poly_primitive {
    ($($t:ty),*) => {
        $(
            impl OwnablePoly for $t {
                type Owned = Self;
                fn into_owned(self) -> Self::Owned {
                    self
                }
            }
        )*
    };
}

impl_ownable_poly_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
impl_ownable_poly_primitive!(char, String);
impl_ownable_poly_primitive!(bool);

impl<T: OwnablePoly> OwnablePoly for Vec<T> {
    type Owned = Vec<T::Owned>;
    fn into_owned(self) -> Self::Owned {
        self.into_iter().map(|x| x.into_owned()).collect()
    }
}

impl<T: OwnablePoly> OwnablePoly for Option<T> {
    type Owned = Option<T::Owned>;
    fn into_owned(self) -> Self::Owned {
        self.map(|x| x.into_owned())
    }
}

impl<T: OwnablePoly, E: OwnablePoly> OwnablePoly for Result<T, E> {
    type Owned = Result<T::Owned, E::Owned>;
    fn into_owned(self) -> Self::Owned {
        match self {
            Ok(v) => Ok(v.into_owned()),
            Err(e) => Err(e.into_owned()),
        }
    }
}

impl<'a, B> OwnablePoly for Cow<'a, B>
where
    B: ToOwned + ?Sized,
    B::Owned: Clone,
    Cow<'static, B>: 'static,
{
    type Owned = Cow<'static, B>;

    fn into_owned(self) -> <Self as OwnablePoly>::Owned {
        Cow::Owned(self.into_owned())
    }
}

#[cfg(feature = "uuid")]
impl_ownable_poly_primitive!(uuid::Uuid);
