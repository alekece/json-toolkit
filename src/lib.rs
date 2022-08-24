//! # json-toolkit
//!
//! The `json-toolkit` crate exposes all the common manipulation/validation operation expected from a JSON pointer and support
//! several JSON value representation :
//! - Encode [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901) representation in [`Pointer`] type.
//! - Manipulate any JSON value by a JSON pointer.
//!
//! ```
//! use json_toolkit::{ValueExt, Pointer};
//! use serde_json::{Value, json};
//!
//! let mut json = json!({ "foo": "bar", "zoo": { "id": 1 } });
//!
//! json.insert_at(&Pointer::new("/zoo/new_field").unwrap(), "new_value").unwrap();
//! assert_eq!(json, json!({ "foo": "bar", "zoo": { "id": 1, "new_field": "new_value" } }));
//!
//! let old_value = json.insert("foo".to_string(), 42).unwrap();
//! assert_eq!(old_value, Some("bar".into()));
//! assert_eq!(json, json!({ "foo": 42, "zoo": { "id": 1, "new_field": "new_value" } }));
//!
//! let id = ValueExt::pointer(&json, &Pointer::new("/zoo/id").unwrap());
//! assert_eq!(id, Some(&1.into()));
//! ```
//!
//! ## Features
//!
//! `json-toolkit` supports several JSON value representation, and has features that may be enabled or disabled :
//! - `serde`: Enable [`serde`](https://docs.rs/serde/latest/serde/) {de}serialization on [`Pointer`] type
//! and implement [`ValueExt`]on [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html) type.
//! - `json`: Implement [`ValueExt`] on [`json::JsonValue`](https://docs.rs/json/latest/json/enum.JsonValue.html) type.

mod error;
#[cfg(feature = "json")]
/// [`ValueExt`] implementation for [`json::Value`][::json::JsonValue] type.
pub mod json;
mod pointer;
#[cfg(feature = "serde")]
/// [`ValueExt`] implementation for [`serde_json::Value`] type.
pub mod serde;

pub use error::Error;
pub use pointer::Pointer;

/// An extension trait for any JSON value representation that provides a variety of manipulation methods.
pub trait ValueExt: Sized {
    /// Inserts any data at the given pointee JSON value.
    ///
    /// If the JSON pointer's key already exists in the JSON pointee value, it will be overrided.
    ///
    /// # Arguments
    /// * `pointer`: A JSON pointer.
    /// * `value`: A data to insert at the pointee JSON value.
    ///
    /// # Errors
    /// This method may fail if the pointee JSON value is not a JSON object or if it does not exist.
    fn insert_at(&mut self, pointer: &Pointer<'_>, value: impl Into<Self>) -> Result<Option<Self>, Error> {
        let mut value = value.into();

        if pointer.is_root() {
            std::mem::swap(self, &mut value);

            return Ok(Some(value));
        }

        // both `unwrap` calls are safe here since we checked earlier than the given pointer is not a root JSON pointer.
        let parent_pointer = pointer.parent().unwrap();
        let pointer_key = pointer.key().unwrap();

        match self.pointer_mut(&parent_pointer) {
            Some(pointee_value) => pointee_value.insert(pointer_key, value),
            None => Err(Error::KeyNotFound),
        }
    }

    /// Insert any data in the current JSON value.
    ///
    /// If the JSON value already contains the given key, it will be overrided.
    ///
    /// # Errors
    /// This method may fail if the current JSON value is not a JSON object.
    fn insert(&mut self, key: String, value: impl Into<Self>) -> Result<Option<Self>, Error>;

    /// Looks up a value by a JSON pointer.
    fn pointer(&self, pointer: &Pointer<'_>) -> Option<&Self>;

    /// Looks up a value by a JSON pointer and returns a mutable reference to that value.
    fn pointer_mut(&mut self, pointer: &Pointer<'_>) -> Option<&mut Self>;
}
