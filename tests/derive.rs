use serde::{Deserialize, Serialize};
use serde_poly::{DeserializePoly, Poly, SerializePoly};

#[derive(Debug, Serialize, Deserialize, Poly)]
struct Owned {
    value: String,
}

#[derive(Debug, Serialize, Deserialize, Poly)]
struct Borrowed<'a> {
    value: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Poly)]
#[poly(name = "BorrowedAlias")]
struct WithCustomName<'a> {
    data: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Poly)]
struct ZerocopyBytes<'a, const LEN: usize> {
    bytes: &'a str,
}

mod visibility_scope {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Poly)]
    pub struct Public<'a> {
        pub data: &'a str,
    }
}

fn assert_type_eq<A, B>()
where
    AssertEq<A, B>: True,
{
    let _ = core::marker::PhantomData::<AssertEq<A, B>>;
}

struct AssertEq<A, B>(core::marker::PhantomData<(A, B)>);

trait True {}

impl<T> True for AssertEq<T, T> {}

#[test]
fn owned_types_use_self() {
    assert_type_eq::<<Owned as DeserializePoly>::Out<'static>, Owned>();
    assert_type_eq::<<Owned as SerializePoly>::Out, Owned>();
}

#[test]
fn borrowed_types_generate_poly_struct() {
    type BorrowedSerializeOut = <Borrowed<'static> as SerializePoly>::Out;
    assert_type_eq::<BorrowedSerializeOut, BorrowedPoly>();

    type BorrowedOut<'de> = <BorrowedPoly as DeserializePoly>::Out<'de>;
    assert_type_eq::<BorrowedOut<'static>, Borrowed<'static>>();

    assert_eq!(std::mem::size_of::<BorrowedPoly>(), 0);
}

#[test]
fn custom_name_attribute_applies() {
    type CustomSerializeOut = <WithCustomName<'static> as SerializePoly>::Out;
    assert_type_eq::<CustomSerializeOut, BorrowedAlias>();

    type CustomOut<'de> = <BorrowedAlias as DeserializePoly>::Out<'de>;
    assert_type_eq::<CustomOut<'static>, WithCustomName<'static>>();
}

#[test]
fn public_poly_struct_is_public() {
    // TODO: this isn't a fair test, but can confirm it works from project usage
    let _ = visibility_scope::PublicPoly;
}

#[test]
fn const_generics_are_supported() {
    type SerializeOut = <ZerocopyBytes<'static, 8> as SerializePoly>::Out;
    assert_type_eq::<SerializeOut, ZerocopyBytesPoly<8>>();

    type DeserializeOut<'de> = <ZerocopyBytesPoly<8> as DeserializePoly>::Out<'de>;
    assert_type_eq::<DeserializeOut<'static>, ZerocopyBytes<'static, 8>>();

    let _ = ZerocopyBytesPoly::<8>(::core::marker::PhantomData);
}
