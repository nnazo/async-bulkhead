# async-bulkhead

This crate provies utilities for using the Bulkhead resiliency pattern in your async calls.

**NOTE: This crate is still in early development and has not been published to crates.io**

## Usage

Add one of the following to your `Cargo.toml` depending on your async runtime:

If you are using Tokio, the `tokio` feature is enabled by default so you
can specify the dependency as follows:
```toml
[dependencies]
async-bulkhead = "0.1"
```

For `async-std` or `smol`, use the following:

```toml
[dependencies]
async-bulkhead = { version = "0.1", default-features = false, features = ["async-std"] }
```

```toml
[dependencies]
async-bulkhead = { version = "0.1", default-features = false, features = ["smol"] }
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
