use nightshade::prelude::Entity;
use nightshade::prelude::Vec3;
use nightshade::prelude::rapier3d;

#[derive(Default, Clone, Debug)]
pub struct Door {
    pub entity: Entity,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub hinge_position: Vec3,
    pub door_half_width: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Drawer {
    pub entity: Entity,
    pub front_entity: Entity,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub closed_position: Vec3,
    pub current_offset: f32,
    pub velocity: f32,
    pub max_offset: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Lever {
    pub pivot_entity: Entity,
    pub collider_entity: Entity,
    pub collider_rb_handle: rapier3d::prelude::RigidBodyHandle,
    pub pivot_position: Vec3,
    pub arm_half_length: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Wheel {
    pub entity: Entity,
    pub spoke_entities: Vec<Entity>,
    pub rigid_body_handle: rapier3d::prelude::RigidBodyHandle,
    pub center_position: Vec3,
    pub current_angle: f32,
    pub angular_velocity: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Button {
    pub entity: Entity,
    pub base_position: Vec3,
    pub current_press: f32,
    pub is_pressed: bool,
    pub action: ButtonAction,
}

#[derive(Default, Clone, Debug)]
pub enum ButtonAction {
    #[default]
    RecallBaubles,
}

#[derive(Default, Clone, Debug)]
pub struct Note {
    pub entity: Entity,
    pub title: String,
    pub content: String,
}

#[derive(Default, Clone, Debug)]
pub struct BaubleSpawn {
    pub entity: Entity,
    pub spawn_position: Vec3,
}

#[derive(Default, Clone, Debug)]
pub struct ShotBauble {
    pub entity: Entity,
    pub spawn_time_ms: u64,
    pub original_scale: f32,
    pub landed: bool,
}

#[derive(Default, Clone, Debug)]
pub struct PrismaticSlider {
    pub entity: Entity,
    pub time_accumulator: f32,
}

#[derive(Default, Clone, Debug)]
pub struct SphericalJointVisual {
    pub anchor_entity: Entity,
    pub ball_entity: Entity,
    pub rod_entity: Entity,
}

#[derive(Default, Clone, Debug)]
pub struct RopeJointVisual {
    pub anchor_entity: Entity,
    pub ball_entity: Entity,
    pub rope_entity: Entity,
}

#[derive(Default, Clone, Debug)]
pub struct SpringJointVisual {
    pub anchor_entity: Entity,
    pub object_entity: Entity,
    pub spring_entities: Vec<Entity>,
}

#[derive(Default, Clone, Debug)]
pub struct CoulombFrictionJoint {
    pub arm_entity: Entity,
    pub friction_torque: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub bar_entity: Entity,
    pub fill_entity: Entity,
}

impl Health {
    pub fn new(max: f32, bar_entity: Entity, fill_entity: Entity) -> Self {
        Self {
            current: max,
            max,
            bar_entity,
            fill_entity,
        }
    }

    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

#[derive(Default, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub position: Vec3,
    pub base_scale: f32,
    pub color: Vec3,
    pub health: Health,
    pub popped: bool,
    pub pop_time_ms: u64,
    pub respawn_delay_ms: u64,
    pub pulse_phase: f32,
    pub pop_emitter_entity: Option<Entity>,
}

#[derive(Default, Clone, Debug)]
pub struct VelocityFrictionJoint {
    pub arm_entity: Entity,
    pub damping_factor: f32,
    pub initialized: bool,
}
