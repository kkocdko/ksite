# ksite

All in one solution for my server.

## TODO

### 0.8.2

- `units::meet` supports real-time cloud record.

- `crate::log` lightweight logging, use `httpdate`. provide a log viewer mode.

### 0.9.0

- `crate` proactive traffic restriction.

- `crate` no out-of-service updates.

- `units::sandwich` cut sandwich game with co-op mode, webrtc or server forward fallback.

- `units::spacehuggers` [space-huggers](https://github.com/KilledByAPixel/SpaceHuggers) game with co-op mode, webrtc or server forward fallback.

- `units::admin` acme protocol.

- `units::paste_next` (see the comments in its code).

- `crate::database` pre-compile the sql statements and remove sql compiler in sqlite?

- `units::?` convert office files to PDF by ms office rpc.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).

If you prefer musl libc, try [cargo-zigbuild](https://github.com/messense/cargo-zigbuild), it's awesome:

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```
