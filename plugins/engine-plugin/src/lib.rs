#[cfg(target_arch = "wasm32")]
use nightshade::plugin::{
    despawn_entity, drain_engine_events, load_texture, log, set_entity_color, set_entity_material,
    set_entity_position, set_entity_rotation, set_entity_scale, spawn_cone, spawn_cube,
    spawn_cylinder, spawn_plane, spawn_sphere, EngineEvent,
};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
struct SpawnedEntity {
    entity_id: u64,
    base_y: f32,
    phase: f32,
}

#[cfg(target_arch = "wasm32")]
const COLORS: [[f32; 4]; 8] = [
    [1.0, 0.3, 0.3, 1.0],
    [0.3, 1.0, 0.3, 1.0],
    [0.3, 0.3, 1.0, 1.0],
    [1.0, 1.0, 0.3, 1.0],
    [1.0, 0.3, 1.0, 1.0],
    [0.3, 1.0, 1.0, 1.0],
    [1.0, 0.6, 0.2, 1.0],
    [0.6, 0.2, 1.0, 1.0],
];

#[cfg(target_arch = "wasm32")]
struct EnginePluginState {
    initialized: bool,
    entities: Vec<SpawnedEntity>,
    pending_spawns: Vec<(u64, f32, f32, [f32; 4])>,
    texture_id: Option<u64>,
    texture_request_id: Option<u64>,
    time: f32,
    spawn_cooldown: f32,
    next_spawn_x: f32,
    color_index: usize,
}

