[![crates.io](https://img.shields.io/crates/v/json-toolkit.svg)](https://crates.io/crates/json-toolkit)
[![MIT licensed](https://img.shields.io/crates/l/json-toolkit.svg)](./LICENSE)
[![Documentation](https://docs.rs/json-toolkit/badge.svg)](https://docs.rs/json-toolkit)
[![CI](https://github.com/alekece/json-toolkit-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/alekece/json-toolkit-rs/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/alekece/json-toolkit-rs/branch/main/graph/badge.svg?token=40M1Q98JMQ)](https://codecov.io/gh/alekece/json-toolkit-rs)

# json-toolkit

The `json-toolkit` crate exposes all the common manipulation/validation operation expected from a JSON pointer and support
several JSON value representation :
- Encode [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901) representation in [`Pointer`](https://docs.rs/json-toolkit/latest/json_toolkit/pointer/struct.Pointer.html) type.
- Manipulate any JSON value by a JSON pointer.

```rust
use json_toolkit::{ValueExt, Pointer};
use serde_json::{Value, json};

let mut json = json!({ "foo": "bar", "zoo": { "id": 1 } });

json.insert_at(&Pointer::new("/zoo/new_field").unwrap(), "new_value").unwrap();
assert_eq!(json, json!({ "foo": "bar", "zoo": { "id": 1, "new_field": "new_value" } }));

let old_value = json.insert("foo".to_string(), 42).unwrap();
assert_eq!(old_value, Some("bar".into()));
assert_eq!(json, json!({ "foo": 42, "zoo": { "id": 1, "new_field": "new_value" } }));

let id = ValueExt::pointer(&json, &Pointer::new("/zoo/id").unwrap());
assert_eq!(id, Some(&1.into()));
```

## Features

`json-toolkit` supports several JSON value representation, and has features that may be enabled or disabled :
- `serde`: Enable [`serde`](https://docs.rs/serde/latest/serde/) {de}serialization on [`Pointer`](https://docs.rs/json-toolkit/latest/json_toolkit/struct.Pointer.html) type
and implement [`ValueExt`](https://docs.rs/json-toolkit/latest/json_toolkit/trait.ValueExt.html) on [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html) type.
- `json`: Implement [`ValueExt`](https://docs.rs/json-toolkit/latest/json_toolkit/trait.ValueExt.html) on [`json::JsonValue`](https://docs.rs/json/latest/json/enum.JsonValue.html) type.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
