//! This example demonstrates a strongly typed Json<T> class that represents strongly-typed
//! serialized JSON data. The underlying representation is a `Cow<'a, str>`, allowing for both
//! zerocopy and owned deserialization in the same type.
//!
//! We use `Json<'static, T>` to represent owned data, and `Json<'a, T>` to represent borrowed data.
//!
//! This Json type could be expanded to support encryption, signing, automatic (de)serialization
//! into DB blobs like diesel etc. For simplicity, this example focuses on the core serialization
//! and deserialization with strong blob typing.

use std::borrow::Cow;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use serde_poly::{DeserializePoly, DeserializePolyOwned, OwnablePoly, SerializePoly};
use serde_poly_macro::Poly;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // lifetimes in this example can all be elided, but are shown explicitly for clarity

    let data: MyType<'static> = MyType {
        name: "example".to_string(),
        data: Json::serialize(&vec![1, 2, 3, 4, 5])?,
    };

    let json: Json<'static, MyTypePoly> = Json::serialize(&data)?;
    let json_str = json.as_ref();
    println!("Serialized JSON: {}", json_str);

    // send the string over the wire, store in DB, etc.

    let deserialized_json: Json<'_, MyTypePoly> = Json::from(json_str);
    let deserialized_data_borrowed: MyType<'_> = deserialized_json.deserialize()?;
    let deserialized_data_owned: MyType<'static> = deserialized_json.deserialize_into_owned()?;

    assert_eq!(data, deserialized_data_borrowed);
    assert_eq!(data, deserialized_data_owned);

    let deserialized_json_owned: Json<'static, MyTypePoly> = deserialized_json.into_owned();
    assert_eq!(json.as_ref(), deserialized_json_owned.as_ref());

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Poly, OwnablePoly)]
pub struct MyType<'a> {
    pub name: String,
    /// Nest a serialized JSON blob in this type!
    pub data: Json<'a, Vec<u64>>,
}

/// A strongly typed JSON blob wrapper that can serialize/deserialize any type T to/from JSON.
///
/// The inner type could be a `Cow<'a, str>` or `Cow<'a, [u8]>` depending on the use case.
#[derive(Clone, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Poly, OwnablePoly)]
#[serde(transparent)]  // ensure we serialize/deserialize as the inner string
pub struct Json<'a, T>(Cow<'a, str>, #[serde(skip)] PhantomData<fn() -> T>);

impl<T: SerializePoly> Json<'_, T> {
    pub fn serialize(item: &T) -> Result<Json<'static, T::Out>, serde_json::Error> {
        let s = serde_json::to_string(item)?;
        Ok(Json(Cow::Owned(s), PhantomData))
    }
}

impl<'a, T: DeserializePoly> Json<'a, T> {
    pub fn deserialize(&'a self) -> Result<T::Out<'a>, serde_json::Error> {
        let item = serde_json::from_str(&self.0)?;
        Ok(item)
    }

    pub fn deserialize_into_owned<R>(&'a self) -> Result<R, serde_json::Error>
    where
            for<'b> T::Out<'b>: OwnablePoly<Owned = R>,
    {
        let item: T::Out<'a> = self.deserialize()?;
        Ok(item.into_owned())
    }
}

impl<'a, T: DeserializePolyOwned> Json<'a, T> {
    pub fn deserialize_owned(&self) -> Result<T, serde_json::Error> {
        let item = serde_json::from_str(&self.0)?;
        Ok(item)
    }
}

impl<'a, T> From<&'a str> for Json<'a, T> {
    fn from(s: &'a str) -> Self {
        Json(Cow::Borrowed(s), PhantomData)
    }
}

impl<'a, T> From<String> for Json<'a, T> {
    fn from(s: String) -> Self {
        Json(Cow::Owned(s), PhantomData)
    }
}

impl<T> AsRef<str> for Json<'_, T> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> std::fmt::Debug for Json<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Json").field(&self.0).finish()
    }
}