#[cfg(target_arch = "wasm32")]
impl Default for EnginePluginState {
    fn default() -> Self {
        Self {
            initialized: false,
            entities: Vec::new(),
            pending_spawns: Vec::new(),
            texture_id: None,
            texture_request_id: None,
            time: 0.0,
            spawn_cooldown: 0.0,
            next_spawn_x: -6.0,
            color_index: 0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static STATE: RefCell<EnginePluginState> = RefCell::new(EnginePluginState::default());
}

#[cfg(target_arch = "wasm32")]
fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut EnginePluginState) -> R,
{
    STATE.with_borrow_mut(f)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    with_state(|state| {
        if state.initialized {
            return;
        }
        state.initialized = true;

        log("[Engine Plugin] Initialized - demonstrates engine-level primitives");
        log("[Engine Plugin] Press: 1=Cube, 2=Sphere, 3=Cylinder, 4=Cone, 5=Plane");
        log("[Engine Plugin] Press: T=Apply texture, D=Despawn oldest");

        let req = load_texture("test_texture.png");
        state.texture_request_id = Some(req);

        let cube_req = spawn_cube(-4.0, 1.0, -5.0);
        state.pending_spawns.push((cube_req, 1.0, 0.0, COLORS[0]));

        let sphere_req = spawn_sphere(-2.0, 1.2, -5.0);
        state.pending_spawns.push((sphere_req, 1.2, 1.0, COLORS[1]));

        let cylinder_req = spawn_cylinder(0.0, 1.4, -5.0);
        state.pending_spawns.push((cylinder_req, 1.4, 2.0, COLORS[2]));

        let cone_req = spawn_cone(2.0, 1.6, -5.0);
        state.pending_spawns.push((cone_req, 1.6, 3.0, COLORS[3]));

        let plane_req = spawn_plane(4.0, 1.8, -5.0);
        state.pending_spawns.push((plane_req, 1.8, 4.0, COLORS[4]));

        state.color_index = 5;

        log("[Engine Plugin] Spawned: cube, sphere, cylinder, cone, plane with colors");
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn on_frame() {
    for event in drain_engine_events() {
        match event {
            EngineEvent::FrameStart { delta_time, .. } => {
                with_state(|state| {
                    state.time += delta_time;
                    state.spawn_cooldown = (state.spawn_cooldown - delta_time).max(0.0);
                });
            }
            EngineEvent::EntitySpawned {
                request_id,
                entity_id,
            } => {
                with_state(|state| {
                    if let Some(index) = state
                        .pending_spawns
                        .iter()
                        .position(|(req, _, _, _)| *req == request_id)
                    {
                        let (_, base_y, phase, color) = state.pending_spawns.remove(index);
                        state.entities.push(SpawnedEntity {
                            entity_id,
                            base_y,
                            phase,
                        });

                        set_entity_color(entity_id, color[0], color[1], color[2], color[3]);

                        if let Some(tex_id) = state.texture_id {
                            if state.entities.len() % 3 == 0 {
                                set_entity_material(entity_id, tex_id);
                            }
                        }
                    }
                });
            }
            EngineEvent::TextureLoaded {
                request_id,
                texture_id,
            } => {
                with_state(|state| {
                    if state.texture_request_id == Some(request_id) {
                        state.texture_id = Some(texture_id);
                        log(&format!(
                            "[Engine Plugin] Texture loaded with id {}",
                            texture_id
                        ));

                        for (index, entity) in state.entities.iter().enumerate() {
                            if index % 3 == 0 {
                                set_entity_material(entity.entity_id, texture_id);
                            }
                        }
                    }
                });
            }
            EngineEvent::AssetError { request_id, error } => {
                with_state(|state| {
                    if state.texture_request_id == Some(request_id) {
                        log(&format!("[Engine Plugin] Texture load failed: {}", error));
                    }
                });
            }
            EngineEvent::KeyPressed { key_code } => {
                with_state(|state| {
                    if state.spawn_cooldown > 0.0 {
                        return;
                    }

                    let x = state.next_spawn_x;
                    let z = -8.0;
                    let color = COLORS[state.color_index % COLORS.len()];
                    let y_offset = (state.color_index as f32) * 0.1;

                    match key_code {
                        6 => {
                            let req = spawn_cube(x, 1.0 + y_offset, z);
                            state.pending_spawns.push((req, 1.0 + y_offset, state.time, color));
                            state.next_spawn_x += 1.5;
                            state.spawn_cooldown = 0.2;
                            state.color_index += 1;
                        }
                        7 => {
                            let req = spawn_sphere(x, 1.0 + y_offset, z);
                            state.pending_spawns.push((req, 1.0 + y_offset, state.time, color));
                            state.next_spawn_x += 1.5;
                            state.spawn_cooldown = 0.2;
                            state.color_index += 1;
                        }
                        8 => {
                            let req = spawn_cylinder(x, 1.0 + y_offset, z);
                            state.pending_spawns.push((req, 1.0 + y_offset, state.time, color));
                            state.next_spawn_x += 1.5;
                            state.spawn_cooldown = 0.2;
                            state.color_index += 1;
                        }
                        9 => {
                            let req = spawn_cone(x, 1.0 + y_offset, z);
                            state.pending_spawns.push((req, 1.0 + y_offset, state.time, color));
                            state.next_spawn_x += 1.5;
                            state.spawn_cooldown = 0.2;
                            state.color_index += 1;
                        }
                        10 => {
                            let req = spawn_plane(x, 0.5 + y_offset, z);
                            state.pending_spawns.push((req, 0.5 + y_offset, state.time, color));
                            state.next_spawn_x += 2.0;
                            state.spawn_cooldown = 0.2;
                            state.color_index += 1;
                        }
                        38 => {
                            if let Some(tex_id) = state.texture_id {
                                for entity in &state.entities {
                                    set_entity_material(entity.entity_id, tex_id);
                                }
                                log("[Engine Plugin] Applied texture to all entities");
                            } else {
                                log("[Engine Plugin] No texture loaded yet");
                            }
                        }
                        22 => {
                            if !state.entities.is_empty() {
                                let entity = state.entities.remove(0);
                                despawn_entity(entity.entity_id);
                                log(&format!(
                                    "[Engine Plugin] Despawned entity {}",
                                    entity.entity_id
                                ));
                            }
                        }
                        _ => {}
                    }
                });
            }
            _ => {}
        }
    }

    with_state(|state| {
        for (index, entity) in state.entities.iter().enumerate() {
            let y = entity.base_y + ((state.time * 2.0 + entity.phase).sin() * 0.3);
            let scale = 1.0 + ((state.time * 1.5 + entity.phase * 0.5).sin() * 0.2);

            let base_x = -4.0 + (index as f32) * 2.0;
            if index < 5 {
                set_entity_position(entity.entity_id, base_x, y, -5.0);
            }

            set_entity_scale(entity.entity_id, scale, scale, scale);

            let angle = state.time * 0.5 + entity.phase;
            let half_angle = angle * 0.5;
            let (sin_half, cos_half) = (half_angle.sin(), half_angle.cos());
            set_entity_rotation(entity.entity_id, 0.0, sin_half, 0.0, cos_half);
        }
    });
}
