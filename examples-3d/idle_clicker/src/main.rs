use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

const ORB_BASE_SCALE: f32 = 1.0;
const ORB_PULSE_AMOUNT: f32 = 0.15;
const ORB_PULSE_SPEED: f32 = 2.0;
const CLICK_BOUNCE_DURATION: f32 = 0.25;
const CLICK_BOUNCE_SCALE: f32 = 0.4;
const ORBIT_RADIUS: f32 = 3.5;
const ORBIT_SPEED: f32 = 0.5;
const SATELLITE_SCALE: f32 = 0.3;
const POPUP_LIFETIME: f32 = 1.5;
const POPUP_RISE_SPEED: f32 = 2.0;
const PASSIVE_POPUP_INTERVAL: f32 = 0.8;
const GOLD_LERP_SPEED: f64 = 12.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(IdleClickerGame::default())?;
    Ok(())
}

struct Upgrade {
    base_cost: f64,
    cost_multiplier: f64,
    level: u32,
}

impl Upgrade {
    fn cost(&self) -> f64 {
        self.base_cost * self.cost_multiplier.powi(self.level as i32)
    }
}

struct FloatingPopup {
    entity: Entity,
    lifetime: f32,
    start_position: Vec3,
    base_font_size: f32,
}

struct Satellite {
    entity: Entity,
    orbit_offset: f32,
}

struct UiHandles {
    gold_label: Entity,
    click_power_label: Entity,
    gold_per_sec_label: Entity,
    multiplier_label: Entity,
    tier_label: Entity,
    total_clicks_label: Entity,
    upgrade_buttons: Vec<Entity>,
    upgrade_cost_labels: Vec<Entity>,
}

struct IdleClickerGame {
    orb_entity: Option<Entity>,
    particle_emitter_entity: Option<Entity>,
    gold: f64,
    displayed_gold: f64,
    total_gold: f64,
    total_clicks: u64,
    click_power: f64,
    gold_per_second: f64,
    gold_multiplier: f64,
    click_bounce_timer: f32,
    passive_popup_timer: f32,
    passive_accumulated: f64,
    upgrades: Vec<Upgrade>,
    popups: Vec<FloatingPopup>,
    satellites: Vec<Satellite>,
    ui: Option<UiHandles>,
    previous_tier: u32,
    oneshot_emitters: Vec<(Entity, f32)>,
}

impl Default for IdleClickerGame {
    fn default() -> Self {
        Self {
            orb_entity: None,
            particle_emitter_entity: None,
            gold: 0.0,
            displayed_gold: 0.0,
            total_gold: 0.0,
            total_clicks: 0,
            click_power: 1.0,
            gold_per_second: 0.0,
            gold_multiplier: 1.0,
            click_bounce_timer: 0.0,
            passive_popup_timer: 0.0,
            passive_accumulated: 0.0,
            upgrades: Vec::new(),
            popups: Vec::new(),
            satellites: Vec::new(),
            ui: None,
            previous_tier: 0,
            oneshot_emitters: Vec::new(),
        }
    }
}

impl IdleClickerGame {
    fn register_material(world: &mut World, name: &str, material: Material) {
        material_registry_insert(
            &mut world.resources.material_registry,
            name.to_string(),
            material,
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
    }

    fn spawn_satellite(&mut self, world: &mut World) {
        let orbit_offset = self.satellites.len() as f32 * std::f32::consts::TAU
            / (self.satellites.len() as f32 + 1.0);

        let entity = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(ORBIT_RADIUS, 0.5, 0.0),
            Vec3::new(SATELLITE_SCALE, SATELLITE_SCALE, SATELLITE_SCALE),
        );

        let material_name = format!("satellite_{}", self.satellites.len());
        let hue = (self.satellites.len() as f32 * 0.15) % 1.0;
        let color = hsv_to_rgb(hue, 0.9, 1.0);
        Self::register_material(
            world,
            &material_name,
            Material {
                base_color: [color.x, color.y, color.z, 1.0],
                roughness: 0.2,
                metallic: 0.8,
                emissive_factor: [color.x * 2.0, color.y * 2.0, color.z * 2.0],
                ..Default::default()
            },
        );
        world
            .core
            .set_material_ref(entity, MaterialRef::new(material_name));

        let count = self.satellites.len() as f32;
        for existing in &mut self.satellites {
            existing.orbit_offset = existing.orbit_offset * count / (count + 1.0);
        }

        self.satellites.push(Satellite {
            entity,
            orbit_offset,
        });
    }

