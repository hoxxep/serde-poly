use serde_poly::{OwnablePoly};
use std::borrow::Cow;

#[derive(OwnablePoly)]
struct SimpleExample<'a> {
    data: Cow<'a, str>,
}

#[derive(OwnablePoly)]
struct MultiFieldExample<'a> {
    name: Cow<'a, str>,
    count: u32,
    values: Vec<Cow<'a, str>>,
}

#[derive(OwnablePoly)]
struct TupleExample<'a>(Cow<'a, str>, u32);

#[derive(OwnablePoly)]
struct NoLifetimeExample {
    data: String,
    count: u32,
}

#[derive(OwnablePoly)]
struct WithGenerics<'a, T> {
    data: Cow<'a, str>,
    value: T,
}

#[test]
fn test_simple_example() {
    let example = SimpleExample {
        data: Cow::Borrowed("hello"),
    };

    // Verify that we can call into_owned
    let owned: SimpleExample<'static> = example.into_owned();
    assert_eq!(owned.data, "hello");
}

#[test]
fn test_multi_field_example() {
    let example = MultiFieldExample {
        name: Cow::Borrowed("test"),
        count: 42,
        values: vec![Cow::Borrowed("a"), Cow::Borrowed("b")],
    };

    let owned: MultiFieldExample<'static> = example.into_owned();
    assert_eq!(owned.name, "test");
    assert_eq!(owned.count, 42);
    assert_eq!(owned.values.len(), 2);
}

#[test]
fn test_tuple_example() {
    let example = TupleExample(Cow::Borrowed("world"), 100);
    let owned: TupleExample<'static> = example.into_owned();
    assert_eq!(owned.0, "world");
    assert_eq!(owned.1, 100);
}

#[test]
fn test_no_lifetime_example() {
    let example = NoLifetimeExample {
        data: "test".to_string(),
        count: 5,
    };

    // For types without lifetimes, Owned = Self
    let owned = example.into_owned();
    assert_eq!(owned.data, "test");
    assert_eq!(owned.count, 5);
}

#[test]
fn test_with_generics() {
    let example = WithGenerics {
        data: Cow::Borrowed("generic"),
        value: 123i32,
    };

    let owned: WithGenerics<'static, i32> = example.into_owned();
    assert_eq!(owned.data, "generic");
    assert_eq!(owned.value, 123);
}