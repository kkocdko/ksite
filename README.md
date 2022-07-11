# ksite

All in one solution for my server.

## TODO

- `units::record`: record evidence picture, audio and video in real-time.

- `units::chat`: c2c crypto. ~~try implememt the RSA~~.

- `units::chat`: room.

- `units::status`: speed, latency, network status. ssl cert remain.

- Use `anyhow` and error handling refactor.

- HTTP2? **OMG it's much more complex than I think!**

## Build with MUSL

```
# dnf install zig
export CC="zig cc -target x86_64-linux-musl"
cargo +nightly build --release --target=x86_64-unknown-linux-musl
```