    fn spawn_popup(&mut self, world: &mut World, amount: f64, position: Vec3, is_passive: bool) {
        let offset = Vec3::new(
            rand::rng().random_range(-0.5..0.5),
            0.0,
            rand::rng().random_range(-0.5..0.5),
        );
        let spawn_pos = position + offset;

        let amount_scale = (1.0 + amount).log10().max(1.0) as f32;
        let (font_size, color, outline_color) = if is_passive {
            (
                (24.0 + amount_scale * 4.0).min(48.0),
                Vec4::new(0.3, 1.0, 0.4, 0.9),
                Vec4::new(0.0, 0.2, 0.0, 1.0),
            )
        } else {
            (
                (36.0 + amount_scale * 6.0).min(72.0),
                Vec4::new(1.0, 0.9, 0.1, 1.0),
                Vec4::new(0.3, 0.15, 0.0, 1.0),
            )
        };

        let entity = spawn_3d_billboard_text_with_properties(
            world,
            &format!("+{}", format_gold(amount)),
            spawn_pos,
            TextProperties {
                font_size,
                color,
                alignment: TextAlignment::Center,
                outline_width: 0.1,
                outline_color,
                smoothing: 0.2,
                ..Default::default()
            },
        );

        self.popups.push(FloatingPopup {
            entity,
            lifetime: POPUP_LIFETIME,
            start_position: spawn_pos,
            base_font_size: font_size,
        });
    }

    fn spawn_click_particles(&mut self, world: &mut World, position: Vec3) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let mut emitter = ParticleEmitter::sparks(position);
        emitter.one_shot = true;
        emitter.burst_count = 20;
        emitter.spawn_rate = 0.0;
        emitter.particle_lifetime_min = 0.3;
        emitter.particle_lifetime_max = 0.8;
        emitter.initial_velocity_min = 3.0;
        emitter.initial_velocity_max = 8.0;
        emitter.size_start = 0.08;
        emitter.size_end = 0.02;
        emitter.velocity_spread = 0.9;
        emitter.emissive_strength = 8.0;

