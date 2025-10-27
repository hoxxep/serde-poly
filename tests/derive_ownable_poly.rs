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

#[derive(OwnablePoly, Debug, PartialEq)]
enum SimpleEnum<'a> {
    Borrowed(Cow<'a, str>),
    Owned(String),
    Unit,
}

#[derive(OwnablePoly, Debug, PartialEq)]
enum ComplexEnum<'a> {
    Named {
        name: Cow<'a, str>,
        count: u32
    },
    Tuple(Cow<'a, str>, u32, String),
    Unit,
}

#[derive(OwnablePoly, Debug, PartialEq)]
enum MixedEnum<'a, T> {
    WithLifetime(Cow<'a, str>),
    WithGeneric(T),
    WithBoth {
        data: Cow<'a, str>,
        value: T
    },
}

#[derive(OwnablePoly, Debug, PartialEq)]
enum NoLifetimeEnum {
    Variant1(String),
    Variant2 { data: u32 },
    Variant3,
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

#[test]
fn test_simple_enum_borrowed() {
    let example = SimpleEnum::Borrowed(Cow::Borrowed("test"));
    let owned: SimpleEnum<'static> = example.into_owned();
    assert_eq!(owned, SimpleEnum::Borrowed(Cow::Owned("test".to_string())));
}

#[test]
fn test_simple_enum_owned() {
    let example = SimpleEnum::Owned("test".to_string());
    let owned: SimpleEnum<'static> = example.into_owned();
    assert_eq!(owned, SimpleEnum::Owned("test".to_string()));
}

#[test]
fn test_simple_enum_unit() {
    let example = SimpleEnum::Unit;
    let owned: SimpleEnum<'static> = example.into_owned();
    assert_eq!(owned, SimpleEnum::Unit);
}

#[test]
fn test_complex_enum_named() {
    let example = ComplexEnum::Named {
        name: Cow::Borrowed("test"),
        count: 42,
    };
    let owned: ComplexEnum<'static> = example.into_owned();
    assert_eq!(owned, ComplexEnum::Named {
        name: Cow::Owned("test".to_string()),
        count: 42,
    });
}

#[test]
fn test_complex_enum_tuple() {
    let example = ComplexEnum::Tuple(
        Cow::Borrowed("hello"),
        123,
        "world".to_string(),
    );
    let owned: ComplexEnum<'static> = example.into_owned();
    assert_eq!(owned, ComplexEnum::Tuple(
        Cow::Owned("hello".to_string()),
        123,
        "world".to_string(),
    ));
}

#[test]
fn test_complex_enum_unit() {
    let example = ComplexEnum::Unit;
    let owned: ComplexEnum<'static> = example.into_owned();
    assert_eq!(owned, ComplexEnum::Unit);
}

#[test]
fn test_mixed_enum_with_lifetime() {
    let example: MixedEnum<'_, String> = MixedEnum::WithLifetime(Cow::Borrowed("test"));
    let owned: MixedEnum<'static, String> = example.into_owned();
    assert_eq!(owned, MixedEnum::WithLifetime(Cow::Owned("test".to_string())));
}

#[test]
fn test_mixed_enum_with_generic() {
    let example: MixedEnum<'_, i32> = MixedEnum::WithGeneric(42);
    let owned: MixedEnum<'static, i32> = example.into_owned();
    assert_eq!(owned, MixedEnum::WithGeneric(42));
}

#[test]
fn test_mixed_enum_with_both() {
    let example: MixedEnum<'_, String> = MixedEnum::WithBoth {
        data: Cow::Borrowed("hello"),
        value: "world".to_string(),
    };
    let owned: MixedEnum<'static, String> = example.into_owned();
    assert_eq!(owned, MixedEnum::WithBoth {
        data: Cow::Owned("hello".to_string()),
        value: "world".to_string(),
    });
}

#[test]
fn test_no_lifetime_enum() {
    let example = NoLifetimeEnum::Variant1("test".to_string());
    let owned = example.into_owned();
    assert_eq!(owned, NoLifetimeEnum::Variant1("test".to_string()));

    let example = NoLifetimeEnum::Variant2 { data: 42 };
    let owned = example.into_owned();
    assert_eq!(owned, NoLifetimeEnum::Variant2 { data: 42 });

    let example = NoLifetimeEnum::Variant3;
    let owned = example.into_owned();
    assert_eq!(owned, NoLifetimeEnum::Variant3);
}