pub use serde_json::Value;

use super::{Error, Pointer, ValueExt};

impl ValueExt for Value {
    fn pointer(&self, pointer: &Pointer<'_>) -> Option<&Self> {
        self.pointer(pointer.as_str())
    }

    fn pointer_mut(&mut self, pointer: &Pointer<'_>) -> Option<&mut Self> {
        self.pointer_mut(pointer.as_str())
    }

    fn insert(&mut self, key: String, value: impl Into<Self>) -> Result<Option<Self>, Error> {
        match self {
            Value::Object(object) => Ok(object.insert(key, value.into())),
            _ => Err(Error::UnsupportedInsertion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    #[test]
    fn it_looks_up_values_by_json_pointer() -> Result<(), Box<dyn std::error::Error>> {
        let mut value = json!({"foo": "bar", "zoo": {"id": 42}});
        let tests = [("", value.clone()), ("/foo", "bar".into()), ("/zoo/id", 42.into())];

        for (s, mut expected_value) in tests {
            let pointer = Pointer::new(s)?;

            let pointee_value = ValueExt::pointer(&value, &pointer);
            assert_eq!(pointee_value, Some(&expected_value));

            let pointee_value = ValueExt::pointer_mut(&mut value, &pointer);
            assert_eq!(pointee_value, Some(&mut expected_value));
        }

        Ok(())
    }

    #[test]
    fn it_inserts_value_at_pointee_json_value() -> Result<(), Box<dyn std::error::Error>> {
        let value = json!({"foo": {"bar": "zoo"}});

        let tests = [
            (json!({"foo": {"bar": "zoo", "test": 42}}), "/foo/test", 42),
            (json!({"foo": {"bar": "zoo"}, "test": 21}), "/test", 21),
        ];

        for (expected_value, s, new_value) in tests {
            let mut value = value.clone();
            let old_value = value.insert_at(&Pointer::new(s)?, new_value)?;

            assert_eq!(old_value, None);
            assert_eq!(value, expected_value);
        }

        Ok(())
    }

    #[test]
    fn it_inserts_value_at_root_json_value() -> Result<(), Box<dyn std::error::Error>> {
        let mut value = json!({"foo": {"bar": "zoo"}});
        let new_value = "test2";

        let expected_old_value = value.clone();
        let old_value = value.insert_at(&Pointer::root(), new_value)?;

        assert_eq!(old_value, Some(expected_old_value));
        assert_eq!(value, new_value);

        Ok(())
    }

    #[test]
    fn it_fails_to_insert_value_at_non_existing_pointee_json_value() -> Result<(), Box<dyn std::error::Error>> {
        let mut value = json!({"foo": {"bar": "zoo"}});

        let result = value.insert_at(&Pointer::new("/foo/not_existing/zoo")?, 42);

        assert_eq!(result, Err(Error::KeyNotFound));

        Ok(())
    }

    #[test]
    fn it_fails_to_insert_value_at_json_scalar_value() -> Result<(), Box<dyn std::error::Error>> {
        let mut value = json!({"foo": {"bar": "zoo", "array": [1, 2, 3]}});

        let tests = ["/foo/bar/zoo", "/foo/array/0"];

        for s in tests {
            let result = value.insert_at(&Pointer::new(s)?, 42);

            assert_eq!(result, Err(Error::UnsupportedInsertion));
        }

        Ok(())
    }
}
