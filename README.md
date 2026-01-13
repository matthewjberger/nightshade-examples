# Nightshade Examples

Example applications built with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Gallery

[View the live demos here](https://matthewberger.dev/nightshade)

## Quickstart

```bash
# native
just run doom

# wasm (webgpu)
just run-wasm doom

# vr (openxr)
just run-openxr horror
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
just run doom       # run an example
just run chess      # run another example
just build          # release build all examples
```

### WebAssembly (WASM)

Build for web browsers with WebGPU support:

```bash
just run-wasm doom        # serve in browser
just build-wasm doom      # build only
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) starting with version `141`.

### VR (OpenXR)

Run examples in VR with OpenXR support:

```bash
just run-openxr horror
```

## Examples

| Example | Description |
|---------|-------------|
| alpha_blending | Alpha blending demonstration |
| asteroid_belt | Asteroid belt simulation |
| audio | Audio playback |
| block_breaker | Classic block breaker game |
| block_breaker_scripts | Block breaker with Rhai scripting |
| bloom | Bloom post-processing effect |
| bunnymark | Bunnymark benchmark |
| chess | 3D chess game with physics |
| custom_multipass | Custom multi-pass rendering |
| custom_pass | Custom render pass |
| dance | Animated character demo |
| decals | Decal rendering |
| depth_of_field | Depth of field effect |
| doom | DOOM WAD renderer |
| fireworks | Particle fireworks |
| gizmo | 3D gizmo tools |
| hex_war | Hexagonal strategy game |
| horror | First-person horror game |
| hud_text | HUD text rendering |
| jigsaw | Jigsaw puzzle game |
| lattice | Lattice deformation |
| level | Level loading demo |
| maps | Map rendering |
| menu | Menu system demo |
| morph | Morph target animation |
| multiplayer_pong | Multiplayer pong (Steam) |
| navmesh | Navigation mesh pathfinding |
| physics | Physics simulation |
| picking | Mouse picking |
| platformer | 2D platformer with physics |
| plugins | WASI plugin system demo |
| pong | Classic pong game |
| prefabs | Prefab system demo |
| psx | PSX-style rendering |
| render_layers | Render layer system |
| roguelike | Roguelike game |
| sdf_text | SDF text rendering |
| shadows | Shadow mapping |
| shell | Shell texturing |
| skybox | Skybox rendering |
| spotlight_shadows | Spotlight shadow mapping |
| steam | Steam integration demo |
| survivors | Survivors-style game |
| terrain | Terrain rendering |
| text_adventure | Text adventure game |
| textures | Texture demo |
| tower_defense | Tower defense game |
| ui | UI system demo |
| voxels | Voxel rendering |
| water | Water rendering |
| winter | Winter scene demo |

## Notes

- `steam` and `multiplayer_pong` require the Steam SDK
- `plugins` is a standalone workspace demonstrating the WASI plugin system

## License

These examples are free, open source and permissively licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
