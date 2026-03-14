# Nightshade Examples

Example applications built with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

<img width="2124" height="1440" alt="demo" src="https://github.com/user-attachments/assets/b476d112-c10e-40c3-8dd3-2816153c2a32" />
<img width="2560" height="1440" alt="Screenshot 2025-12-28 121420" src="https://github.com/user-attachments/assets/5ac1e414-a615-4ceb-ac58-0e209cba60f1" />

## Gallery

[View the gallery here](https://matthewberger.dev/nightshade)

<img width="1238" height="1346" alt="image" src="https://github.com/user-attachments/assets/dcfab536-f067-4988-b3f5-d8839444aaea" />

## Quickstart

```bash
# run a specific example
just run alpha_blending

# interactive pickers
just pick            # pick from all examples
just pick-2d         # pick from examples-2d/
just pick-3d         # pick from examples-3d/
just pick-wasm       # pick from examples-2d/ and examples-3d/, serve in browser
just pick-native     # pick from examples-native/
just pick-terminal   # pick from examples-terminal/, run in terminal
just pick-tui        # pick from examples-terminal/, run in a window
just pick-wasm-tui   # pick from examples-terminal/, serve in browser

# run by name
just run alpha_blending              # windowed example
just run bouncing_balls              # terminal game (in terminal)
just run-tui bouncing_balls          # terminal game (in a window)
just run-wasm alpha_blending         # windowed example (in browser)
just run-tui-wasm bouncing_balls     # terminal game (in browser)
just run-openxr prefabs              # VR (OpenXR)
```

> Run `just` with no arguments to list all commands

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)

Install WASM tooling:

```bash
just init-wasm
```

## Project Structure

Examples are organized into four directories:

| Directory | Description | Run with |
|-----------|-------------|----------|
| `examples-2d/` | 2D sprite-based examples (sprites, particle effects, 2D games). Run natively, in browser (WASM), or in VR (OpenXR). | `just run <name>` |
| `examples-3d/` | 3D mesh-based examples (rendering, 3D games, physics, shaders). Run natively, in browser (WASM), or in VR (OpenXR). | `just run <name>` |
| `examples-native/` | Native-only examples that cannot be compiled to WASM (Steam SDK, raw terminal, etc). | `just run <name>` |
| `examples-terminal/` | Terminal UI games built with the TUI framework. Run directly in a terminal, in a window via SDF text rendering, or in a browser. | `just run <name>`, `just run-tui <name>`, `just run-tui-wasm <name>` |

## Examples

### 2D (`examples-2d/`)

| Category | Examples |
|----------|----------|
| Sprites | `sprites`, `gfx`, `gfx_showcase` |
| Games | `bunnymark`, `chip8`, `roguelike`, `sokoban`, `space_shooter`, `topdown_shooter` |
| Effects | `fireworks` |
| UI | `ui`, `text_adventure` |

### 3D (`examples-3d/`)

| Category | Examples |
|----------|----------|
| Rendering | `alpha_blending`, `bloom`, `custom_multipass`, `custom_pass`, `decals`, `depth_of_field`, `hiz`, `lights`, `neon_lights`, `psx`, `render_layers`, `shadows`, `skybox`, `spotlight_shadows`, `ssao`, `ssr`, `textures`, `water` |
| Games | `block_breaker`, `block_breaker_scripts`, `castle_siege`, `chess`, `doom`, `farming`, `hex_war`, `jigsaw`, `platformer`, `pong`, `survivors`, `tower_defense`, `village_survival` |
| 3D Scenes | `asteroid_belt`, `horror`, `immersive_sim`, `maps`, `prefabs`, `winter` |
| Physics | `physics`, `physics_benchmark`, `navmesh` |
| UI | `menu`, `sdf_text`, `gizmo`, `picking` |
| Procedural | `sdf_sculpt`, `terrain`, `voxels`, `lattice`, `city` |
| Shaders | `interior_mapping`, `mosaic`, `shader_studio`, `shell` |
| Audio | `audio` |
| Advanced | `camera_in_camera`, `dance`, `genetic_walkers`, `morph`, `multi_world`, `speedreader` |

### Native-only (`examples-native/`)

| Example | Description |
|---------|-------------|
| `assets` | 3D model and asset viewer |
| `claude_chat` | Claude chat with egui UI |
| `claude_chat_leptos` | Claude chat with Leptos webview |
| `claude_chat_leptos_mcp` | Claude chat with Leptos + MCP tools |
| `claude_chat_mcp` | Claude chat with MCP scene tools |
| `demo_scene` | Visual effects and lighting demo |
| `mcp` | MCP model viewer |
| `multiplayer_pong` | Multiplayer pong (requires Steam SDK) |
| `plugins` | WASM plugin system demo |
| `steam` | Steam integration (requires Steam SDK) |
| `webview` | Embedded webview demo |

### Terminal (`examples-terminal/`)

These are TUI games that run in your terminal, in a window (via `just run-tui`), or in a browser (via `just run-tui-wasm`).

| Category | Examples |
|----------|----------|
| Arcade | `asteroids`, `breakout`, `flappy_bird`, `frogger`, `pacman`, `space_invaders`, `bouncing_balls` |
| Puzzle | `match3`, `minesweeper`, `puzzle_2048`, `solitaire`, `terminal_chess`, `terminal_sokoban`, `wordle` |
| Action | `bomberman`, `bullet_hell`, `terminal_platformer`, `terminal_doom` |
| Strategy | `colony`, `deck_builder`, `hex_strategy`, `terminal_tower_defense` |
| RPG | `terminal_roguelike`, `terminal_text_adventure`, `wasteland` |
| Simulation | `falling_sand`, `game_of_life`, `lemonade`, `pathfinding_demo` |
| Classic | `snake`, `tetris`, `terminal_pong`, `typing_game`, `rhythm` |

## Platform Guides

### Native (Windows/macOS/Linux)

Build and run natively:

```bash
just run alpha_blending  # run a windowed example
just run bouncing_balls  # run a terminal example
just build               # release build all examples
```

### Terminal (TUI in a window)

Run terminal games in a native window with SDF text rendering:

```bash
just run-tui bouncing_balls
just run-tui snake
```

### WebAssembly (WASM)

Build for web browsers with WebGPU support:

```bash
just run-wasm alpha_blending             # serve windowed example in browser
just run-tui-wasm bouncing_balls    # serve terminal example in browser
just build-wasm alpha_blending           # build only
just build-wasm-tui bouncing_balls  # build only
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
- `claude_chat_leptos` and `claude_chat_leptos_mcp` require Node.js for the Leptos webview frontend

## License

These examples are free, open source and permissively licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
