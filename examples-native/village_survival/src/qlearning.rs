use crate::agent::AgentNeeds;
use crate::genome::Genome;
use rand::Rng;

pub const NUM_STATES: usize = 216;
pub const NUM_ACTIONS: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    GoToFood,
    GoToRest,
    GoHome,
    Build,
    Wander,
    Flee,
}

impl Action {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Action::GoToFood,
            1 => Action::GoToRest,
            2 => Action::GoHome,
            3 => Action::Build,
            4 => Action::Wander,
            _ => Action::Flee,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Action::GoToFood => 0,
            Action::GoToRest => 1,
            Action::GoHome => 2,
            Action::Build => 3,
            Action::Wander => 4,
            Action::Flee => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Action::GoToFood => "Eating",
            Action::GoToRest => "Resting",
            Action::GoHome => "Going Home",
            Action::Build => "Building",
            Action::Wander => "Wandering",
            Action::Flee => "Fleeing",
        }
    }
}

#[derive(Clone)]
pub struct QTable {
    pub table: Vec<[f32; NUM_ACTIONS]>,
    pub alpha: f32,
    pub gamma: f32,
    pub epsilon: f32,
}

impl QTable {
    pub fn new() -> Self {
        Self {
            table: vec![[0.0; NUM_ACTIONS]; NUM_STATES],
            alpha: 0.15,
            gamma: 0.9,
            epsilon: 0.4,
        }
    }

    pub fn decay_epsilon(&mut self) {
        self.epsilon = (self.epsilon * 0.995).max(0.05);
    }

    pub fn reset_epsilon(&mut self) {
        self.epsilon = 0.15;
    }
}

pub struct AgentState {
    pub hunger_bucket: usize,
    pub energy_bucket: usize,
    pub loneliness_bucket: usize,
    pub threat_nearby: bool,
    pub at_home: bool,
    pub is_night: bool,
}

impl AgentState {
    pub fn encode(&self) -> usize {
        self.hunger_bucket * 72
            + self.energy_bucket * 24
            + self.loneliness_bucket * 8
            + (self.threat_nearby as usize) * 4
            + (self.at_home as usize) * 2
            + self.is_night as usize
    }
}

pub fn bucket(value: f32) -> usize {
    if value < 0.33 {
        0
    } else if value < 0.66 {
        1
    } else {
        2
    }
}

pub fn select_action(q_table: &QTable, state_index: usize, rng: &mut impl Rng) -> Action {
    if rng.random::<f32>() < q_table.epsilon {
        Action::from_index(rng.random_range(0..NUM_ACTIONS))
    } else {
        let q_values = &q_table.table[state_index];
        let best_index = q_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap_or(0);
        Action::from_index(best_index)
    }
}

pub fn update_q_value(
    q_table: &mut QTable,
    state_index: usize,
    action: Action,
    reward: f32,
    next_state_index: usize,
) {
    let max_next_q = q_table.table[next_state_index]
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);

    let action_index = action.to_index();
    let current_q = q_table.table[state_index][action_index];
    q_table.table[state_index][action_index] +=
        q_table.alpha * (reward + q_table.gamma * max_next_q - current_q);
}

pub struct RewardContext {
    pub needs: AgentNeeds,
    pub in_food_zone: bool,
    pub in_rest_zone: bool,
    pub near_wolf: bool,
    pub fleeing_wolf: bool,
    pub died: bool,
    pub at_home: bool,
    pub near_campfire: bool,
    pub nearby_agents: usize,
    pub building_safe: bool,
}

pub fn compute_reward(genome: &Genome, context: &RewardContext) -> f32 {
    let mut reward = 0.0;

    let worst_need = context.needs.worst();
    reward -= 3.0 * worst_need;

    if context.in_food_zone {
        reward += 5.0 * context.needs.hunger;
    }
    if context.in_rest_zone {
        reward += 5.0 * context.needs.energy;
    }

    if worst_need < 0.3 {
        reward += 1.0;
    }

    if context.near_wolf {
        reward -= 10.0 * (1.0 - genome.boldness);
    }
    if context.fleeing_wolf {
        reward += 4.0 * (1.0 - genome.boldness);
    }

    if context.at_home && context.needs.energy < 0.3 {
        reward += 2.0 * (1.0 - context.needs.energy);
    }

    if context.building_safe {
        reward += 1.5 * (1.0 + genome.home_investment);
    }

    if context.near_campfire && context.near_wolf {
        reward += 3.0;
    }

    reward -= 2.0 * context.needs.loneliness;

    if context.nearby_agents >= 2 {
        reward += 0.5;
    }

    if context.died {
        reward -= 100.0;
    }

    reward
}
