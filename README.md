# ksite

All in one solution for my server.

## TODO

### 0.8.0

- `crate::tls` run server with default cert and key.

- `crate` proactive traffic restriction.

- `crate` no out-of-service updates.

- `units::throw` tiny 2D game, with webrtc, ai.

- `units::record` record evidence picture, audio and video in real-time.

### 0.9.0

- `units::paste_next` (see the comments in its code).

- `units::?` convert office files to PDF by ms office rpc.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).

If you prefer musl libc, try [cargo-zigbuild](https://github.com/messense/cargo-zigbuild):

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```
