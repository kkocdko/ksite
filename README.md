# ksite

All in one solution for my server.

## TODO

- `units::record`: record evidence picture, audio and video in real-time.

- `units::chat`: webrtc?

- `units::paste`: crypto.

- Sqlite `WAL` mode sync config?

- Use `Mutex::new` const fn widely.

## Build with MUSL

Use [messense/cargo-zigbuild](https://github.com/messense/cargo-zigbuild) please.

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```

## License

Dual license: If `qqbot` feature is enabled, AGPL-3.0; Or it's MIT.

I'm not sure is this valid, FFmpeg uses different licenses (GPL / LGPL) for different features so...
