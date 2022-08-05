# prost-gen

A pure-rust, zero-dependency alternative to [prost-build](https://crates.io/crates/prost-build).

## Warning

**DO NOT USE THIS IN PRODUCTION**. This crate is just a toy!

## Why

I don't need the complex features of prost-build which is troubled by compilation issues [for a long time](https://github.com/tokio-rs/prost/pull/620). The goal of this crate is just able to compile [ricq's proto files](https://github.com/lz1998/ricq/tree/master/ricq-core/src/pb).

## Usage

Override the `prost-build` dependence in `Cargo.toml`:

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
