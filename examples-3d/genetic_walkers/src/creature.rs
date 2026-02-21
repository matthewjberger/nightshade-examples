use crate::genome::{
    Genome, JUMP, L_ELBOW, L_HIP, L_KNEE, L_SHOULDER, R_ELBOW, R_HIP, R_KNEE, R_SHOULDER,
};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

const UPPER_LEG_LENGTH: f32 = 0.40;
const LOWER_LEG_LENGTH: f32 = 0.42;
const UPPER_ARM_LENGTH: f32 = 0.30;
const FOREARM_LENGTH: f32 = 0.28;
const TORSO_HEIGHT: f32 = 0.55;
const HEAD_RADIUS: f32 = 0.12;
const HIP_SPREAD: f32 = 0.10;
const SHOULDER_SPREAD: f32 = 0.14;
const START_HIP_HEIGHT: f32 = 0.88;

const EYE_RADIUS: f32 = 0.035;
const PUPIL_RADIUS: f32 = 0.018;
const EYE_FORWARD: f32 = 0.09;
const EYE_UP: f32 = 0.03;
const EYE_SPREAD: f32 = 0.045;

const GRAVITY: f32 = 15.0;
const GROUND_SPRING: f32 = 800.0;
const GROUND_DAMPING: f32 = 20.0;
const GROUND_GRIP: f32 = 8.0;
const DRAG: f32 = 0.3;
const FALLEN_THRESHOLD: f32 = 0.3;
const FALLEN_TIME_LIMIT: f32 = 0.3;
const JUMP_STRENGTH: f32 = 5.0;
const JUMP_COOLDOWN_DURATION: f32 = 0.3;
const STUCK_VELOCITY_THRESHOLD: f32 = 0.5;
const STUCK_TIME_LIMIT: f32 = 3.0;
const ENERGY_COST_RATE: f32 = 0.5;
const OBSTACLE_BONUS: f32 = 2.0;
const COLLAPSE_DURATION: f32 = 2.0;
const JUMP_THRESHOLD: f32 = 0.3;
const FOOT_GROUND_TOLERANCE: f32 = 0.05;
const SIDE_WALL_Z: f32 = 6.0;

pub const FINISH_DISTANCE: f32 = 120.0;

pub struct Obstacle {
    pub x_start: f32,
    pub x_end: f32,
    pub height: f32,
}

pub fn generate_obstacles() -> Vec<Obstacle> {
    let mut obstacles = Vec::new();
    let count = 12;
    let start_x = 12.0;
    let end_x = 115.0;
    let spacing = (end_x - start_x) / count as f32;

    let heights = [0.3, 0.5, 0.6, 0.8, 0.85, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5];
    let widths = [1.5, 1.8, 2.0, 2.2, 2.0, 2.5, 2.2, 2.8, 2.5, 3.0, 2.8, 3.5];

    for index in 0..count {
        let center_x = start_x + spacing * (index as f32 + 0.5);
        let width = widths[index];
        let height = heights[index];
        obstacles.push(Obstacle {
            x_start: center_x - width * 0.5,
            x_end: center_x + width * 0.5,
            height,
        });
    }
    obstacles
}

fn effective_ground_height(x: f32, obstacles: &[Obstacle]) -> f32 {
    for obstacle in obstacles {
        if x >= obstacle.x_start && x <= obstacle.x_end {
            return obstacle.height;
        }
    }
    0.0
}

fn compute_hand_position(
    shoulder_angle: f32,
    elbow_angle: f32,
    shoulder_offset: nalgebra_glm::Vec3,
) -> nalgebra_glm::Vec3 {
    let z_axis = nalgebra_glm::vec3(0.0, 0.0, 1.0);
    let down = nalgebra_glm::vec3(0.0, -1.0, 0.0);

    let shoulder_rot = nalgebra_glm::quat_angle_axis(shoulder_angle, &z_axis);
    let upper_arm_dir = nalgebra_glm::quat_rotate_vec3(&shoulder_rot, &down);
    let elbow_pos = shoulder_offset + upper_arm_dir * UPPER_ARM_LENGTH;

    let elbow_rot = shoulder_rot * nalgebra_glm::quat_angle_axis(elbow_angle, &z_axis);
    let forearm_dir = nalgebra_glm::quat_rotate_vec3(&elbow_rot, &down);

    elbow_pos + forearm_dir * FOREARM_LENGTH
}

