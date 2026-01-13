# Plugins Demo

Runtime plugin system using WebAssembly (WASI) for dynamic mod loading.

## Structure

```
plugins-demo/
├── example-plugin/      # Example plugin that spawns objects
├── plugins/             # Directory for compiled .wasm plugins
└── src/main.rs          # Host application with plugin runtime
```

## Usage

```bash
# Install wasm target (one-time)
just setup

# Build plugin and run
just run

# Or step by step:
just build-plugin    # Compiles plugin to wasm
just build-host      # Builds host application
```

## Controls

- Press `S` to spawn a sphere
- Press `C` to spawn a cube
- Objects spawn automatically every 2 seconds

## Creating Plugins

Plugins depend on `nightshade-plugin` with the `guest` feature:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
nightshade-plugin = { version = "0.6", features = ["guest"] }
```

Export lifecycle functions:

```rust
use nightshade_plugin::{log, spawn_cube, drain_engine_events, EngineEvent};

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    log("Plugin loaded!");
    spawn_cube(0.0, 1.0, -5.0);
}

#[unsafe(no_mangle)]
pub extern "C" fn on_frame() {
    for event in drain_engine_events() {
        // handle events
    }
}
```

Build for wasm32-wasip1 and copy to `plugins/` directory.

See [PLUGINS.md](../../PLUGINS.md) for full documentation.
