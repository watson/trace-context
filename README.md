# trace-context

Extract and inject [W3C TraceContext](https://w3c.github.io/trace-context/) headers.

- [Documentation][docs]
- [Crates.io][crates]

## Installation

```
cargo add trace-context
```

## Example usage

```rust
let mut headers = http::HeaderMap::new();
headers.insert(
  "traceparent",
  "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01".parse().unwrap()
);

let context = trace_context::TraceContext::extract(&headers).unwrap();

let trace_id = u128::from_str_radix("0af7651916cd43dd8448eb211c80319c", 16);
let parent_id = u64::from_str_radix("00f067aa0ba902b7", 16);

assert_eq!(context.trace_id(), trace_id.unwrap());
assert_eq!(context.parent_id(), trace_id.ok());
assert_eq!(context.sampled(), true);
```

## Safety

This crate uses `#![deny(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

## License

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)

[crates]: https://crates.io/crates/trace-context
[docs]: https://docs.rs/trace-context
