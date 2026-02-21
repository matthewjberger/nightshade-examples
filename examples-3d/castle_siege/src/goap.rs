use std::cmp::Ordering;
use std::collections::BinaryHeap;

use nightshade::prelude::Vec3;

use crate::ecs::LocationId;

pub const CARRYING_WATER: u32 = 1 << 0;
pub const CARRYING_REPAIR: u32 = 1 << 1;
pub const CARRYING_ARROWS: u32 = 1 << 2;
pub const WELL_HAS_WATER: u32 = 1 << 3;
pub const ARMORY_EXISTS: u32 = 1 << 4;
pub const HEALING_EXISTS: u32 = 1 << 5;
pub const REPAIR_PILE_EXISTS: u32 = 1 << 6;
pub const BACK_GATE_INTACT: u32 = 1 << 7;
pub const PATH_TO_RIVER_CLEAR: u32 = 1 << 8;
pub const AGENT_WOUNDED: u32 = 1 << 9;
pub const RUBBLE_EXISTS: u32 = 1 << 10;
pub const FIRE_DOUSED: u32 = 1 << 11;
pub const WALL_REPAIRED: u32 = 1 << 12;
pub const ARCHER_RESUPPLIED: u32 = 1 << 13;
pub const GATE_REINFORCED: u32 = 1 << 14;

#[derive(Clone, Debug, Default)]
pub struct GoapWorldState {
    pub flags_set: u32,
    pub flags_clear: u32,
}

impl GoapWorldState {
    pub fn set_flag(&mut self, flag: u32) {
        self.flags_set |= flag;
        self.flags_clear &= !flag;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionTarget {
    Well,
    River,
    Fire,
    RepairPile,
    RubbleNearest,
    Breach,
    Armory,
    ArcherPost,
    Gate,
    HealStation,
    BackGate,
}

#[derive(Clone, Debug)]
pub struct GoapAction {
    pub name: &'static str,
    pub precondition_set: u32,
    pub precondition_clear: u32,
    pub effect_set: u32,
    pub effect_clear: u32,
    pub cost: f32,
    pub duration: f32,
    pub target: ActionTarget,
}

pub fn build_action_table() -> Vec<GoapAction> {
    vec![
        GoapAction {
            name: "FetchWaterWell",
            precondition_set: WELL_HAS_WATER,
            precondition_clear: 0,
            effect_set: CARRYING_WATER,
            effect_clear: 0,
            cost: 1.0,
            duration: 1.5,
            target: ActionTarget::Well,
        },
        GoapAction {
            name: "FetchWaterRiver",
            precondition_set: BACK_GATE_INTACT | PATH_TO_RIVER_CLEAR,
            precondition_clear: 0,
            effect_set: CARRYING_WATER,
            effect_clear: 0,
            cost: 3.0,
            duration: 2.5,
            target: ActionTarget::River,
        },
        GoapAction {
            name: "DouseFire",
            precondition_set: CARRYING_WATER,
            precondition_clear: 0,
            effect_set: FIRE_DOUSED,
            effect_clear: CARRYING_WATER,
            cost: 2.0,
            duration: 1.0,
            target: ActionTarget::Fire,
        },
        GoapAction {
            name: "FetchRepairMaterials",
            precondition_set: REPAIR_PILE_EXISTS,
            precondition_clear: 0,
            effect_set: CARRYING_REPAIR,
            effect_clear: 0,
            cost: 1.0,
            duration: 1.5,
            target: ActionTarget::RepairPile,
        },
        GoapAction {
            name: "SalvageRubble",
            precondition_set: RUBBLE_EXISTS,
            precondition_clear: 0,
            effect_set: REPAIR_PILE_EXISTS,
            effect_clear: 0,
            cost: 2.5,
            duration: 3.0,
            target: ActionTarget::RubbleNearest,
        },
        GoapAction {
            name: "RepairWall",
            precondition_set: CARRYING_REPAIR,
            precondition_clear: 0,
            effect_set: WALL_REPAIRED,
            effect_clear: CARRYING_REPAIR,
            cost: 3.0,
            duration: 3.5,
            target: ActionTarget::Breach,
        },
        GoapAction {
            name: "ClearRubble",
            precondition_set: 0,
            precondition_clear: 0,
            effect_set: PATH_TO_RIVER_CLEAR,
            effect_clear: 0,
            cost: 2.0,
            duration: 2.5,
            target: ActionTarget::RubbleNearest,
        },
        GoapAction {
            name: "FetchArrows",
            precondition_set: ARMORY_EXISTS,
            precondition_clear: 0,
            effect_set: CARRYING_ARROWS,
            effect_clear: 0,
            cost: 1.0,
            duration: 1.5,
            target: ActionTarget::Armory,
        },
        GoapAction {
            name: "ResupplyArcher",
            precondition_set: CARRYING_ARROWS,
            precondition_clear: 0,
            effect_set: ARCHER_RESUPPLIED,
            effect_clear: CARRYING_ARROWS,
            cost: 1.5,
            duration: 1.0,
            target: ActionTarget::ArcherPost,
        },
        GoapAction {
            name: "ReinforceGate",
            precondition_set: CARRYING_REPAIR,
            precondition_clear: 0,
            effect_set: GATE_REINFORCED,
            effect_clear: CARRYING_REPAIR,
            cost: 3.0,
            duration: 3.0,
            target: ActionTarget::Gate,
        },
        GoapAction {
            name: "RepairBackGate",
            precondition_set: CARRYING_REPAIR,
            precondition_clear: 0,
            effect_set: BACK_GATE_INTACT,
            effect_clear: CARRYING_REPAIR,
            cost: 3.0,
            duration: 3.5,
            target: ActionTarget::BackGate,
        },
        GoapAction {
            name: "TendWounded",
            precondition_set: HEALING_EXISTS,
            precondition_clear: 0,
            effect_set: 0,
            effect_clear: AGENT_WOUNDED,
            cost: 4.0,
            duration: 4.0,
            target: ActionTarget::HealStation,
        },
    ]
}

#[derive(Clone, Debug)]
pub struct PlannedAction {
    pub action: GoapAction,
    pub resolved_target: Option<LocationId>,
    pub target_position: Option<Vec3>,
}

#[derive(Clone, Debug)]
struct PlannerNode {
    pub unsatisfied_set: u32,
    pub unsatisfied_clear: u32,
    pub actions: Vec<GoapAction>,
    pub cost: f32,
    pub heuristic: f32,
}

impl PartialEq for PlannerNode {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost().to_bits() == other.total_cost().to_bits()
    }
}

impl Eq for PlannerNode {}

impl PlannerNode {
    fn total_cost(&self) -> f32 {
        self.cost + self.heuristic
    }
}

impl PartialOrd for PlannerNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlannerNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .total_cost()
            .partial_cmp(&self.total_cost())
            .unwrap_or(Ordering::Equal)
    }
}

