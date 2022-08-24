# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2022-08-24
### Changed
- Disable `serde` and `json` features by default.

## [0.1.0] - 2022-08-24 
### Added
- Create `serde` and `json` features.
- Implement `ValueExt` trait on `json::JsonValue` type.
- Implement `ValueExt` trait on `serde_json::Value` type.
- Add `ValueExt` trait providing a variety of manipulation methods for any JSON value representation. 
- Add `Pointer` type encoding [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901) representation.

[Unreleased]: https://github.com/alekece/json-toolkit-rs/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/alekece/json-toolkit-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/alekece/json-toolkit-rs/releases/tag/v0.1.0
