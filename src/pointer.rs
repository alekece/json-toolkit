use std::borrow::Cow;
use std::cmp::Ordering;
use std::str::FromStr;

use derive_more::Display;

use crate::Error;

fn decode_token(s: &str) -> String {
    s.replace("~1", "/").replace("~0", "~")
}

/// `Pointer`, a JSON pointer representation based on [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901).
///
/// This type offers strong ordering over the underlying Unicode string:
/// - JSON pointers are sorted by ascending depth.
/// - JSON pointers with the same depth are alphanumerically sorted.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[display(fmt = "{}", .0)]
pub struct Pointer<'a>(Cow<'a, str>);

impl<'a> Pointer<'a> {
    /// Creates a `Pointer` from a Unicode string as describe in [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901#section-3).
    ///
    /// # Arguments
    /// * `s`: A Unicode string representing a JSON pointer.
    ///
    /// # Examples
    /// ```
    /// # use json_toolkit::Pointer;
    ///
    /// // Construct a `Pointer` from a string literal.
    /// let pointer = Pointer::new("/a/b/c").unwrap();
    ///
    /// // Construct a `Pointer` from a owned string.
    /// let pointer = Pointer::new(String::from("/a/b/c")).unwrap();
    /// ```

    pub fn new(s: impl Into<Cow<'a, str>>) -> Result<Self, Error> {
        let pointer = s.into();

        if !pointer.is_empty() && !pointer.starts_with('/') {
            Err(Error::MissingLeadingBackslash)
        } else {
            Ok(Self(pointer))
        }
    }

    /// Creates a root JSON pointer.
    pub const fn root() -> Self {
        Self(Cow::Borrowed(""))
    }

    /// Indicates if the JSON pointer points to root value.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the Unicode string representation of the JSON pointer.
    pub fn as_str(&self) -> &str {
        &*self.0
    }

    /// Returns the last reference token of the JSON pointer, also called JSON key.
    ///
    /// Note that the root pointer does not contains any reference tokens and so no JSON key.
    ///
    /// # Example
    /// ```
    /// # use json_toolkit::Pointer;
    ///
    /// let pointer = Pointer::new("/key").unwrap();
    /// assert_eq!(pointer.key(), Some("key".to_string()));
    ///
    /// let pointer = Pointer::root();
    /// assert!(pointer.key().is_none());
    /// ```
    pub fn key(&self) -> Option<String> {
        self.0.rsplit_once('/').map(|(_, token)| decode_token(token))
    }

