use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::map::components::{
    Map, MapCamera, MapLight, MapMaterial, MapNode, MeshInstance,
};
use nightshade::ecs::script::components::{Script, ScriptSource};
use nightshade::ecs::text::commands::spawn_hud_text_with_properties;
use nightshade::ecs::text::components::{HudAnchor, TextAlignment, TextProperties};
use nightshade::ecs::transform::LocalTransform;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = create_block_breaker_map();

    launch(BlockBreakerScripts {
        map: Some(map),
        score_text: None,
        lives_text: None,
        message_text: None,
        start_text: None,
        combo_text: None,
    })?;
    Ok(())
}

struct BlockBreakerScripts {
    map: Option<Map>,
    score_text: Option<Entity>,
    lives_text: Option<Entity>,
    message_text: Option<Entity>,
    start_text: Option<Entity>,
    combo_text: Option<Entity>,
}

fn create_block_breaker_map() -> Map {
    let mut map = Map::new("Block Breaker Scripts");
    map.atmosphere = Atmosphere::Nebula;

    let camera_script = r#"
let shake_time = if state.contains("shake_time") { state["shake_time"] } else { 0.0 };
if shake_time > 0.0 {
    let intensity = if state.contains("shake_intensity") { state["shake_intensity"] } else { 0.2 };
    let decay = shake_time / 0.15;
    pos_x = (time * 20.0).sin() * intensity * decay;
    pos_y = (time * 25.0).cos() * intensity * decay;
    state["shake_time"] = shake_time - dt;
} else {
    pos_x = 0.0;
    pos_y = 0.0;
}
"#;

    let camera_entity = map.add_root_node(MapNode::entity_with_script(
        Some("Camera".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 18.0),
            ..Default::default()
        },
        Script {
            source: ScriptSource::Embedded {
                source: camera_script.to_string(),
            },
            enabled: true,
        },
    ));
    map.add_child_node(
        camera_entity,
        MapNode::camera(MapCamera::perspective(1.2, 0.01)),
    );

    let sun_entity = map.add_root_node(MapNode::entity_full(
        Some("Sun".to_string()),
        LocalTransform {
            translation: Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_6,
                &Vec3::new(1.0, 0.0, 0.0),
            ),
            ..Default::default()
        },
    ));
    map.add_child_node(
        sun_entity,
        MapNode::light(MapLight::directional([1.0, 0.95, 0.9], 3.0)),
    );

    let wall_material = MapMaterial {
        base_color: [0.5, 0.5, 0.55, 1.0],
        roughness: 0.7,
        ..Default::default()
    };

    add_wall(
        &mut map,
        "Wall_Top",
        [0.0, 8.0, 0.0],
        [20.0, 0.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut map,
        "Wall_Left",
        [-10.0, 0.0, 0.0],
        [0.5, 16.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut map,
        "Wall_Right",
        [10.0, 0.0, 0.0],
        [0.5, 16.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut map,
        "Wall_Bottom",
        [0.0, -8.0, 0.0],
        [20.0, 0.5, 1.0],
        wall_material,
    );

    add_game_controller(&mut map);
    add_paddle(&mut map);
    add_ball(&mut map);

    let brick_colors = [
        [1.0, 0.2, 0.2, 1.0],
        [1.0, 0.6, 0.2, 1.0],
        [1.0, 1.0, 0.2, 1.0],
        [0.2, 1.0, 0.2, 1.0],
        [0.2, 0.6, 1.0, 1.0],
        [0.6, 0.2, 1.0, 1.0],
    ];

    for row in 0..6 {
        for col in -8..=8 {
            let x = col as f32 * 1.05;
            let y = 6.0 - row as f32 * 1.05;
            add_brick(&mut map, row, col, [x, y, 0.0], brick_colors[row as usize]);
        }
    }

    map
}

fn add_game_controller(map: &mut Map) {
    let controller_script = r#"
if just_pressed_keys.contains("W") {
    let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };
    if game_state < 0.5 {
        state["game_state"] = 1.0;
    }
}
"#;

    map.add_root_node(MapNode::entity_with_script(
        Some("GameController".to_string()),
        LocalTransform::default(),
        Script {
            source: ScriptSource::Embedded {
                source: controller_script.to_string(),
            },
            enabled: true,
        },
    ));
}

fn add_wall(map: &mut Map, name: &str, position: [f32; 3], scale: [f32; 3], material: MapMaterial) {
    let entity = map.add_root_node(MapNode::entity_full(
        Some(name.to_string()),
        LocalTransform {
            translation: Vec3::new(position[0], position[1], position[2]),
            ..Default::default()
        },
    ));
    map.add_child_node(
        entity,
        MapNode::instanced_mesh_with_material(
            "Cube",
            vec![MeshInstance::new([0.0, 0.0, 0.0]).with_scale(scale)],
            material,
        ),
    );
}

fn add_paddle(map: &mut Map) {
    let paddle_script = r#"
let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };
if game_state > 1.5 {
    return;
}

let speed = 12.0;
if pressed_keys.contains("LEFT") { pos_x = pos_x - speed * dt; }
if pressed_keys.contains("RIGHT") { pos_x = pos_x + speed * dt; }
if pressed_keys.contains("A") { pos_x = pos_x - speed * dt; }
if pressed_keys.contains("D") { pos_x = pos_x + speed * dt; }
if pos_x < -8.0 { pos_x = -8.0; }
if pos_x > 8.0 { pos_x = 8.0; }
state["paddle_x"] = pos_x;
"#;

    let entity = map.add_root_node(MapNode::entity_with_script(
        Some("Paddle".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, -6.0, 0.0),
            ..Default::default()
        },
        Script {
            source: ScriptSource::Embedded {
                source: paddle_script.to_string(),
            },
            enabled: true,
        },
    ));
    map.add_child_node(
        entity,
        MapNode::instanced_mesh_with_material(
            "Cube",
            vec![MeshInstance::new([0.0, 0.0, 0.0]).with_scale([3.0, 0.5, 0.5])],
            MapMaterial {
                base_color: [0.2, 0.6, 1.0, 1.0],
                roughness: 0.3,
                ..Default::default()
            },
        ),
    );
}

