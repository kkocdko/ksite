# ksite

All in one solution for my server.

## TODO

### 0.11.0

- sqlite bench.

- admin: log view. log use tracing.

- dav: fix gvfs webdav Peer sent fatal TLS alert: Decode error.

- dav: fix windows explorer built-in client support.

- dav: process filenames properly.

- dav: remote download + mirror.

### 0.12.0

- reverse proxy.

- proxy with sni stripper.

- fast uint to uint map.

- supports real-time video cloud record.

- no out-of-service updates.

- the [space-huggers](https://github.com/KilledByAPixel/SpaceHuggers) game with co-op mode, webrtc or server forward fallback.

## Build

This crate used some unstable Rust features (most in `ricq` dependency), so use nightly toolchain please (or set `RUSTC_BOOTSTRAP=1` for stable toolchain).
