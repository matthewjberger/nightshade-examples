use nightshade::prelude::*;

use crate::agent::Agent;
use crate::bombardment::BombardmentState;
use crate::castle::CastleState;
use crate::pathfinding::WaypointGraph;

#[derive(Default, Clone, Debug)]
pub struct EntityHandle(pub Entity);

#[derive(Default, Clone, Debug)]
pub struct Boulder {
    pub start: Vec3,
    pub target: Vec3,
    pub arc_height: f32,
    pub progress: f32,
    pub speed: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Fire {
    pub position: Vec3,
    pub spread_timer: f32,
    pub entities: Vec<Entity>,
    pub light_entity: Option<Entity>,
    pub smoke_entity: Option<Entity>,
    pub near_location: Option<LocationId>,
    pub doused_amount: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LocationId {
    #[default]
    Well,
    Armory,
    HealingStation,
    RepairPile,
    Gate,
    River,
    ArcherPost(usize),
    WallNorth,
    WallSouth,
    WallEast,
    WallWest,
}

#[derive(Default, Clone, Debug)]
pub struct Rubble {
    pub position: Vec3,
    pub entities: Vec<Entity>,
    pub blocks_path: Option<(usize, usize)>,
}

#[derive(Default, Clone, Debug)]
pub struct ReplanRing {
    pub entity: Entity,
    pub timer: f32,
    pub max_time: f32,
}

#[derive(Default, Clone, Debug)]
pub struct EnemyInvader {
    pub position: Vec3,
    pub entity: Entity,
    pub velocity: Vec3,
}

#[derive(Clone, Debug)]
pub struct TimedEffect {
    pub entity: Entity,
    pub timer: f32,
    pub max_time: f32,
}

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        agent: Agent => AGENT,
        boulder: Boulder => BOULDER,
        fire: Fire => FIRE,
        rubble: Rubble => RUBBLE,
        replan_ring: ReplanRing => REPLAN_RING,
        enemy_invader: EnemyInvader => ENEMY_INVADER,
    }
    GameResources {
        castle: CastleState,
        bombardment: BombardmentState,
        agents: Vec<freecs::Entity>,
        boulders: Vec<freecs::Entity>,
        fires: Vec<freecs::Entity>,
        rubble_list: Vec<freecs::Entity>,
        replan_rings: Vec<freecs::Entity>,
        invaders: Vec<freecs::Entity>,
        elapsed_time: f32,
        game_speed: f32,
        selected_agent: Option<usize>,
        planner_feed_dirty: bool,
        failure_triggered: bool,
        failure_timer: f32,
        failure_invaders_spawned: bool,
        survival_time: f32,
        camera_entity: Entity,
        waypoints: WaypointGraph,
        manual_boulder_requested: bool,
        manual_burn_armory: bool,
        manual_drain_well: bool,
        paused: bool,
        impact_flashes: Vec<TimedEffect>,
        trail_particles: Vec<TimedEffect>,
        restart_requested: bool,
        claimed_fire_targets: Vec<(usize, freecs::Entity)>,
    }
}