    /// Returns the parent JSON pointer.
    ///
    /// Note that the returned JSON pointer borrows a part of the underlying Unicode string then it can be
    /// [`clone`](Clone::clone) without any extra allocation.
    ///
    /// # Example
    /// ```
    /// # use json_toolkit::Pointer;
    ///
    /// let pointer = Pointer::new("/nested/key").unwrap();
    /// let parent_pointer = Pointer::new("/nested").unwrap();
    ///
    /// assert_eq!(pointer.parent(), Some(parent_pointer));
    /// ```
    pub fn parent(&self) -> Option<Pointer<'_>> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Pointer(Cow::Borrowed(parent)))
    }

    /// Produces an iterator over `Pointer` and its parent JSON pointers.
    ///
    /// As [`Pointer::parent`] method, all the returned JSON pointers borrow parts of the underlying Unicode string
    /// then any of them can be [`clone`](Clone::clone) without any extra allocation.
    ///
    /// The iterator will yield the `Pointer` then its parents like `self`, `self.parent().unwrap()`,
    /// `self.parent().unwrap().parent().unwrap()` and so on until reaching the root JSON pointer.
    ///
    /// # Examples
    /// ```
    /// # use json_toolkit::Pointer;
    ///
    /// let pointer = Pointer::new("/foo/bar/zoo").unwrap();
    /// let ancestors = pointer.ancestors().collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     ancestors,
    ///     vec![
    ///         Pointer::new("/foo/bar/zoo").unwrap(),
    ///         Pointer::new("/foo/bar").unwrap(),
    ///         Pointer::new("/foo").unwrap(),
    ///         Pointer::root()
    ///     ]
    /// );
    ///
    /// ```
    pub fn ancestors(&self) -> impl Iterator<Item = Pointer<'_>> {
        self.0
            .match_indices('/')
            .map(|(i, _)| i)
            .chain([self.0.len()])
            .rev()
            .map(|i| Pointer(Cow::Borrowed(&self.0[0..i])))
    }

    /// Indicates if `Pointer` is an ancestor of the given JSON pointer.
    ///
    /// Note that `Pointer` is an ancestor of itself.
    pub fn is_ancestor_of(&self, other: &Pointer<'_>) -> bool {
        other.ancestors().any(|pointer| pointer == *self)
    }

    /// Indicates if `Pointer` is a parent of the given JSON pointer.
    ///
    /// Note that the root JSON pointer is the only one with no parent.
    pub fn is_parent_of(&self, other: &Pointer<'_>) -> bool {
        other.parent().as_ref() == Some(self)
    }

    /// Indicates if `Pointer` is a sibling of the given JSON pointer.
    pub fn is_sibling_of(&self, other: &Pointer<'_>) -> bool {
        self != other && self.parent() == other.parent()
    }

    /// Indicates the number of reference tokens in the JSON pointer, in a zero-based indexed way.
    pub fn depth(&self) -> usize {
        self.0.split('/').skip(1).count()
    }

    /// Creates an owned instance of `Pointer`.
    ///
    /// Note that this function may call `Clone::clone` if the underlying Unicode string is borrowed.
    pub fn into_owned(self) -> Pointer<'static> {
        Pointer(Cow::Owned(self.0.into_owned()))
    }

    /// Evaluates `Pointer` into tokens as define in [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901#section-4).
    ///
    /// # Examples
    /// ```
    /// # use json_toolkit::Pointer;
    ///
    /// let pointer = Pointer::new("/~1foo/~0bar/zoo").unwrap();
    /// let tokens = pointer.tokenize().collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     tokens,
    ///     vec![
    ///         "/foo".to_string(),
    ///         "~bar".to_string(),
    ///         "zoo".to_string(),
    ///     ]
    /// );
    /// ```
    pub fn tokenize(&'a self) -> impl Iterator<Item = String> + 'a {
        self.0.split('/').skip(1).map(decode_token)
    }
}

impl FromStr for Pointer<'_> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl<'a> TryFrom<&'a str> for Pointer<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<String> for Pointer<'_> {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl AsRef<str> for Pointer<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Ord for Pointer<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.depth().cmp(&other.depth()) {
            Ordering::Equal => self.0.cmp(&other.0),
            ordering => ordering,
        }
    }
}

impl PartialOrd for Pointer<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_valid_json_pointer() -> Result<(), Error> {
        let tests = [
            // point to root JSON value
            "",
            // point to an empty key in the root JSON value
            "/",
            "/path/to/object",
            "/path/to/an/array/0/dummy",
        ];

        for s in tests {
            let result = Pointer::new(s);

            assert!(result.is_ok(), "'{}' is a valid JSON pointer", s);
        }