fn add_ball(map: &mut Map) {
    let ball_script = r#"
let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };

if game_state > 1.5 {
    state["ball_x"] = pos_x;
    state["ball_y"] = pos_y;
    return;
}

if game_state < 0.5 {
    state["ball_x"] = pos_x;
    state["ball_y"] = pos_y;
    return;
}

let brick_hit = if state.contains("brick_hit") { state["brick_hit"] } else { 0.0 };
if brick_hit > 0.5 {
    let axis = if state.contains("brick_hit_axis") { state["brick_hit_axis"] } else { 0.0 };
    if axis < 0.5 {
        let old_vx = if state.contains("ball_vx") { state["ball_vx"] } else { 6.0 };
        state["ball_vx"] = 0.0 - old_vx;
    } else {
        let old_vy = if state.contains("ball_vy") { state["ball_vy"] } else { 6.0 };
        state["ball_vy"] = 0.0 - old_vy;
    }
    state["brick_hit"] = 0.0;

    let score = if state.contains("score") { state["score"] } else { 0.0 };
    let combo = if state.contains("combo") { state["combo"] } else { 0.0 };
    state["score"] = score + 10.0 * (combo + 1.0);
    state["combo"] = combo + 1.0;

    state["shake_time"] = 0.15;
    let shake_base = 0.2 + (combo + 1.0) * 0.05;
    if shake_base > 0.7 { state["shake_intensity"] = 0.7; } else { state["shake_intensity"] = shake_base; }
}

let vx = if state.contains("ball_vx") { state["ball_vx"] } else { 6.0 };
let vy = if state.contains("ball_vy") { state["ball_vy"] } else { 6.0 };

pos_x = pos_x + vx * dt;
pos_y = pos_y + vy * dt;

if pos_x < -9.0 {
    if vx < 0.0 { state["ball_vx"] = 0.0 - vx; } else { state["ball_vx"] = vx; }
    pos_x = -9.0;
}
if pos_x > 9.0 {
    if vx > 0.0 { state["ball_vx"] = 0.0 - vx; } else { state["ball_vx"] = vx; }
    pos_x = 9.0;
}
if pos_y > 7.0 {
    if vy > 0.0 { state["ball_vy"] = 0.0 - vy; } else { state["ball_vy"] = vy; }
    pos_y = 7.0;
}

if pos_y < -7.5 {
    let lives = if state.contains("lives") { state["lives"] } else { 3.0 };
    let new_lives = lives - 1.0;
    state["lives"] = new_lives;
    state["combo"] = 0.0;

    if new_lives < 0.5 {
        state["game_state"] = 2.0;
    } else {
        pos_y = -5.0;
        pos_x = 0.0;
        state["ball_vx"] = 6.0;
        state["ball_vy"] = 6.0;
        state["game_state"] = 0.0;
    }
}

