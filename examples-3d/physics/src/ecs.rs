mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::Entity;
use nightshade::prelude::Vec3;
use nightshade::prelude::freecs;

freecs::ecs! {
    GameWorld {
        door: Door => DOOR,
        drawer: Drawer => DRAWER,
        lever: Lever => LEVER,
        wheel: Wheel => WHEEL,
        button: Button => BUTTON,
        note: Note => NOTE,
        bauble_spawn: BaubleSpawn => BAUBLE_SPAWN,
        shot_bauble: ShotBauble => SHOT_BAUBLE,
        prismatic_slider: PrismaticSlider => PRISMATIC_SLIDER,
        spherical_joint_visual: SphericalJointVisual => SPHERICAL_JOINT_VISUAL,
        rope_joint_visual: RopeJointVisual => ROPE_JOINT_VISUAL,
        spring_joint_visual: SpringJointVisual => SPRING_JOINT_VISUAL,
        coulomb_friction_joint: CoulombFrictionJoint => COULOMB_FRICTION_JOINT,
        target: Target => TARGET,
        velocity_friction_joint: VelocityFrictionJoint => VELOCITY_FRICTION_JOINT,
        interactable: Interactable => INTERACTABLE,
    }
    Tags {
    }
    Events {
        target_killed: TargetKilledEvent,
    }
    Resources {
        config: GameConfig,
        player: PlayerResources,
        interaction: InteractionState,
        grab: nightshade::ecs::physics::grab::GrabState,
        #[cfg(not(feature = "openxr"))]
        weapon: WeaponState,
        flashlight: FlashlightState,
        ui: UiHandles,
        show_physics_debug: bool,
        bauble_table_center: Vec3,
        bauble_table_top_y: f32,
        lantern_entity: Option<Entity>,
        lantern_light_entity: Option<Entity>,
        #[cfg(feature = "openxr")]
        left_hand_cube: Option<Entity>,
        #[cfg(feature = "openxr")]
        gun_root_entity: Option<Entity>,
    }
}