        Ok(())
    }

    #[test]
    fn it_rejects_json_pointer_without_leading_backslash() {
        let s = "path/without/leading/backslash";
        let e = Pointer::new(s);

        assert_eq!(e, Err(Error::MissingLeadingBackslash), "Invalid '{}' JSON pointer", s);
    }

    #[test]
    fn it_detects_root_json_pointer() -> Result<(), Error> {
        let tests = [Pointer::new("")?, Pointer::root()];

        for pointer in tests {
            assert!(pointer.is_root(), "'{}' is a root JSON pointer", pointer);
        }

        Ok(())
    }

    #[test]
    fn it_rejects_non_root_json_pointer() -> Result<(), Error> {
        let tests = [
            Pointer::new("/")?,
            Pointer::new("/dummy_path/to/something")?,
            Pointer::new("/0/1/2/3")?,
        ];

        for pointer in tests {
            assert!(!pointer.is_root(), "'{}' is not a root JSON pointer", pointer);
        }

        Ok(())
    }

    #[test]
    fn it_gets_parent_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), None),
            (Pointer::new("/")?, Some(Pointer::root())),
            (Pointer::new("/key")?, Some(Pointer::new("")?)),
            (Pointer::new("/nested/key")?, Some(Pointer::new("/nested")?)),
            (
                Pointer::new("/deeper/nested/key")?,
                Some(Pointer::new("/deeper/nested")?),
            ),
        ];

        for (pointer, expected_parent_pointer) in tests {
            assert_eq!(
                pointer.parent(),
                expected_parent_pointer,
                "Parent of '{}' JSON pointer",
                pointer,
            );
        }

        Ok(())
    }

    #[test]
    fn it_gets_key_from_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), None),
            (Pointer::new("/")?, Some("")),
            (Pointer::new("/key")?, Some("key")),
            (Pointer::new("/nested/key")?, Some("key")),
            (Pointer::new("/deeper/nested/key")?, Some("key")),
            (Pointer::new("/with_encoded_char/~1key")?, Some("/key")),
            (Pointer::new("/with_encoded_char/~0key")?, Some("~key")),
            (Pointer::new("/with_encoded_char/~10key")?, Some("/0key")),
            (Pointer::new("/with_encoded_char/~01key")?, Some("~1key")),
        ];

        for (pointer, expected_key) in tests {
            let expected_key = expected_key.map(ToString::to_string);
            assert_eq!(pointer.key(), expected_key, "Key of '{}' JSON pointer", pointer);
        }

        Ok(())
    }

    #[test]
    fn it_detects_parent_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), Pointer::new("/")?),
            (Pointer::new("/")?, Pointer::new("//a")?),
            (Pointer::new("/foo/0")?, Pointer::new("/foo/0/zoo")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                pointer_a.is_parent_of(&pointer_b),
                "'{}' is the parent of '{}' JSON pointer",
                pointer_a,
                pointer_b
            );
        }

        Ok(())
    }

    #[test]
    fn it_detects_non_parent_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), Pointer::root()),
            (Pointer::new("/a/b")?, Pointer::new("/a")?),
            (Pointer::new("/a/b")?, Pointer::new("/a/b")?),
            (Pointer::new("/a/b")?, Pointer::new("/a/b/c/d")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                !pointer_a.is_parent_of(&pointer_b),
                "'{}' is not the parent of '{}' JSON pointer",
                pointer_a,
                pointer_b,
            );
        }

        Ok(())
    }

    #[test]
    fn it_detects_ancestor_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), Pointer::root()),
            (Pointer::root(), Pointer::new("/")?),
            (Pointer::new("/")?, Pointer::new("//a")?),
            (Pointer::new("/a/b")?, Pointer::new("/a/b")?),
            (Pointer::new("/a/b/c")?, Pointer::new("/a/b/c/d/e/f/g")?),
            (Pointer::new("/foo/0")?, Pointer::new("/foo/0/bar/zoo")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                pointer_a.is_ancestor_of(&pointer_b),
                "'{}' is an ancestor of '{}' JSON pointer",
                pointer_a,
                pointer_b
            );
        }

        Ok(())
    }

    #[test]
    fn it_detects_non_ancestor_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::new("/a/b")?, Pointer::new("/a")?),
            (Pointer::new("/0/foo/bar/zoo")?, Pointer::new("/1/foo/bar/zoo")?),
            (Pointer::new("/tric")?, Pointer::new("/tricky/test")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                !pointer_a.is_ancestor_of(&pointer_b),
                "'{}' is not an ancestor of '{}' JSON pointer",
                pointer_a,
                pointer_b,
            );
        }

        Ok(())
    }

    #[test]
    fn it_detects_sibling_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::new("/")?, Pointer::new("/a")?),
            (Pointer::new("/a")?, Pointer::new("/")?),
            (Pointer::new("/a/b/c")?, Pointer::new("/a/b/d")?),
            (Pointer::new("/foo/bar/zoo/0")?, Pointer::new("/foo/bar/zoo/42")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                pointer_a.is_sibling_of(&pointer_b),
                "'{}' is a sibling of '{}' JSON pointer",
                pointer_a,
                pointer_b
            );
        }

        Ok(())
    }

    #[test]
    fn it_detects_non_sibling_json_pointer() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), Pointer::root()),
            (Pointer::new("/b/d")?, Pointer::new("/b/d")?),
            (Pointer::new("/b/d")?, Pointer::new("/a")?),
            (Pointer::new("/a")?, Pointer::new("/b/d")?),
            (Pointer::new("/a/b/c")?, Pointer::new("/d/e/f")?),
            (Pointer::new("/0/foo/bar/zoo")?, Pointer::new("/1/foo/bar/zoo")?),
        ];

        for (pointer_a, pointer_b) in tests {
            assert!(
                !pointer_a.is_sibling_of(&pointer_b),
                "'{}' is not a sibling of '{}' JSON pointer",
                pointer_a,
                pointer_b
            );
        }

        Ok(())
    }

    #[test]
    fn it_gets_ancestor_json_pointers() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), vec![Pointer::root()]),
            (Pointer::new("/")?, vec![Pointer::new("/")?, Pointer::root()]),
            (
                Pointer::new("/a/b")?,
                vec![Pointer::new("/a/b")?, Pointer::new("/a")?, Pointer::root()],
            ),
            (
                Pointer::new("/0/foo/bar/zoo")?,
                vec![
                    Pointer::new("/0/foo/bar/zoo")?,
                    Pointer::new("/0/foo/bar")?,
                    Pointer::new("/0/foo")?,
                    Pointer::new("/0")?,
                    Pointer::root(),
                ],
            ),
        ];

        for (pointer, expected_ancestor_pointers) in tests {
            let ancestor_pointers = pointer.ancestors().collect::<Vec<_>>();

            assert_eq!(
                ancestor_pointers, expected_ancestor_pointers,
                "Ancestors of '{}' JSON pointer",
                pointer
            );
        }

        Ok(())
    }

    #[test]
    fn it_gets_json_pointer_depth() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), 0),
            (Pointer::new("/")?, 1),
            (Pointer::new("/a")?, 1),
            (Pointer::new("/a/b/c")?, 3),
            (Pointer::new("/foo/0/bar/1/zoo/2")?, 6),
        ];

        for (pointer, expected_depth) in tests {
            assert_eq!(pointer.depth(), expected_depth, "Depth of '{}' JSON pointer", pointer);
        }

        Ok(())
    }

    #[test]
    fn it_evaluates_json_pointer_into_tokens() -> Result<(), Error> {
        let tests = [
            (Pointer::root(), vec![]),
            (Pointer::new("/")?, vec![""]),
            (Pointer::new("/~1a")?, vec!["/a"]),
            (Pointer::new("/~01a")?, vec!["~1a"]),
            (Pointer::new("/~10a")?, vec!["/0a"]),
            (Pointer::new("/~1a/~0b/c")?, vec!["/a", "~b", "c"]),
        ];

        for (pointer, expected_tokens) in tests {
            let tokens = pointer.tokenize().collect::<Vec<_>>();
            let tokens = tokens.iter().map(|s| s.as_str()).collect::<Vec<_>>();

            assert_eq!(tokens, expected_tokens, "Tokens of '{}' JSON pointer", pointer);
        }

        Ok(())
    }
}
