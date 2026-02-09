# Nightshade Examples

Example applications built with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

<img width="2124" height="1440" alt="demo" src="https://github.com/user-attachments/assets/b476d112-c10e-40c3-8dd3-2816153c2a32" />
<img width="2560" height="1440" alt="Screenshot 2025-12-28 121420" src="https://github.com/user-attachments/assets/5ac1e414-a615-4ceb-ac58-0e209cba60f1" />

## Gallery

[View the gallery here](https://matthewberger.dev/nightshade)

<img width="1238" height="1346" alt="image" src="https://github.com/user-attachments/assets/dcfab536-f067-4988-b3f5-d8839444aaea" />

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

## Examples

| Category | Examples |
|----------|----------|
| Rendering | `alpha_blending`, `bloom`, `custom_multipass`, `custom_pass`, `decals`, `depth_of_field`, `hiz`, `lights`, `psx`, `render_layers`, `shadows`, `skybox`, `spotlight_shadows`, `sprites`, `ssao`, `textures`, `water` |
| Games | `block_breaker`, `block_breaker_scripts`, `chess`, `doom`, `farming`, `hex_war`, `jigsaw`, `platformer`, `pong`, `roguelike`, `space_shooter`, `survivors`, `tower_defense` |
| Bullet Hell | `space_shooter` — Touhou-inspired wave-based bullet hell with data-driven pattern emitters, graze mechanics, bombs, power levels, boss phase transitions, and procedural wave generation |
| Terminal | `tui_breakout`, `tui_frogger`, `tui_roguelike`, `text_adventure`, `shell`, `speedreader` |
| 3D Scenes | `asteroid_belt`, `demo_scene`, `horror`, `immersive_sim`, `level`, `maps`, `prefabs`, `winter` |
| Physics | `physics`, `physics_benchmark`, `navmesh` |
| UI | `hud_text`, `menu`, `ui`, `sdf_text`, `gizmo`, `picking` |
| Procedural | `cyberdust`, `fireworks`, `sdf_sculpt`, `terrain`, `voxels`, `lattice` |
| Audio | `audio` |
| Benchmarks | `bunnymark`, `physics_benchmark` |
| Advanced | `morph`, `multi_world`, `dance` |

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