        let tier = self.orb_tier();
        let tier_color = Self::orb_color_for_tier(tier);
        emitter.color_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.2, tier_color),
                (
                    0.6,
                    Vec4::new(
                        tier_color.x * 0.8,
                        tier_color.y * 0.8,
                        tier_color.z * 0.5,
                        0.8,
                    ),
                ),
                (
                    1.0,
                    Vec4::new(tier_color.x * 0.3, tier_color.y * 0.1, 0.0, 0.0),
                ),
            ],
        };
        world.core.set_particle_emitter(entity, emitter);
        self.oneshot_emitters.push((entity, 1.5));
    }

    fn spawn_tier_up_particles(&mut self, world: &mut World, position: Vec3) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let tier_color = Self::orb_color_for_tier(self.orb_tier());

        let mut emitter = ParticleEmitter::sparks(position);
        emitter.one_shot = true;
        emitter.burst_count = 120;
        emitter.spawn_rate = 0.0;
        emitter.particle_lifetime_min = 1.0;
        emitter.particle_lifetime_max = 2.5;
        emitter.initial_velocity_min = 5.0;
        emitter.initial_velocity_max = 15.0;
        emitter.size_start = 0.15;
        emitter.size_end = 0.03;
        emitter.velocity_spread = 1.0;
        emitter.emissive_strength = 15.0;
        emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
        emitter.shape =
            nightshade::ecs::particles::components::EmitterShape::Sphere { radius: 0.8 };
        emitter.color_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.1, tier_color),
                (
                    0.5,
                    Vec4::new(tier_color.x, tier_color.y, tier_color.z, 0.9),
                ),
                (
                    1.0,
                    Vec4::new(
                        tier_color.x * 0.2,
                        tier_color.y * 0.2,
                        tier_color.z * 0.2,
                        0.0,
                    ),
                ),
            ],
        };
        world.core.set_particle_emitter(entity, emitter);
        self.oneshot_emitters.push((entity, 3.5));
    }

    fn orb_tier(&self) -> u32 {
        match self.total_gold as u64 {
            0..100 => 0,
            100..1_000 => 1,
            1_000..10_000 => 2,
            10_000..100_000 => 3,
            100_000..1_000_000 => 4,
            _ => 5,
        }
    }

    fn orb_color_for_tier(tier: u32) -> Vec4 {
        match tier {
            0 => Vec4::new(0.4, 0.6, 1.0, 1.0),
            1 => Vec4::new(0.2, 1.0, 0.4, 1.0),
            2 => Vec4::new(1.0, 0.85, 0.1, 1.0),
            3 => Vec4::new(1.0, 0.3, 0.05, 1.0),
            4 => Vec4::new(0.7, 0.15, 1.0, 1.0),
            _ => Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    fn orb_emissive_for_tier(tier: u32) -> [f32; 3] {
        let color = Self::orb_color_for_tier(tier);
        let intensity = 0.5 + tier as f32 * 0.4;
        [
            color.x * intensity,
            color.y * intensity,
            color.z * intensity,
        ]
    }

    fn tier_name(tier: u32) -> &'static str {
        match tier {
            0 => "Stone",
            1 => "Jade",
            2 => "Gold",
            3 => "Ember",
            4 => "Arcane",
            _ => "Radiant",
        }
    }

    fn update_ui_labels(&self, world: &mut World) {
        let Some(ui) = &self.ui else {
            return;
        };

        world.ui_set_text(
            ui.gold_label,
            &format!("Gold: {}", format_gold(self.displayed_gold)),
        );
        world.ui_set_text(
            ui.click_power_label,
            &format!(
                "Click: {}",
                format_gold(self.click_power * self.gold_multiplier)
            ),
        );
        world.ui_set_text(
            ui.gold_per_sec_label,
            &format!(
                "Income: {}/s",
                format_gold(self.gold_per_second * self.gold_multiplier)
            ),
        );
        world.ui_set_text(
            ui.multiplier_label,
            &format!("Multiplier: x{:.1}", self.gold_multiplier),
        );

        let tier = self.orb_tier();
        world.ui_set_text(ui.tier_label, &format!("Orb: {}", Self::tier_name(tier)));
        world.ui_set_text(
            ui.total_clicks_label,
            &format!("Clicks: {}", self.total_clicks),
        );

        for (index, upgrade) in self.upgrades.iter().enumerate() {
            let cost = upgrade.cost();
            let can_afford = self.gold >= cost;

            if index < ui.upgrade_buttons.len() {
                world.ui_set_disabled(ui.upgrade_buttons[index], !can_afford);
            }

            if index < ui.upgrade_cost_labels.len() {
                world.ui_set_text(
                    ui.upgrade_cost_labels[index],
                    &format!("Lv.{} - {}", upgrade.level, format_gold(cost)),
                );
            }
        }
    }

    fn build_ui(&mut self, world: &mut World) {
        let gold_color = Vec4::new(1.0, 0.85, 0.1, 1.0);
        let text_color = Vec4::new(0.85, 0.85, 0.9, 1.0);
        let panel_bg = Vec4::new(0.06, 0.06, 0.1, 0.88);
        let panel_border = Vec4::new(0.25, 0.2, 0.4, 0.5);
        let accent = Vec4::new(0.3, 0.5, 0.9, 1.0);
        let accent_dim = Vec4::new(0.2, 0.35, 0.65, 1.0);

        let mut tree = UiTreeBuilder::new(world);

        let placeholder = Entity {
            id: 0,
            generation: 0,
        };
        let mut gold_label = placeholder;
        let mut click_power_label = placeholder;
        let mut gold_per_sec_label = placeholder;
        let mut multiplier_label = placeholder;
        let mut tier_label = placeholder;
        let mut total_clicks_label = placeholder;
        let mut upgrade_buttons = Vec::new();
        let mut upgrade_cost_labels = Vec::new();

        tree.add_node()
            .window(
                Rl(Vec2::new(0.0, 0.0)) + Ab(Vec2::new(16.0, 16.0)),
                Ab(Vec2::new(280.0, 255.0)),
                Anchor::TopLeft,
            )
            .with_rect(10.0, 1.0, panel_border)
            .with_color::<UiBase>(panel_bg)
            .flow(FlowDirection::Vertical, 20.0, 6.0)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_spacing(4.0);

                gold_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 28.0)))
                    .with_text("Gold: 0.0", 22.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(gold_color)
                    .without_pointer_events()
                    .done();

                tree.add_spacing(2.0);

                click_power_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 20.0)))
                    .with_text("Click: 1.0", 13.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(text_color)
                    .without_pointer_events()
                    .done();

                gold_per_sec_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 20.0)))
                    .with_text("Income: 0.0/s", 13.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(text_color)
                    .without_pointer_events()
                    .done();

                multiplier_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 20.0)))
                    .with_text("Multiplier: x1.0", 13.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(text_color)
                    .without_pointer_events()
                    .done();

                tree.add_spacing(2.0);

                tier_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 22.0)))
                    .with_text("Orb: Stone", 14.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(accent)
                    .without_pointer_events()
                    .done();

                total_clicks_label = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 18.0)))
                    .with_text("Clicks: 0", 11.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(Vec4::new(0.5, 0.5, 0.55, 1.0))
                    .without_pointer_events()
                    .done();
            })
            .done();

        let upgrade_labels = [
            "Click Power (x2)",
            "Auto Clicker (+1/s)",
            "Gold Rush (+10/s)",
            "Multiplier (x1.5)",
            "Mega Click (x3)",
        ];

        tree.add_node()
            .window(
                Rl(Vec2::new(0.0, 0.0)) + Ab(Vec2::new(16.0, 285.0)),
                Ab(Vec2::new(280.0, 340.0)),
                Anchor::TopLeft,
            )
            .with_rect(10.0, 1.0, panel_border)
            .with_color::<UiBase>(panel_bg)
            .flow(FlowDirection::Vertical, 16.0, 3.0)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_spacing(2.0);

                tree.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 22.0)))
                    .with_text("UPGRADES", 15.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(gold_color)
                    .without_pointer_events()
                    .done();

                tree.add_spacing(2.0);

                for (index, label) in upgrade_labels.iter().enumerate() {
                    let button = tree
                        .add_button_colored(label, if index == 0 { accent } else { accent_dim });
                    upgrade_buttons.push(button);

                    let cost_label = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 14.0)))
                        .with_text("Lv.0 - 10.0", 10.0)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(Vec4::new(0.6, 0.55, 0.4, 1.0))
                        .without_pointer_events()
                        .done();
                    upgrade_cost_labels.push(cost_label);
                }
            })
            .done();

        tree.finish();

        self.ui = Some(UiHandles {
            gold_label,
            click_power_label,
            gold_per_sec_label,
            multiplier_label,
            tier_label,
            total_clicks_label,
            upgrade_buttons,
            upgrade_cost_labels,
        });
    }

    fn recalculate_stats(&mut self) {
        let click_power_level = self.upgrades[0].level;
        let auto_clicker_level = self.upgrades[1].level;
        let gold_rush_level = self.upgrades[2].level;
        let multiplier_level = self.upgrades[3].level;
        let mega_click_level = self.upgrades[4].level;

        self.click_power =
            2.0_f64.powi(click_power_level as i32) * 3.0_f64.powi(mega_click_level as i32);
        self.gold_per_second = auto_clicker_level as f64 + gold_rush_level as f64 * 10.0;
        self.gold_multiplier = 1.5_f64.powi(multiplier_level as i32);
    }

    fn process_upgrade_clicks(&mut self, world: &mut World) {
        let Some(ui) = &self.ui else {
            return;
        };

        let mut purchase_index = None;
        for (index, &button) in ui.upgrade_buttons.iter().enumerate() {
            if world.ui_clicked(button) && index < self.upgrades.len() {
                let cost = self.upgrades[index].cost();
                if self.gold >= cost {
                    purchase_index = Some(index);
                    break;
                }
            }
        }

        if let Some(index) = purchase_index {
            let cost = self.upgrades[index].cost();
            self.gold -= cost;
            self.upgrades[index].level += 1;

            self.recalculate_stats();
            self.spawn_satellite(world);
        }
    }
}