fn count_unsatisfied(set: u32, clear: u32, current: &GoapWorldState) -> u32 {
    let unmet_set = set & !current.flags_set;
    let unmet_clear = clear & current.flags_set;
    unmet_set.count_ones() + unmet_clear.count_ones()
}

fn is_satisfied(set: u32, clear: u32, current: &GoapWorldState) -> bool {
    (set & !current.flags_set) == 0 && (clear & current.flags_set) == 0
}

pub fn plan_for_goal(
    current_state: &GoapWorldState,
    goal_set: u32,
    goal_clear: u32,
    actions: &[GoapAction],
    max_depth: usize,
) -> Option<Vec<PlannedAction>> {
    if is_satisfied(goal_set, goal_clear, current_state) {
        return Some(Vec::new());
    }

    let mut open = BinaryHeap::new();
    let heuristic = count_unsatisfied(goal_set, goal_clear, current_state) as f32 * 1.5;

    open.push(PlannerNode {
        unsatisfied_set: goal_set,
        unsatisfied_clear: goal_clear,
        actions: Vec::new(),
        cost: 0.0,
        heuristic,
    });

    let mut iterations = 0;
    let max_iterations = 500;

    while let Some(node) = open.pop() {
        iterations += 1;
        if iterations > max_iterations {
            return None;
        }

        if node.actions.len() >= max_depth {
            continue;
        }

        let unmet_set = node.unsatisfied_set & !current_state.flags_set;
        let unmet_clear = node.unsatisfied_clear & current_state.flags_set;

        if unmet_set == 0 && unmet_clear == 0 {
            let planned: Vec<PlannedAction> = node
                .actions
                .into_iter()
                .rev()
                .map(|action| PlannedAction {
                    action,
                    resolved_target: None,
                    target_position: None,
                })
                .collect();
            return Some(planned);
        }

        for action in actions {
            let satisfies_set = action.effect_set & unmet_set;
            let satisfies_clear = action.effect_clear & unmet_clear;

            if satisfies_set == 0 && satisfies_clear == 0 {
                continue;
            }

            let mut new_unsat_set = node.unsatisfied_set;
            let mut new_unsat_clear = node.unsatisfied_clear;

            new_unsat_set &= !action.effect_set;
            new_unsat_clear &= !action.effect_clear;

            new_unsat_set |= action.precondition_set;
            new_unsat_clear |= action.precondition_clear;

            let new_cost = node.cost + action.cost;
            let new_heuristic =
                count_unsatisfied(new_unsat_set, new_unsat_clear, current_state) as f32 * 1.5;

            let mut new_actions = node.actions.clone();
            new_actions.push(action.clone());

            open.push(PlannerNode {
                unsatisfied_set: new_unsat_set,
                unsatisfied_clear: new_unsat_clear,
                actions: new_actions,
                cost: new_cost,
                heuristic: new_heuristic,
            });
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GoalType {
    ExtinguishFires,
    RepairWalls,
    ResupplyArchers,
    ReinforceGate,
    ClearRubblePaths,
    TendWounded,
}

impl std::fmt::Display for GoalType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalType::ExtinguishFires => write!(formatter, "Extinguish Fires"),
            GoalType::RepairWalls => write!(formatter, "Repair Walls"),
            GoalType::ResupplyArchers => write!(formatter, "Resupply Archers"),
            GoalType::ReinforceGate => write!(formatter, "Reinforce Gate"),
            GoalType::ClearRubblePaths => write!(formatter, "Clear Rubble"),
            GoalType::TendWounded => write!(formatter, "Tend Wounded"),
        }
    }
}

