# Nightshade Examples

Example applications built with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Gallery

[View the live demos here](https://matthewberger.dev/nightshade)

## Quickstart

```bash
# native
just run alpha_blending

# wasm (webgpu)
just run-wasm alpha_blending

# vr (openxr)
just run-openxr prefabs
```

> Run `just` with no arguments to list all commands

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)

Install WASM tooling:

```bash
just init-wasm
```

## Platform Guides

### Native (Windows/macOS/Linux)

Build and run natively using cargo:

```bash
just run alpha_blending  # run an example
just run picking         # run another example
just build               # release build all examples
```

### WebAssembly (WASM)

Build for web browsers with WebGPU support:

```bash
just run-wasm alpha_blending    # serve in browser
just build-wasm alpha_blending  # build only
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) starting with version `141`.

### VR (OpenXR)

Run examples in VR with OpenXR support:

```bash
just run-openxr prefabs
```

## Notes

- `steam` and `multiplayer_pong` require the Steam SDK
- `plugins` is a standalone workspace demonstrating the WASI plugin system

## License

These examples are free, open source and permissively licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