pub struct CreatureBody {
    pub head: Entity,
    pub torso: Entity,
    pub left_upper_arm: Entity,
    pub left_forearm: Entity,
    pub right_upper_arm: Entity,
    pub right_forearm: Entity,
    pub left_upper_leg: Entity,
    pub left_lower_leg: Entity,
    pub right_upper_leg: Entity,
    pub right_lower_leg: Entity,
    pub left_eye: Entity,
    pub right_eye: Entity,
    pub left_pupil: Entity,
    pub right_pupil: Entity,
}

impl CreatureBody {
    pub fn all_entities(&self) -> Vec<Entity> {
        vec![
            self.head,
            self.torso,
            self.left_upper_arm,
            self.left_forearm,
            self.right_upper_arm,
            self.right_forearm,
            self.left_upper_leg,
            self.left_lower_leg,
            self.right_upper_leg,
            self.right_lower_leg,
            self.left_eye,
            self.right_eye,
            self.left_pupil,
            self.right_pupil,
        ]
    }

    pub fn colored_entities(&self) -> Vec<Entity> {
        vec![
            self.head,
            self.torso,
            self.left_upper_arm,
            self.left_forearm,
            self.right_upper_arm,
            self.right_forearm,
            self.left_upper_leg,
            self.left_lower_leg,
            self.right_upper_leg,
            self.right_lower_leg,
        ]
    }
}

pub struct Creature {
    pub body: CreatureBody,
    pub genome: Genome,
    pub position: nalgebra_glm::Vec3,
    pub velocity: nalgebra_glm::Vec3,
    pub fallen: bool,
    pub finished: bool,
    pub finish_time: Option<f32>,
    pub low_timer: f32,
    pub fitness: f32,
    pub material_name: String,
    pub prev_left_foot: nalgebra_glm::Vec3,
    pub prev_right_foot: nalgebra_glm::Vec3,
    pub start_x: f32,
    pub base_color: [f32; 4],
    pub is_greyed: bool,
    pub jump_cooldown: f32,
    pub stuck_timer: f32,
    pub energy_spent: f32,
    pub obstacles_cleared: usize,
    pub collapse_timer: f32,
    pub fallen_at_time: f32,
    pub prev_left_hand: nalgebra_glm::Vec3,
    pub prev_right_hand: nalgebra_glm::Vec3,
}

pub fn creature_color_from_index(creature_index: usize) -> [f32; 4] {
    let hue = (creature_index as f32 * 137.508) % 360.0;
    let saturation = 0.7;
    let lightness = 0.6;
    let (red, green, blue) = hsl_to_rgb(hue, saturation, lightness);
    [red, green, blue, 1.0]
}

fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_section = hue / 60.0;
    let secondary = chroma * (1.0 - (hue_section % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hue_section as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let match_value = lightness - chroma / 2.0;
    (r1 + match_value, g1 + match_value, b1 + match_value)
}

impl Creature {
    pub fn spawn(world: &mut World, genome: Genome, creature_index: usize, z_offset: f32) -> Self {
        let base_color = creature_color_from_index(creature_index);

        let material_name = format!("creature_{}", creature_index);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color,
                roughness: 0.6,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        let head = spawn_body_part(world, "Sphere", &material_name);
        let torso = spawn_body_part(world, "Cube", &material_name);
        let left_upper_arm = spawn_body_part(world, "Cube", &material_name);
        let left_forearm = spawn_body_part(world, "Cube", &material_name);
        let right_upper_arm = spawn_body_part(world, "Cube", &material_name);
        let right_forearm = spawn_body_part(world, "Cube", &material_name);
        let left_upper_leg = spawn_body_part(world, "Cube", &material_name);
        let left_lower_leg = spawn_body_part(world, "Cube", &material_name);
        let right_upper_leg = spawn_body_part(world, "Cube", &material_name);
        let right_lower_leg = spawn_body_part(world, "Cube", &material_name);

        ensure_eye_materials(world);
        let left_eye = spawn_body_part(world, "Sphere", "eye_white");
        let right_eye = spawn_body_part(world, "Sphere", "eye_white");
        let left_pupil = spawn_body_part(world, "Sphere", "eye_pupil");
        let right_pupil = spawn_body_part(world, "Sphere", "eye_pupil");

        let start_x = 0.0;
        let position = nalgebra_glm::vec3(start_x, START_HIP_HEIGHT, z_offset);

        let initial_left_foot = compute_foot_position(
            genome.joints[L_HIP].evaluate(0.0),
            genome.joints[L_KNEE].evaluate(0.0),
            nalgebra_glm::vec3(0.0, 0.0, HIP_SPREAD),
        );
        let initial_right_foot = compute_foot_position(
            genome.joints[R_HIP].evaluate(0.0),
            genome.joints[R_KNEE].evaluate(0.0),
            nalgebra_glm::vec3(0.0, 0.0, -HIP_SPREAD),
        );
        let initial_left_hand = compute_hand_position(
            genome.joints[L_SHOULDER].evaluate(0.0),
            genome.joints[L_ELBOW].evaluate(0.0),
            nalgebra_glm::vec3(0.0, TORSO_HEIGHT, SHOULDER_SPREAD),
        );
        let initial_right_hand = compute_hand_position(
            genome.joints[R_SHOULDER].evaluate(0.0),
            genome.joints[R_ELBOW].evaluate(0.0),
            nalgebra_glm::vec3(0.0, TORSO_HEIGHT, -SHOULDER_SPREAD),
        );

        let creature = Creature {
            body: CreatureBody {
                head,
                torso,
                left_upper_arm,
                left_forearm,
                right_upper_arm,
                right_forearm,
                left_upper_leg,
                left_lower_leg,
                right_upper_leg,
                right_lower_leg,
                left_eye,
                right_eye,
                left_pupil,
                right_pupil,
            },
            genome,
            position,
            velocity: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            fallen: false,
            finished: false,
            finish_time: None,
            low_timer: 0.0,
            fitness: 0.0,
            material_name,
            prev_left_foot: initial_left_foot,
            prev_right_foot: initial_right_foot,
            start_x,
            base_color,
            is_greyed: false,
            jump_cooldown: 0.0,
            stuck_timer: 0.0,
            energy_spent: 0.0,
            obstacles_cleared: 0,
            collapse_timer: 0.0,
            fallen_at_time: 0.0,
            prev_left_hand: initial_left_hand,
            prev_right_hand: initial_right_hand,
        };

        creature.update_body_transforms(world, 0.0);
        creature
    }

    pub fn despawn(self, world: &mut World) {
        let entities = self.body.all_entities();
        for &entity in &entities {
            world
                .resources
                .mesh_render_state
                .mark_entity_removed(entity);
        }
        world.despawn_entities(&entities);
    }

    pub fn set_greyed(&mut self, world: &mut World, greyed: bool) {
        if self.is_greyed == greyed {
            return;
        }
        self.is_greyed = greyed;

        let color = if greyed {
            [0.18, 0.18, 0.18, 1.0]
        } else {
            self.base_color
        };

        material_registry_insert(
            &mut world.resources.material_registry,
            self.material_name.clone(),
            Material {
                base_color: color,
                roughness: 0.6,
                metallic: 0.0,
                ..Default::default()
            },
        );

        for entity in self.body.colored_entities() {
            world
                .resources
                .mesh_render_state
                .mark_material_dirty(entity);
        }
    }

    pub fn is_done(&self) -> bool {
        (self.fallen && self.collapse_timer > COLLAPSE_DURATION) || self.finished
    }

    pub fn update(
        &mut self,
        world: &mut World,
        time: f32,
        delta_time: f32,
        obstacles: &[Obstacle],
    ) {
        if self.finished {
            return;
        }

        if self.fallen {
            self.collapse_timer += delta_time;
            if self.collapse_timer > COLLAPSE_DURATION {
                return;
            }
            self.velocity.y -= GRAVITY * delta_time;
            self.velocity.x *= (1.0 - 5.0 * delta_time).max(0.0);
            self.velocity.z *= (1.0 - 5.0 * delta_time).max(0.0);
            self.position += self.velocity * delta_time;
            if self.position.y < 0.05 {
                self.position.y = 0.05;
                self.velocity.y = self.velocity.y.max(0.0);
            }
            self.update_body_transforms(world, self.fallen_at_time);
            return;
        }

        self.velocity.y -= GRAVITY * delta_time;
        self.velocity.x -= self.velocity.x * DRAG * delta_time;

        let left_hip_angle = self.genome.joints[L_HIP].evaluate(time);
        let left_knee_angle = self.genome.joints[L_KNEE].evaluate(time);
        let right_hip_angle = self.genome.joints[R_HIP].evaluate(time);
        let right_knee_angle = self.genome.joints[R_KNEE].evaluate(time);
        let left_shoulder_angle = self.genome.joints[L_SHOULDER].evaluate(time);
        let left_elbow_angle = self.genome.joints[L_ELBOW].evaluate(time);
        let right_shoulder_angle = self.genome.joints[R_SHOULDER].evaluate(time);
        let right_elbow_angle = self.genome.joints[R_ELBOW].evaluate(time);

        let left_foot = compute_foot_position(
            left_hip_angle,
            left_knee_angle,
            nalgebra_glm::vec3(0.0, 0.0, HIP_SPREAD),
        );
        let right_foot = compute_foot_position(
            right_hip_angle,
            right_knee_angle,
            nalgebra_glm::vec3(0.0, 0.0, -HIP_SPREAD),
        );
        let left_hand = compute_hand_position(
            left_shoulder_angle,
            left_elbow_angle,
            nalgebra_glm::vec3(0.0, TORSO_HEIGHT, SHOULDER_SPREAD),
        );
        let right_hand = compute_hand_position(
            right_shoulder_angle,
            right_elbow_angle,
            nalgebra_glm::vec3(0.0, TORSO_HEIGHT, -SHOULDER_SPREAD),
        );

        let left_foot_world = self.position + left_foot;
        let right_foot_world = self.position + right_foot;
        let left_hand_world = self.position + left_hand;
        let right_hand_world = self.position + right_hand;

        let left_foot_vel = (left_foot - self.prev_left_foot) / delta_time.max(0.001);
        let right_foot_vel = (right_foot - self.prev_right_foot) / delta_time.max(0.001);
        let left_hand_vel = (left_hand - self.prev_left_hand) / delta_time.max(0.001);
        let right_hand_vel = (right_hand - self.prev_right_hand) / delta_time.max(0.001);

        self.prev_left_foot = left_foot;
        self.prev_right_foot = right_foot;
        self.prev_left_hand = left_hand;
        self.prev_right_hand = right_hand;

        let left_on_ground = left_foot_world.y < FOOT_GROUND_TOLERANCE;
        let right_on_ground = right_foot_world.y < FOOT_GROUND_TOLERANCE;

        self.jump_cooldown -= delta_time;
        let jump_value = self.genome.joints[JUMP].evaluate(time);
        if jump_value > JUMP_THRESHOLD
            && (left_on_ground || right_on_ground)
            && self.jump_cooldown <= 0.0
        {
            self.velocity.y += JUMP_STRENGTH * self.genome.joints[JUMP].amplitude;
            self.jump_cooldown = JUMP_COOLDOWN_DURATION;
        }

        apply_ground_contact(
            &mut self.velocity,
            left_foot_world,
            left_foot_vel,
            delta_time,
            0.0,
        );
        apply_ground_contact(
            &mut self.velocity,
            right_foot_world,
            right_foot_vel,
            delta_time,
            0.0,
        );

        let left_hand_surface = effective_ground_height(left_hand_world.x, obstacles);
        let right_hand_surface = effective_ground_height(right_hand_world.x, obstacles);
        apply_ground_contact(
            &mut self.velocity,
            left_hand_world,
            left_hand_vel,
            delta_time,
            left_hand_surface,
        );
        apply_ground_contact(
            &mut self.velocity,
            right_hand_world,
            right_hand_vel,
            delta_time,
            right_hand_surface,
        );

        self.velocity.x = self.velocity.x.max(0.0);
        self.position += self.velocity * delta_time;

        if self.position.y < 0.05 {
            self.position.y = 0.05;
            self.velocity.y = self.velocity.y.max(0.0);
        }

        for obstacle in obstacles {
            if self.position.x >= obstacle.x_start
                && self.position.x <= obstacle.x_end
                && self.position.y < obstacle.height
            {
                self.position.x = obstacle.x_start;
                self.velocity.x = 0.0;
            }
        }

        if self.position.z > SIDE_WALL_Z {
            self.position.z = SIDE_WALL_Z;
            self.velocity.z = 0.0;
        } else if self.position.z < -SIDE_WALL_Z {
            self.position.z = -SIDE_WALL_Z;
            self.velocity.z = 0.0;
        }

        if self.position.y < FALLEN_THRESHOLD {
            self.low_timer += delta_time;
            if self.low_timer > FALLEN_TIME_LIMIT {
                self.fallen = true;
                self.fallen_at_time = time;
            }
        } else {
            self.low_timer = 0.0;
        }

        if self.velocity.x < STUCK_VELOCITY_THRESHOLD {
            self.stuck_timer += delta_time;
            if self.stuck_timer > STUCK_TIME_LIMIT {
                self.fallen = true;
                self.fallen_at_time = time;
            }
        } else {
            self.stuck_timer = 0.0;
        }

        let total_joint_effort: f32 = self
            .genome
            .joints
            .iter()
            .map(|joint| joint.amplitude * joint.frequency)
            .sum();
        self.energy_spent += total_joint_effort * delta_time * ENERGY_COST_RATE;

        let mut cleared = 0usize;
        for obstacle in obstacles {
            if self.position.x > obstacle.x_end {
                cleared += 1;
            }
        }
        self.obstacles_cleared = cleared;

        let distance = self.position.x - self.start_x;
        if distance >= FINISH_DISTANCE && !self.finished {
            self.finished = true;
            self.finish_time = Some(time);
            self.fitness = FINISH_DISTANCE
                + (crate::GENERATION_DURATION - time).max(0.0)
                + self.obstacles_cleared as f32 * OBSTACLE_BONUS
                - self.energy_spent;
        } else if !self.finished {
            self.fitness =
                distance + self.obstacles_cleared as f32 * OBSTACLE_BONUS - self.energy_spent;
        }

        self.update_body_transforms(world, time);
    }

    fn update_body_transforms(&self, world: &mut World, time: f32) {
        let hip = self.position;

        let left_shoulder_angle = self.genome.joints[L_SHOULDER].evaluate(time);
        let right_shoulder_angle = self.genome.joints[R_SHOULDER].evaluate(time);
        let left_elbow_angle = self.genome.joints[L_ELBOW].evaluate(time);
        let right_elbow_angle = self.genome.joints[R_ELBOW].evaluate(time);
        let left_hip_angle = self.genome.joints[L_HIP].evaluate(time);
        let right_hip_angle = self.genome.joints[R_HIP].evaluate(time);
        let left_knee_angle = self.genome.joints[L_KNEE].evaluate(time);
        let right_knee_angle = self.genome.joints[R_KNEE].evaluate(time);

        let z_axis = nalgebra_glm::vec3(0.0, 0.0, 1.0);
        let down = nalgebra_glm::vec3(0.0, -1.0, 0.0);

        world.assign_local_transform(
            self.body.torso,
            LocalTransform {
                translation: hip + nalgebra_glm::vec3(0.0, TORSO_HEIGHT * 0.5, 0.0),
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(0.15, TORSO_HEIGHT, 0.10),
            },
        );

        let head_center = hip + nalgebra_glm::vec3(0.0, TORSO_HEIGHT + HEAD_RADIUS * 1.2, 0.0);
        world.assign_local_transform(
            self.body.head,
            LocalTransform {
                translation: head_center,
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(HEAD_RADIUS * 2.0, HEAD_RADIUS * 2.0, HEAD_RADIUS * 2.0),
            },
        );

        let eye_diameter = EYE_RADIUS * 2.0;
        let pupil_diameter = PUPIL_RADIUS * 2.0;

        let left_eye_pos = head_center + nalgebra_glm::vec3(EYE_FORWARD, EYE_UP, EYE_SPREAD);
        let right_eye_pos = head_center + nalgebra_glm::vec3(EYE_FORWARD, EYE_UP, -EYE_SPREAD);
        world.assign_local_transform(
            self.body.left_eye,
            LocalTransform {
                translation: left_eye_pos,
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(eye_diameter, eye_diameter, eye_diameter),
            },
        );
        world.assign_local_transform(
            self.body.right_eye,
            LocalTransform {
                translation: right_eye_pos,
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(eye_diameter, eye_diameter, eye_diameter),
            },
        );

        let pupil_forward_offset = EYE_RADIUS * 0.5;
        let left_pupil_pos = left_eye_pos + nalgebra_glm::vec3(pupil_forward_offset, 0.0, 0.0);
        let right_pupil_pos = right_eye_pos + nalgebra_glm::vec3(pupil_forward_offset, 0.0, 0.0);
        world.assign_local_transform(
            self.body.left_pupil,
            LocalTransform {
                translation: left_pupil_pos,
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(pupil_diameter, pupil_diameter, pupil_diameter),
            },
        );
        world.assign_local_transform(
            self.body.right_pupil,
            LocalTransform {
                translation: right_pupil_pos,
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(pupil_diameter, pupil_diameter, pupil_diameter),
            },
        );

        let l_shoulder_pos = hip + nalgebra_glm::vec3(0.0, TORSO_HEIGHT, SHOULDER_SPREAD);
        let l_shoulder_rot = nalgebra_glm::quat_angle_axis(left_shoulder_angle, &z_axis);
        let l_upper_arm_dir = nalgebra_glm::quat_rotate_vec3(&l_shoulder_rot, &down);
        world.assign_local_transform(
            self.body.left_upper_arm,
            LocalTransform {
                translation: l_shoulder_pos + l_upper_arm_dir * (UPPER_ARM_LENGTH * 0.5),
                rotation: l_shoulder_rot,
                scale: nalgebra_glm::vec3(0.06, UPPER_ARM_LENGTH, 0.06),
            },
        );

        let l_elbow_pos = l_shoulder_pos + l_upper_arm_dir * UPPER_ARM_LENGTH;
        let l_elbow_rot = l_shoulder_rot * nalgebra_glm::quat_angle_axis(left_elbow_angle, &z_axis);
        let l_forearm_dir = nalgebra_glm::quat_rotate_vec3(&l_elbow_rot, &down);
        world.assign_local_transform(
            self.body.left_forearm,
            LocalTransform {
                translation: l_elbow_pos + l_forearm_dir * (FOREARM_LENGTH * 0.5),
                rotation: l_elbow_rot,
                scale: nalgebra_glm::vec3(0.05, FOREARM_LENGTH, 0.05),
            },
        );

        let r_shoulder_pos = hip + nalgebra_glm::vec3(0.0, TORSO_HEIGHT, -SHOULDER_SPREAD);
        let r_shoulder_rot = nalgebra_glm::quat_angle_axis(right_shoulder_angle, &z_axis);
        let r_upper_arm_dir = nalgebra_glm::quat_rotate_vec3(&r_shoulder_rot, &down);
        world.assign_local_transform(
            self.body.right_upper_arm,
            LocalTransform {
                translation: r_shoulder_pos + r_upper_arm_dir * (UPPER_ARM_LENGTH * 0.5),
                rotation: r_shoulder_rot,
                scale: nalgebra_glm::vec3(0.06, UPPER_ARM_LENGTH, 0.06),
            },
        );

        let r_elbow_pos = r_shoulder_pos + r_upper_arm_dir * UPPER_ARM_LENGTH;
        let r_elbow_rot =
            r_shoulder_rot * nalgebra_glm::quat_angle_axis(right_elbow_angle, &z_axis);
        let r_forearm_dir = nalgebra_glm::quat_rotate_vec3(&r_elbow_rot, &down);
        world.assign_local_transform(
            self.body.right_forearm,
            LocalTransform {
                translation: r_elbow_pos + r_forearm_dir * (FOREARM_LENGTH * 0.5),
                rotation: r_elbow_rot,
                scale: nalgebra_glm::vec3(0.05, FOREARM_LENGTH, 0.05),
            },
        );

        let l_hip_offset = nalgebra_glm::vec3(0.0, 0.0, HIP_SPREAD);
        let l_hip_rot = nalgebra_glm::quat_angle_axis(left_hip_angle, &z_axis);
        let l_upper_leg_dir = nalgebra_glm::quat_rotate_vec3(&l_hip_rot, &down);
        world.assign_local_transform(
            self.body.left_upper_leg,
            LocalTransform {
                translation: hip + l_hip_offset + l_upper_leg_dir * (UPPER_LEG_LENGTH * 0.5),
                rotation: l_hip_rot,
                scale: nalgebra_glm::vec3(0.08, UPPER_LEG_LENGTH, 0.08),
            },
        );

        let l_knee_pos = hip + l_hip_offset + l_upper_leg_dir * UPPER_LEG_LENGTH;
        let l_knee_rot = l_hip_rot * nalgebra_glm::quat_angle_axis(left_knee_angle, &z_axis);
        let l_lower_leg_dir = nalgebra_glm::quat_rotate_vec3(&l_knee_rot, &down);
        world.assign_local_transform(
            self.body.left_lower_leg,
            LocalTransform {
                translation: l_knee_pos + l_lower_leg_dir * (LOWER_LEG_LENGTH * 0.5),
                rotation: l_knee_rot,
                scale: nalgebra_glm::vec3(0.07, LOWER_LEG_LENGTH, 0.07),
            },
        );

        let r_hip_offset = nalgebra_glm::vec3(0.0, 0.0, -HIP_SPREAD);
        let r_hip_rot = nalgebra_glm::quat_angle_axis(right_hip_angle, &z_axis);
        let r_upper_leg_dir = nalgebra_glm::quat_rotate_vec3(&r_hip_rot, &down);
        world.assign_local_transform(
            self.body.right_upper_leg,
            LocalTransform {
                translation: hip + r_hip_offset + r_upper_leg_dir * (UPPER_LEG_LENGTH * 0.5),
                rotation: r_hip_rot,
                scale: nalgebra_glm::vec3(0.08, UPPER_LEG_LENGTH, 0.08),
            },
        );

        let r_knee_pos = hip + r_hip_offset + r_upper_leg_dir * UPPER_LEG_LENGTH;
        let r_knee_rot = r_hip_rot * nalgebra_glm::quat_angle_axis(right_knee_angle, &z_axis);
        let r_lower_leg_dir = nalgebra_glm::quat_rotate_vec3(&r_knee_rot, &down);
        world.assign_local_transform(
            self.body.right_lower_leg,
            LocalTransform {
                translation: r_knee_pos + r_lower_leg_dir * (LOWER_LEG_LENGTH * 0.5),
                rotation: r_knee_rot,
                scale: nalgebra_glm::vec3(0.07, LOWER_LEG_LENGTH, 0.07),
            },
        );
    }
}

fn ensure_eye_materials(world: &mut World) {
    if world
        .resources
        .material_registry
        .registry
        .name_to_index
        .contains_key("eye_white")
    {
        return;
    }

    material_registry_insert(
        &mut world.resources.material_registry,
        "eye_white".to_string(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.3,
            metallic: 0.0,
            ..Default::default()
        },
    );

    material_registry_insert(
        &mut world.resources.material_registry,
        "eye_pupil".to_string(),
        Material {
            base_color: [0.02, 0.02, 0.02, 1.0],
            roughness: 0.2,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

fn compute_foot_position(
    hip_angle: f32,
    knee_angle: f32,
    hip_offset: nalgebra_glm::Vec3,
) -> nalgebra_glm::Vec3 {
    let z_axis = nalgebra_glm::vec3(0.0, 0.0, 1.0);
    let down = nalgebra_glm::vec3(0.0, -1.0, 0.0);

    let hip_rot = nalgebra_glm::quat_angle_axis(hip_angle, &z_axis);
    let upper_leg_dir = nalgebra_glm::quat_rotate_vec3(&hip_rot, &down);
    let knee_pos = hip_offset + upper_leg_dir * UPPER_LEG_LENGTH;

    let knee_rot = hip_rot * nalgebra_glm::quat_angle_axis(knee_angle, &z_axis);
    let lower_leg_dir = nalgebra_glm::quat_rotate_vec3(&knee_rot, &down);

    knee_pos + lower_leg_dir * LOWER_LEG_LENGTH
}

fn apply_ground_contact(
    velocity: &mut nalgebra_glm::Vec3,
    foot_world: nalgebra_glm::Vec3,
    foot_relative_vel: nalgebra_glm::Vec3,
    delta_time: f32,
    ground_height: f32,
) {
    if foot_world.y < ground_height {
        let penetration = ground_height - foot_world.y;
        velocity.y += GROUND_SPRING * penetration * delta_time;
        velocity.y *= 1.0 - (GROUND_DAMPING * delta_time).min(0.95);

        let thrust = -foot_relative_vel.x * GROUND_GRIP * delta_time;
        if thrust > 0.0 {
            velocity.x += thrust;
        }
    }
}

fn spawn_body_part(world: &mut World, mesh_name: &str, material_name: &str) -> Entity {
    let entity = nightshade::ecs::world::commands::spawn_mesh_at(
        world,
        mesh_name,
        nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(1.0, 1.0, 1.0),
    );

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
    world.set_material_ref(entity, MaterialRef::new(material_name));

    entity
}