let paddle_x = if state.contains("paddle_x") { state["paddle_x"] } else { 0.0 };
if pos_y < -5.3 && pos_y > -6.5 && vy < 0.0 {
    let dx = pos_x - paddle_x;
    if dx > -1.7 && dx < 1.7 {
        if vy < 0.0 { state["ball_vy"] = 0.0 - vy; } else { state["ball_vy"] = vy; }
        pos_y = -5.3;
        let new_vx = vx + dx * 2.0;
        state["ball_vx"] = new_vx;
        state["combo"] = 0.0;
    }
}

state["ball_x"] = pos_x;
state["ball_y"] = pos_y;
"#;

    let entity = map.add_root_node(MapNode::entity_with_script(
        Some("Ball".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, -5.0, 0.0),
            ..Default::default()
        },
        Script {
            source: ScriptSource::Embedded {
                source: ball_script.to_string(),
            },
            enabled: true,
        },
    ));
    map.add_child_node(
        entity,
        MapNode::instanced_mesh_with_material(
            "Sphere",
            vec![MeshInstance::new([0.0, 0.0, 0.0]).with_uniform_scale(0.4)],
            MapMaterial {
                base_color: [1.0, 1.0, 1.0, 1.0],
                roughness: 0.2,
                ..Default::default()
            },
        ),
    );
}

fn add_brick(map: &mut Map, row: i32, col: i32, position: [f32; 3], color: [f32; 4]) {
    let brick_script = r#"
let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };
if game_state < 0.5 || game_state > 1.5 {
    return;
}

let ball_x = if state.contains("ball_x") { state["ball_x"] } else { 0.0 };
let ball_y = if state.contains("ball_y") { state["ball_y"] } else { 0.0 };

let dx = ball_x - pos_x;
let dy = ball_y - pos_y;
let abs_dx = if dx < 0.0 { 0.0 - dx } else { dx };
let abs_dy = if dy < 0.0 { 0.0 - dy } else { dy };

if abs_dx < 0.65 && abs_dy < 0.45 {
    do_despawn = true;
    state["brick_hit"] = 1.0;
    if abs_dx / 0.65 > abs_dy / 0.45 {
        state["brick_hit_axis"] = 0.0;
    } else {
        state["brick_hit_axis"] = 1.0;
    }

    let bricks = if state.contains("bricks_remaining") { state["bricks_remaining"] } else { 102.0 };
    let new_bricks = bricks - 1.0;
    state["bricks_remaining"] = new_bricks;

    if new_bricks < 0.5 {
        state["game_state"] = 3.0;
    }
}
"#;

    let entity = map.add_root_node(MapNode::entity_with_script(
        Some(format!("Brick_{}_{}", row, col)),
        LocalTransform {
            translation: Vec3::new(position[0], position[1], position[2]),
            ..Default::default()
        },
        Script {
            source: ScriptSource::Embedded {
                source: brick_script.to_string(),
            },
            enabled: true,
        },
    ));
    map.add_child_node(
        entity,
        MapNode::instanced_mesh_with_material(
            "Cube",
            vec![MeshInstance::new([0.0, 0.0, 0.0]).with_scale([0.9, 0.4, 0.5])],
            MapMaterial {
                base_color: color,
                roughness: 0.4,
                ..Default::default()
            },
        ),
    );
}

