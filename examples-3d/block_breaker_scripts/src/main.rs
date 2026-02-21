use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::scene::{
    Scene, SceneCamera, SceneEntity, SceneInstancedMesh, SceneLight, SceneMaterial, SceneMesh,
    SceneMeshInstance, save_scene, spawn_scene,
};
use nightshade::ecs::script::components::{Script, ScriptSource};
use nightshade::ecs::text::commands::spawn_hud_text_with_properties;
use nightshade::ecs::text::components::{HudAnchor, TextAlignment, TextProperties};
use nightshade::ecs::transform::LocalTransform;
use nightshade::prelude::*;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scene = create_block_breaker_scene();

    if let Err(error) = save_scene(&mut scene, Path::new("block_breaker_scripts.json")) {
        tracing::error!("Failed to save scene: {}", error);
    }

    launch(BlockBreakerScripts {
        scene: Some(scene),
        score_text: None,
        lives_text: None,
        message_text: None,
        start_text: None,
        combo_text: None,
    })?;
    Ok(())
}

struct BlockBreakerScripts {
    scene: Option<Scene>,
    score_text: Option<Entity>,
    lives_text: Option<Entity>,
    message_text: Option<Entity>,
    start_text: Option<Entity>,
    combo_text: Option<Entity>,
}

fn create_block_breaker_scene() -> Scene {
    let mut scene = Scene::new("Block Breaker Scripts");
    scene.atmosphere = Atmosphere::Nebula;

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

    let mut camera_parent = SceneEntity::new()
        .with_name("Camera")
        .with_transform(LocalTransform {
            translation: Vec3::new(0.0, 0.0, 18.0),
            ..Default::default()
        })
        .with_visible(true);
    camera_parent.components.script = Some(Script {
        source: ScriptSource::Embedded {
            source: camera_script.to_string(),
        },
        enabled: true,
    });
    let camera_parent_uuid = camera_parent.uuid;
    scene.add_entity(camera_parent);

    scene.add_entity(
        SceneEntity::new()
            .with_name("Camera_Lens")
            .with_camera(SceneCamera::Perspective {
                aspect_ratio: None,
                y_fov_rad: 1.2,
                z_far: None,
                z_near: 0.01,
            })
            .with_parent(camera_parent_uuid)
            .with_visible(true),
    );

    let sun_entity = SceneEntity::new()
        .with_name("Sun")
        .with_transform(LocalTransform {
            translation: Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_6,
                &Vec3::new(1.0, 0.0, 0.0),
            ),
            ..Default::default()
        })
        .with_visible(true);
    let sun_uuid = sun_entity.uuid;
    scene.add_entity(sun_entity);

    scene.add_entity(
        SceneEntity::new()
            .with_name("SunLight")
            .with_light(SceneLight::Directional {
                color: [1.0, 0.95, 0.9],
                intensity: 3.0,
                cast_shadows: false,
                shadow_bias: 0.0,
            })
            .with_parent(sun_uuid)
            .with_visible(true),
    );

    let wall_material = SceneMaterial {
        base_color: [0.5, 0.5, 0.55, 1.0],
        roughness: 0.7,
        ..Default::default()
    };

    add_wall(
        &mut scene,
        "Wall_Top",
        [0.0, 8.0, 0.0],
        [20.0, 0.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut scene,
        "Wall_Left",
        [-10.0, 0.0, 0.0],
        [0.5, 16.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut scene,
        "Wall_Right",
        [10.0, 0.0, 0.0],
        [0.5, 16.5, 1.0],
        wall_material.clone(),
    );
    add_wall(
        &mut scene,
        "Wall_Bottom",
        [0.0, -8.0, 0.0],
        [20.0, 0.5, 1.0],
        wall_material,
    );

    add_game_controller(&mut scene);
    add_paddle(&mut scene);
    add_ball(&mut scene);

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
            add_brick(
                &mut scene,
                row,
                col,
                [x, y, 0.0],
                brick_colors[row as usize],
            );
        }
    }

    scene
}

fn add_game_controller(scene: &mut Scene) {
    let controller_script = r#"
if just_pressed_keys.contains("W") {
    let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };
    if game_state < 0.5 {
        state["game_state"] = 1.0;
    }
}
"#;

    let mut entity = SceneEntity::new()
        .with_name("GameController")
        .with_visible(true);
    entity.components.script = Some(Script {
        source: ScriptSource::Embedded {
            source: controller_script.to_string(),
        },
        enabled: true,
    });
    scene.add_entity(entity);
}

fn add_wall(
    scene: &mut Scene,
    name: &str,
    position: [f32; 3],
    scale: [f32; 3],
    material: SceneMaterial,
) {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(LocalTransform {
            translation: Vec3::new(position[0], position[1], position[2]),
            ..Default::default()
        })
        .with_visible(true);
    entity.components.instanced_mesh = Some(
        SceneInstancedMesh::from_name(
            "Cube",
            vec![SceneMeshInstance::new([0.0, 0.0, 0.0]).with_scale(scale)],
        )
        .with_material(material),
    );
    scene.add_entity(entity);
}

