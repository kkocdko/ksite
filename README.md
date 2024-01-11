# ksite

All in one solution for my server.

## TODO

### 0.10.1

<!-- - `units::dav`: webdav + static serve + pastebin + account + remote download + mirror. -->

- fast uint to uint map.

### 0.11.0

- `units::reverse`: reverse proxy.

- `crate::database` pre-compile the sql statements and remove sql compiler in sqlite.

- `units::meet` supports real-time cloud record.

- `crate` proactive traffic restriction.

- `crate` no out-of-service updates.

- `units::spacehuggers` [space-huggers](https://github.com/KilledByAPixel/SpaceHuggers) game with co-op mode, webrtc or server forward fallback.

- `units::paste_next` (see the comments in its code).

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).