impl State for IdleClickerGame {
    fn title(&self) -> &str {
        "Idle Clicker"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        world.resources.graphics.show_grid = false;

        spawn_sun(world);

        self.upgrades = vec![
            Upgrade {
                base_cost: 10.0,
                cost_multiplier: 1.8,
                level: 0,
            },
            Upgrade {
                base_cost: 25.0,
                cost_multiplier: 1.5,
                level: 0,
            },
            Upgrade {
                base_cost: 250.0,
                cost_multiplier: 1.6,
                level: 0,
            },
            Upgrade {
                base_cost: 500.0,
                cost_multiplier: 2.5,
                level: 0,
            },
            Upgrade {
                base_cost: 1000.0,
                cost_multiplier: 3.0,
                level: 0,
            },
        ];

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.5, 0.0),
            10.0,
            0.3,
            0.4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        Self::register_material(
            world,
            "orb",
            Material {
                base_color: [0.4, 0.6, 1.0, 1.0],
                roughness: 0.05,
                metallic: 1.0,
                emissive_factor: Self::orb_emissive_for_tier(0),
                ..Default::default()
            },
        );

        let orb = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(ORB_BASE_SCALE, ORB_BASE_SCALE, ORB_BASE_SCALE),
        );
        world
            .core
            .set_material_ref(orb, MaterialRef::new("orb".to_string()));
        self.orb_entity = Some(orb);

        Self::register_material(
            world,
            "pedestal",
            Material {
                base_color: [0.15, 0.15, 0.2, 1.0],
                roughness: 0.3,
                metallic: 0.6,
                emissive_factor: [0.02, 0.02, 0.05],
                ..Default::default()
            },
        );
        spawn_mesh_with_material(
            world,
            "Cylinder",
            Vec3::new(0.0, 0.35, 0.0),
            Vec3::new(1.5, 0.7, 1.5),
            "pedestal",
        );

        Self::register_material(
            world,
            "pedestal_base",
            Material {
                base_color: [0.1, 0.1, 0.15, 1.0],
                roughness: 0.4,
                metallic: 0.5,
                ..Default::default()
            },
        );
        spawn_mesh_with_material(
            world,
            "Cylinder",
            Vec3::new(0.0, 0.05, 0.0),
            Vec3::new(2.0, 0.1, 2.0),
            "pedestal_base",
        );

        Self::register_material(
            world,
            "ground",
            Material {
                base_color: [0.08, 0.08, 0.12, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                ..Default::default()
            },
        );
        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(30.0, 0.5, 30.0),
            "ground",
        );

        Self::register_material(
            world,
            "ring",
            Material {
                base_color: [0.4, 0.35, 0.15, 0.4],
                alpha_mode: AlphaMode::Blend,
                roughness: 0.1,
                metallic: 0.9,
                emissive_factor: [0.15, 0.12, 0.03],
                ..Default::default()
            },
        );
        spawn_mesh_with_material(
            world,
            "Torus",
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(ORBIT_RADIUS, 0.08, ORBIT_RADIUS),
            "ring",
        );

        let emitter_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let mut ambient_emitter = ParticleEmitter::sparks(Vec3::new(0.0, 2.0, 0.0));
        ambient_emitter.spawn_rate = 8.0;
        ambient_emitter.burst_count = 0;
        ambient_emitter.one_shot = false;
        ambient_emitter.particle_lifetime_min = 1.0;
        ambient_emitter.particle_lifetime_max = 2.5;
        ambient_emitter.initial_velocity_min = 0.5;
        ambient_emitter.initial_velocity_max = 1.5;
        ambient_emitter.size_start = 0.04;
        ambient_emitter.size_end = 0.01;
        ambient_emitter.velocity_spread = 1.0;
        ambient_emitter.gravity = Vec3::new(0.0, 0.5, 0.0);
        ambient_emitter.emissive_strength = 4.0;
        ambient_emitter.shape =
            nightshade::ecs::particles::components::EmitterShape::Sphere { radius: 1.2 };
        ambient_emitter.color_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.4, 0.6, 1.0, 0.0)),
                (0.2, Vec4::new(0.4, 0.6, 1.0, 0.6)),
                (0.7, Vec4::new(0.3, 0.4, 0.8, 0.3)),
                (1.0, Vec4::new(0.2, 0.2, 0.5, 0.0)),
            ],
        };
        world
            .core
            .set_particle_emitter(emitter_entity, ambient_emitter);
        self.particle_emitter_entity = Some(emitter_entity);

        self.build_ui(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        let passive_income = self.gold_per_second * self.gold_multiplier;
        let passive_gold = passive_income * delta_time as f64;
        self.gold += passive_gold;
        self.total_gold += passive_gold;
        self.passive_accumulated += passive_gold;

        let diff = self.gold - self.displayed_gold;
        if diff.abs() < 0.01 {
            self.displayed_gold = self.gold;
        } else {
            self.displayed_gold += diff * (GOLD_LERP_SPEED * delta_time as f64).min(1.0);
        }

        if passive_income > 0.0 {
            self.passive_popup_timer -= delta_time;
            if self.passive_popup_timer <= 0.0 && self.passive_accumulated >= 0.1 {
                if let Some(orb) = self.orb_entity
                    && let Some(translation) =
                        world.core.get_local_transform(orb).map(|t| t.translation)
                {
                    let popup_pos = translation + Vec3::new(0.0, 1.5, 0.0);
                    self.spawn_popup(world, self.passive_accumulated, popup_pos, true);
                }
                self.passive_accumulated = 0.0;
                self.passive_popup_timer = PASSIVE_POPUP_INTERVAL;
            }
        }

        if self.click_bounce_timer > 0.0 {
            self.click_bounce_timer = (self.click_bounce_timer - delta_time).max(0.0);
        }

        if let Some(orb) = self.orb_entity {
            let pulse = (time * ORB_PULSE_SPEED).sin() * ORB_PULSE_AMOUNT;
            let bounce = if self.click_bounce_timer > 0.0 {
                let progress = self.click_bounce_timer / CLICK_BOUNCE_DURATION;
                (progress * std::f32::consts::PI).sin() * CLICK_BOUNCE_SCALE
            } else {
                0.0
            };

            let tier = self.orb_tier();
            let tier_bonus = tier as f32 * 0.12;
            let scale = ORB_BASE_SCALE + pulse + bounce + tier_bonus;

            if let Some(transform) = world.core.get_local_transform_mut(orb) {
                transform.scale = Vec3::new(scale, scale, scale);
                let spin_speed = 0.5 + tier as f32 * 0.15;
                let rotation =
                    nalgebra_glm::quat_angle_axis(time * spin_speed, &Vec3::new(0.0, 1.0, 0.0));
                transform.rotation = rotation;
            }
            world
                .core
                .set_local_transform_dirty(orb, LocalTransformDirty);

            let tier_changed = tier != self.previous_tier;

            if tier_changed {
                let color = Self::orb_color_for_tier(tier);
                let emissive = Self::orb_emissive_for_tier(tier);
                if let Some(material) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    "orb",
                ) {
                    material.base_color = [color.x, color.y, color.z, 1.0];
                    material.emissive_factor = emissive;
                }
            }

            if tier_changed
                && self.previous_tier < tier
                && let Some(transform) = world.core.get_local_transform(orb)
            {
                self.spawn_tier_up_particles(world, transform.translation);
                self.click_bounce_timer = CLICK_BOUNCE_DURATION * 2.0;
            }
            self.previous_tier = tier;

            if tier_changed
                && let Some(emitter_entity) = self.particle_emitter_entity
                && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
            {
                let color = Self::orb_color_for_tier(tier);
                emitter.color_gradient = ColorGradient {
                    colors: vec![
                        (0.0, Vec4::new(color.x, color.y, color.z, 0.0)),
                        (0.2, Vec4::new(color.x, color.y, color.z, 0.6)),
                        (
                            0.7,
                            Vec4::new(color.x * 0.6, color.y * 0.6, color.z * 0.6, 0.3),
                        ),
                        (
                            1.0,
                            Vec4::new(color.x * 0.2, color.y * 0.2, color.z * 0.2, 0.0),
                        ),
                    ],
                };
            }

            if let Some(emitter_entity) = self.particle_emitter_entity
                && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
            {
                let income_factor = (1.0 + passive_income as f32).ln();
                emitter.spawn_rate = 8.0 + income_factor * 8.0;
                emitter.emissive_strength = 4.0 + income_factor * 2.0;
            }
        }

        let satellite_speed = ORBIT_SPEED + self.satellites.len() as f32 * 0.02;
        for satellite in &self.satellites {
            let angle = time * satellite_speed + satellite.orbit_offset;
            let bob = (time * 1.5 + satellite.orbit_offset).sin() * 0.3;
            let position = Vec3::new(
                angle.cos() * ORBIT_RADIUS,
                0.8 + bob,
                angle.sin() * ORBIT_RADIUS,
            );

            if let Some(transform) = world.core.get_local_transform_mut(satellite.entity) {
                transform.translation = position;
                let spin =
                    nalgebra_glm::quat_angle_axis(time * 3.0 + satellite.orbit_offset, &Vec3::y());
                transform.rotation = spin;
            }
            world
                .core
                .set_local_transform_dirty(satellite.entity, LocalTransformDirty);
        }

        let mut expired = Vec::new();
        for (index, popup) in self.popups.iter_mut().enumerate() {
            popup.lifetime -= delta_time;
            if popup.lifetime <= 0.0 {
                expired.push(index);
                continue;
            }

            let progress = 1.0 - (popup.lifetime / POPUP_LIFETIME);
            let offset_y = progress * POPUP_RISE_SPEED;
            let alpha = (popup.lifetime / POPUP_LIFETIME).min(1.0);
            let scale_factor = 1.0 + progress * 0.3;

            if let Some(text) = world.core.get_text_mut(popup.entity) {
                text.properties.color.w = alpha;
                text.properties.font_size = popup.base_font_size * scale_factor;
                text.dirty = true;
            }

            if let Some(transform) = world.core.get_local_transform_mut(popup.entity) {
                transform.translation = popup.start_position + Vec3::new(0.0, offset_y, 0.0);
            }
            mark_local_transform_dirty(world, popup.entity);
        }

        for index in expired.into_iter().rev() {
            let popup = self.popups.remove(index);
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: popup.entity,
                });
        }

        self.oneshot_emitters.retain_mut(|(entity, remaining)| {
            *remaining -= delta_time;
            if *remaining <= 0.0 {
                world.despawn_entities(&[*entity]);
                false
            } else {
                true
            }
        });

        self.process_upgrade_clicks(world);
        self.update_ui_labels(world);

        update_particle_emitters(world, delta_time);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
    }

    fn on_mouse_input(&mut self, world: &mut World, state: ElementState, button: MouseButton) {
        if state != ElementState::Pressed || button != MouseButton::Left {
            return;
        }

        let mouse_pos = world.resources.input.mouse.position;
        let picking_results = pick_entities(world, mouse_pos, PickingOptions::default());

        let clicked_orb = picking_results
            .iter()
            .any(|result| Some(result.entity) == self.orb_entity);

        if clicked_orb {
            let amount = self.click_power * self.gold_multiplier;
            self.gold += amount;
            self.total_gold += amount;
            self.total_clicks += 1;
            self.click_bounce_timer = CLICK_BOUNCE_DURATION;

            if let Some(orb) = self.orb_entity
                && let Some(translation) =
                    world.core.get_local_transform(orb).map(|t| t.translation)
            {
                let popup_pos = translation + Vec3::new(0.0, 1.8, 0.0);
                self.spawn_popup(world, amount, popup_pos, false);
                self.spawn_click_particles(world, translation);
            }
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.001);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }
}

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name.to_string()));
}

fn format_gold(amount: f64) -> String {
    if amount >= 1_000_000_000.0 {
        format!("{:.2}B", amount / 1_000_000_000.0)
    } else if amount >= 1_000_000.0 {
        format!("{:.2}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.2}K", amount / 1_000.0)
    } else if amount >= 100.0 {
        format!("{:.0}", amount)
    } else {
        format!("{:.1}", amount)
    }
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let chroma = value * saturation;
    let hue_segment = hue * 6.0;
    let secondary = chroma * (1.0 - (hue_segment % 2.0 - 1.0).abs());
    let (red, green, blue) = match hue_segment as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let match_value = value - chroma;
    Vec3::new(red + match_value, green + match_value, blue + match_value)
}
