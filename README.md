# ksite

All in one solution for my server.

## TODO

### 0.9.1

- `crate::log` lightweight logging, use `httpdate`. provide a log viewer.

### 0.10.1

- functions like frp

- `units::meet` supports real-time cloud record.

- `crate` proactive traffic restriction.

- `crate` no out-of-service updates.

- `units::spacehuggers` [space-huggers](https://github.com/KilledByAPixel/SpaceHuggers) game with co-op mode, webrtc or server forward fallback.

- `units::admin` acme protocol.

- `units::paste_next` (see the comments in its code).

- `units::?` convert office files to PDF by ms office rpc?

### 0.10.0

- `crate::database` an simple wrapper / orm, try proc-macro.

- `crate::database` pre-compile the sql statements and remove sql compiler in sqlite.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).

If you prefer musl libc, try [cargo-zigbuild](https://github.com/messense/cargo-zigbuild), it's awesome:

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```
