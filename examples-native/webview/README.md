# nightshade-webview example

A native desktop application that hosts a WebView with bidirectional IPC communication to a Leptos WASM frontend.

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

## Project Structure

```
examples/
├── src/                    # Native desktop app entry point
├── site/                   # Leptos frontend (WASM)
└── protocol/               # Shared IPC message types
```

## Quick Start

### Prerequisites

- Rust 1.90+
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`
- [just](https://github.com/casey/just) - `cargo install just`
- Node.js (for Tailwind CSS)

### Build and Run

```bash
just setup  # Install npm dependencies
just run    # Build and run the desktop app
just --list # See all available commands
```

### Other Commands

```bash
just format   # Format all code
just lint     # Lint all code
just check    # Check all code
```

## License

Dual-licensed under MIT or Apache-2.0.
