# ksite
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkkocdko%2Fksite.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkkocdko%2Fksite?ref=badge_shield)


All in one solution for my server.

## TODO

### 0.6.0

- `units::paste_next`: (see the comments in its code).

- `crate::auth`: auth from database.

### 0.7.0

- `crate`: proactive traffic restriction.

- `crate`: no out-of-service updates.

- `crate::database`: sqlite `WAL` mode.

- `units::admin`: database backup.

- `units::throw`: tiny 2D game, with webrtc, ai.

- `units::record`: record evidence picture, audio and video in real-time.

### 0.8.0

- `units::?`: convert office files to PDF by ms office rpc.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).

If you prefer musl libc, try [cargo-zigbuild](https://github.com/messense/cargo-zigbuild):

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkkocdko%2Fksite.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkkocdko%2Fksite?ref=badge_large)