impl State for BlockBreakerScripts {
    fn title(&self) -> &str {
        "Block Breaker (Scripts)"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_cursor = false;

        world
            .resources
            .script_runtime
            .game_state
            .insert("game_state".to_string(), 0.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("lives".to_string(), 3.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("score".to_string(), 0.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("combo".to_string(), 0.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("bricks_remaining".to_string(), 102.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("ball_vx".to_string(), 6.0);
        world
            .resources
            .script_runtime
            .game_state
            .insert("ball_vy".to_string(), 6.0);

        if let Some(map) = self.map.take() {
            match spawn_map(world, &map) {
                Ok(result) => {
                    for &entity in result.node_to_entity.values() {
                        if let Some(name) = world.get_name(entity)
                            && name.0 == "Camera"
                        {
                            world.resources.active_camera = Some(entity);
                            break;
                        }
                    }
                    println!("Loaded map with {} entities", result.node_to_entity.len());
                }
                Err(error) => {
                    eprintln!("Failed to load map: {}", error);
                    let camera =
                        spawn_camera(world, Vec3::new(0.0, 0.0, 18.0), "Camera".to_string());
                    world.resources.active_camera = Some(camera);
                }
            }
        } else {
            let camera = spawn_camera(world, Vec3::new(0.0, 0.0, 18.0), "Camera".to_string());
            world.resources.active_camera = Some(camera);
        }

        let hud_props = TextProperties {
            font_size: 24.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
            outline_width: 0.08,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.score_text = Some(spawn_hud_text_with_properties(
            world,
            "Score: 0",
            HudAnchor::TopLeft,
            nalgebra_glm::vec2(20.0, 20.0),
            hud_props.clone(),
        ));

        self.lives_text = Some(spawn_hud_text_with_properties(
            world,
            "Lives: 3",
            HudAnchor::TopRight,
            nalgebra_glm::vec2(-20.0, 20.0),
            hud_props.clone(),
        ));

        let message_props = TextProperties {
            font_size: 48.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.message_text = Some(spawn_hud_text_with_properties(
            world,
            "",
            HudAnchor::Center,
            nalgebra_glm::vec2(0.0, 0.0),
            message_props,
        ));

        let start_props = TextProperties {
            font_size: 20.0,
            color: nalgebra_glm::vec4(0.8, 0.8, 0.8, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.05,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.start_text = Some(spawn_hud_text_with_properties(
            world,
            "Press W to start",
            HudAnchor::BottomCenter,
            nalgebra_glm::vec2(0.0, -40.0),
            start_props,
        ));

        let combo_props = TextProperties {
            font_size: 32.0,
            color: nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.combo_text = Some(spawn_hud_text_with_properties(
            world,
            "",
            HudAnchor::Center,
            nalgebra_glm::vec2(0.0, -100.0),
            combo_props,
        ));
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let mut runtime = std::mem::take(&mut world.resources.script_runtime);
        nightshade::ecs::script::run_scripts_system(world, &mut runtime);

        let score = runtime.game_state.get("score").copied().unwrap_or(0.0) as u32;
        let lives = runtime.game_state.get("lives").copied().unwrap_or(3.0) as i32;
        let combo = runtime.game_state.get("combo").copied().unwrap_or(0.0) as u32;
        let current_game_state = runtime.game_state.get("game_state").copied().unwrap_or(0.0);

        world.resources.script_runtime = runtime;

        if let Some(score_entity) = self.score_text
            && let Some(hud_text) = world.get_hud_text(score_entity)
        {
            let text_index = hud_text.text_index;
            world
                .resources
                .text_cache
                .set_text(text_index, format!("Score: {}", score));
            if let Some(hud_text) = world.get_hud_text_mut(score_entity) {
                hud_text.dirty = true;
            }
        }

        if let Some(lives_entity) = self.lives_text
            && let Some(hud_text) = world.get_hud_text(lives_entity)
        {
            let text_index = hud_text.text_index;
            world
                .resources
                .text_cache
                .set_text(text_index, format!("Lives: {}", lives.max(0)));
            if let Some(hud_text) = world.get_hud_text_mut(lives_entity) {
                hud_text.dirty = true;
            }
        }

        if let Some(combo_entity) = self.combo_text
            && let Some(hud_text) = world.get_hud_text(combo_entity)
        {
            let text_index = hud_text.text_index;
            if combo > 1 {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("COMBO x{}", combo));
            } else {
                world.resources.text_cache.set_text(text_index, "");
            }
            if let Some(hud_text) = world.get_hud_text_mut(combo_entity) {
                hud_text.dirty = true;
            }
        }

        if let Some(message_entity) = self.message_text
            && let Some(hud_text) = world.get_hud_text(message_entity)
        {
            let text_index = hud_text.text_index;
            let message = if current_game_state > 1.5 && current_game_state < 2.5 {
                "GAME OVER"
            } else if current_game_state > 2.5 {
                "VICTORY!"
            } else {
                ""
            };
            world.resources.text_cache.set_text(text_index, message);
            if let Some(hud_text) = world.get_hud_text_mut(message_entity) {
                hud_text.dirty = true;
            }
        }

        if let Some(start_entity) = self.start_text
            && let Some(hud_text) = world.get_hud_text(start_entity)
        {
            let text_index = hud_text.text_index;
            let start_message = if current_game_state < 0.5 {
                "Press W to start"
            } else {
                ""
            };
            world
                .resources
                .text_cache
                .set_text(text_index, start_message);
            if let Some(hud_text) = world.get_hud_text_mut(start_entity) {
                hud_text.dirty = true;
            }
        }
    }
}
