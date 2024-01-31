# ksite

All in one solution for my server.

## TODO

### 0.11.0

- dav: process filenames properly.

- dav: remote download + mirror.

### 0.12.0

- reverse proxy.

- sqlite bench.

- proxy with sni stripper.

- fast uint to uint map.

- supports real-time video cloud record.

- no out-of-service updates.

- the [space-huggers](https://github.com/KilledByAPixel/SpaceHuggers) game with co-op mode, webrtc or server forward fallback.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).
