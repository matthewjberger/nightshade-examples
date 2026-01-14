# native-leptos

A native desktop application that hosts a WebView with bidirectional IPC communication to a Rust/WASM frontend.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Native App (Rust)                     │
│  ┌───────────────┐    ┌──────────────────────────────┐  │
│  │   Nightshade  │    │      WebView (wry)           │  │
│  │    (egui)     │    │  ┌────────────────────────┐  │  │
│  │               │    │  │   Leptos WASM App      │  │  │
│  │               │◄───┼──│                        │  │  │
│  │               │    │  │   postcard + base64    │  │  │
│  │               │────┼──►                        │  │  │
│  │               │    │  └────────────────────────┘  │  │
│  └───────────────┘    └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Structure

- `src/` - Native Rust backend
- `protocol/` - Shared message types (used by both native and WASM)
- `site/` - Leptos frontend compiled to WASM

## Communication

Bidirectional IPC using:
- **Serialization**: postcard (binary, compact)
- **Transport**: base64 over wry's native IPC (faster than WebSockets)

## Building

```bash
# Build the frontend (WASM)
just build-frontend

# Build and run the app
just run
```

## Requirements

- Rust 1.90+
- [Trunk](https://trunkrs.dev/) for building the WASM frontend

## License

This project is free, open source and permissively licensed! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
