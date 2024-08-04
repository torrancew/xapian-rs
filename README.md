# xapian-rs

[![GitHub](https://img.shields.io/crates/l/xapian-rs)](https://github.com/torrancew/xapian-rs)
[![crates.io](https://img.shields.io/crates/d/xapian-rs)](https://crates.io/crates/xapian-rs)
[![docs.rs](https://docs.rs/xapian-rs/badge.svg)](https://docs.rs/xapian-rs)

`xapian-rs` provides a set of *low-level*, *mostly-ergonomic* Rust bindings for
the [Xapian](https://xapian.org) search library.

The bindings are provided by a mix of auto-generation (via
[`autocxx`](https://autocxx.rs)) and manual generation (via
[`cxx`](https://cxx.rs)). When necessary, small C++ shims are implemented to
work around incompatibilities between these tools and the Xapian codebase.

## Status / Stability

`xapian-rs` is currently immature, untested and incomplete. During the `0.x`
version series, no stability guarantees are provided for the API, and it may
change or break at any time. A small, limited real-world use case has been
implemented in [`pantry`](https://github.com/torrancew/pantry), which exercises
an interesting but small subset of the capabilities of Xapian:
- Indexing
- Searching
- Faceting

Some functionality is not provided at this time, including (but not limited to):
- `KeyMaker`
- Custom RangeProcessor implementations

## Design

Where possible, `xapian-rs` tries to provide simple and ergonomic interactions
with idiomatic Rust code. However, Xapian is a C++ codebase which uses C++
idioms, and this does have some consequences on the current design (as do
limitations of the `autocxx` and `cxx`):
- Xapian primarily uses exceptions for error handling. `autocxx` does not
  currently support catching exceptions (though `cxx` does). In the current
  version, **any Xapian exception will trigger a panic in Rust code**. This
  will improve as the library evolves.
- Xapian uses C++ strings very heavily. C++ strings provide no encoding
  guarantees, while Rust strings are guaranteed to be valid UTF-8. These
  bindings currently handle this in a way that is inconsistent (though at times
  convenient). This will become more well-defined as the library evolves.
- Several Xapian types are exposed in a way that allows implementation via Rust
  traits. At present, these traits are generally implemented via `&self`
  references, and therefore interior mutability is often needed to implement
  interesting functionality.
- Some of these traits intentionally leak memory when passed to FFI today. This
  will improve as the library evolves.

## Examples

Several examples are provided in the `examples` directory. The `tests`
directory's integration tests are also useful.
