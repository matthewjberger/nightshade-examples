use nightshade::prelude::*;

use crate::goap::{GoalType, PlannedAction};

#[derive(Clone, Debug, Default)]
pub struct Agent {
    pub name: String,
    pub position: Vec3,
    pub target_position: Option<Vec3>,
    pub waypoint_path: Vec<usize>,
    pub waypoint_index: usize,
    pub health: f32,
    pub wounded: bool,
    pub speed: f32,
    pub current_plan: Vec<PlannedAction>,
    pub current_step: usize,
    pub action_progress: f32,
    pub carrying: Option<CarriedItem>,
    pub state: AgentState,
    pub replan_timer: f32,
    pub replan_reason: String,
    pub body: AgentBody,
    pub carried_item_entity: Option<Entity>,
    pub plan_generation: u32,
    pub current_goal: Option<GoalType>,
    pub idle_timer: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AgentState {
    #[default]
    Idle,
    Moving,
    Performing,
    Replanning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CarriedItem {
    Water,
    RepairMaterials,
    Arrows,
}

impl std::fmt::Display for CarriedItem {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CarriedItem::Water => write!(formatter, "Water"),
            CarriedItem::RepairMaterials => write!(formatter, "Repair Materials"),
            CarriedItem::Arrows => write!(formatter, "Arrows"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AgentBody {
    pub head: Entity,
    pub torso: Entity,
    pub left_arm: Entity,
    pub right_arm: Entity,
    pub left_leg: Entity,
    pub right_leg: Entity,
    pub goal_marker: Entity,
}