fn add_paddle(scene: &mut Scene) {
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

    let mut entity = SceneEntity::new()
        .with_name("Paddle")
        .with_transform(LocalTransform {
            translation: Vec3::new(0.0, -6.0, 0.0),
            ..Default::default()
        })
        .with_visible(true);
    entity.components.script = Some(Script {
        source: ScriptSource::Embedded {
            source: paddle_script.to_string(),
        },
        enabled: true,
    });
    entity.components.mesh = Some(SceneMesh {
        mesh_uuid: None,
        mesh_name: Some("Cube".to_string()),
        material: Some(SceneMaterial {
            base_color: [0.2, 0.6, 1.0, 1.0],
            roughness: 0.3,
            ..Default::default()
        }),
    });
    entity.transform.scale = Vec3::new(3.0, 0.5, 0.5);
    scene.add_entity(entity);
}

fn add_ball(scene: &mut Scene) {
    let ball_script = r#"
let game_state = if state.contains("game_state") { state["game_state"] } else { 0.0 };

if game_state > 1.5 {
    return;
}

if game_state < 0.5 {
    return;
}

let vx = if state.contains("ball_vx") { state["ball_vx"] } else { 6.0 };
let vy = if state.contains("ball_vy") { state["ball_vy"] } else { 6.0 };

pos_x = pos_x + vx * dt;
pos_y = pos_y + vy * dt;

// Wall collisions
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

// Ball lost
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

// Paddle collision
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

// Brick collisions - check all bricks using entity_names
let hit_brick = false;
let hit_axis = 0.0;

for name in entity_names {
    if name.starts_with("Brick_") {
        if entities.contains(name) {
            let brick = entities[name];
            let bx = brick.x;
            let by = brick.y;

            let dx = pos_x - bx;
            let dy = pos_y - by;
            let abs_dx = if dx < 0.0 { 0.0 - dx } else { dx };
            let abs_dy = if dy < 0.0 { 0.0 - dy } else { dy };

            if abs_dx < 0.65 && abs_dy < 0.45 {
                despawn_names.push(name);
                hit_brick = true;

                if abs_dx / 0.65 > abs_dy / 0.45 {
                    hit_axis = 0.0;
                } else {
                    hit_axis = 1.0;
                }

                let bricks = if state.contains("bricks_remaining") { state["bricks_remaining"] } else { 102.0 };
                let new_bricks = bricks - 1.0;
                state["bricks_remaining"] = new_bricks;

                if new_bricks < 0.5 {
                    state["game_state"] = 3.0;
                }
            }
        }
    }
}

// Apply brick hit bounce
if hit_brick {
    if hit_axis < 0.5 {
        state["ball_vx"] = 0.0 - vx;
    } else {
        state["ball_vy"] = 0.0 - vy;
    }

    let score = if state.contains("score") { state["score"] } else { 0.0 };
    let combo = if state.contains("combo") { state["combo"] } else { 0.0 };
    state["score"] = score + 10.0 * (combo + 1.0);
    state["combo"] = combo + 1.0;

    state["shake_time"] = 0.15;
    let shake_base = 0.2 + (combo + 1.0) * 0.05;
    if shake_base > 0.7 { state["shake_intensity"] = 0.7; } else { state["shake_intensity"] = shake_base; }
}
"#;

    let mut entity = SceneEntity::new()
        .with_name("Ball")
        .with_transform(LocalTransform {
            translation: Vec3::new(0.0, -5.0, 0.0),
            ..Default::default()
        })
        .with_visible(true);
    entity.components.script = Some(Script {
        source: ScriptSource::Embedded {
            source: ball_script.to_string(),
        },
        enabled: true,
    });
    entity.components.mesh = Some(SceneMesh {
        mesh_uuid: None,
        mesh_name: Some("Sphere".to_string()),
        material: Some(SceneMaterial {
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.2,
            ..Default::default()
        }),
    });
    entity.transform.scale = Vec3::new(0.4, 0.4, 0.4);
    scene.add_entity(entity);
}

fn add_brick(scene: &mut Scene, row: i32, col: i32, position: [f32; 3], color: [f32; 4]) {
    let mut entity = SceneEntity::new()
        .with_name(format!("Brick_{}_{}", row, col))
        .with_transform(LocalTransform {
            translation: Vec3::new(position[0], position[1], position[2]),
            ..Default::default()
        })
        .with_visible(true);
    entity.components.instanced_mesh = Some(
        SceneInstancedMesh::from_name(
            "Cube",
            vec![SceneMeshInstance::new([0.0, 0.0, 0.0]).with_scale([0.9, 0.4, 0.5])],
        )
        .with_material(SceneMaterial {
            base_color: color,
            roughness: 0.4,
            ..Default::default()
        }),
    );
    scene.add_entity(entity);
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

        if let Some(scene) = self.scene.take() {
            match spawn_scene(world, &scene, None) {
                Ok(result) => {
                    for &entity in result.uuid_to_entity.values() {
                        if let Some(name) = world.get_name(entity)
                            && name.0 == "Camera_Lens"
                        {
                            world.resources.active_camera = Some(entity);
                            break;
                        }
                    }
                    println!("Loaded scene with {} entities", result.uuid_to_entity.len());
                }
                Err(error) => {
                    eprintln!("Failed to load scene: {}", error);
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

        let runtime = std::mem::take(&mut world.resources.script_runtime);
        nightshade::ecs::script::run_scripts_system(world);

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
