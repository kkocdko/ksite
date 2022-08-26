# ksite

All in one solution for my server.

## TODO

- `crate::auth`: admin auth from database.

- `units::admin`: db shrink, get token, admin login...

- `units::record`: record evidence picture, audio and video in real-time.

- `units::paste_next`: (see the comments in its code).

- No out-of-service updates.

- SQLite database smooth upgrades.

- SQLite `WAL` mode sync config?

## Build with MUSL

Use [messense/cargo-zigbuild](https://github.com/messense/cargo-zigbuild) please.

```
# dnf install zig # for fedora
cargo zigbuild --release --target=x86_64-unknown-linux-musl
```

## License

Dual license: If `qqbot` feature is enabled, AGPL-3.0; Or it's MIT.

I'm not sure is this valid, FFmpeg uses different licenses (GPL / LGPL) for different features so...
