# prost-gen

A pure-rust, zero-dependency alternative to [prost-build](https://crates.io/crates/prost-build).

## Warning

**DO NOT USE THIS IN PRODUCTION**. This crate is just a toy!

## Why

The prost-build is troubled by compilation issues [for a long time](https://github.com/tokio-rs/prost/pull/620), and I don't need all of its features. The goal of this crate is just able to compile [ricq's proto files](https://github.com/lz1998/ricq/tree/576c1e9/ricq-core/src/pb).

Try to compile `ksite v0.5.1` with debug profile:

| using       | deps | size    |
| ----------- | ---- | ------- |
| prost-build | 240  | 1495 MB |
| prost-gen   | 214  | 1393 MB |

## Usage

Override the `prost-build` dependency in `Cargo.toml`:

```toml
[patch.crates-io]
prost-build = { path = "./src/crates/prost-gen" }
```

Then use it in `build.rs` just like the original prost-build:

```rust
fn main() {
    prost_build::compile_protos(&["src/a.proto"], &["src/"]).unwrap();
}
```