impl GoalType {
    pub fn color(&self) -> [u8; 3] {
        match self {
            GoalType::ExtinguishFires => [255, 120, 40],
            GoalType::RepairWalls => [180, 160, 100],
            GoalType::ResupplyArchers => [120, 80, 50],
            GoalType::ReinforceGate => [140, 100, 60],
            GoalType::ClearRubblePaths => [160, 140, 110],
            GoalType::TendWounded => [80, 200, 120],
        }
    }
}

pub struct GoalSelectionContext<'a> {
    pub current_state: &'a GoapWorldState,
    pub actions: &'a [GoapAction],
    pub fire_count: usize,
    pub breach_count: usize,
    pub archer_posts_empty: usize,
    pub gate_damage_level: i32,
    pub agent_wounded: bool,
    pub rubble_blocking: bool,
    pub claimed_goals: &'a [(usize, GoalType)],
    pub agent_index: usize,
}

pub fn select_goal_and_plan(ctx: &GoalSelectionContext) -> Option<(GoalType, Vec<PlannedAction>)> {
    let fire_count = ctx.fire_count;
    let breach_count = ctx.breach_count;
    let archer_posts_empty = ctx.archer_posts_empty;
    let gate_damage_level = ctx.gate_damage_level;
    let agent_wounded = ctx.agent_wounded;
    let rubble_blocking = ctx.rubble_blocking;
    let agent_index = ctx.agent_index;
    let claimed_goals = ctx.claimed_goals;
    let current_state = ctx.current_state;
    let actions = ctx.actions;

    let goals_in_priority = [
        (GoalType::ExtinguishFires, fire_count > 0),
        (GoalType::RepairWalls, breach_count > 0),
        (GoalType::ResupplyArchers, archer_posts_empty > 0),
        (GoalType::ReinforceGate, gate_damage_level > 1),
        (GoalType::ClearRubblePaths, rubble_blocking),
        (GoalType::TendWounded, agent_wounded),
    ];

    for (goal, relevant) in &goals_in_priority {
        if !relevant {
            continue;
        }

        let already_claimed = claimed_goals
            .iter()
            .any(|(idx, g)| *g == *goal && *idx != agent_index);

        let enough_claimants = claimed_goals
            .iter()
            .filter(|(idx, g)| *g == *goal && *idx != agent_index)
            .count();

        let max_per_goal = match goal {
            GoalType::ExtinguishFires => 3,
            GoalType::RepairWalls => 2,
            _ => 1,
        };

        if already_claimed && enough_claimants >= max_per_goal {
            continue;
        }

        let (goal_set, goal_clear) = match goal {
            GoalType::ExtinguishFires => (FIRE_DOUSED, 0u32),
            GoalType::RepairWalls => (WALL_REPAIRED, 0u32),
            GoalType::ResupplyArchers => (ARCHER_RESUPPLIED, 0u32),
            GoalType::ReinforceGate => (GATE_REINFORCED, 0u32),
            GoalType::ClearRubblePaths => (PATH_TO_RIVER_CLEAR, 0u32),
            GoalType::TendWounded => (0u32, AGENT_WOUNDED),
        };

        if let Some(plan) = plan_for_goal(current_state, goal_set, goal_clear, actions, 6) {
            return Some((*goal, plan));
        }
    }

    None
}
