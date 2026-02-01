use crate::data::items::Inventory;
use crate::data::skills::PlayerSkills;

#[derive(Clone)]
pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
    pub base_damage: f32,
    pub damage_multiplier: f32,
    pub defense: f32,
    pub speed_multiplier: f32,
    pub health_regen: f32,
    pub mana_regen: f32,
    pub skill_points: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            mana: 100.0,
            max_mana: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            level: 1,
            experience: 0,
            experience_to_next_level: 100,
            base_damage: 10.0,
            damage_multiplier: 1.0,
            defense: 0.0,
            speed_multiplier: 1.0,
            health_regen: 1.0,
            mana_regen: 2.0,
            skill_points: 0,
        }
    }
}

impl PlayerStats {
    pub fn add_experience(&mut self, amount: u32) -> bool {
        self.experience += amount;
        if self.experience >= self.experience_to_next_level {
            self.level_up();
            return true;
        }
        false
    }

    fn level_up(&mut self) {
        self.experience -= self.experience_to_next_level;
        self.level += 1;
        self.experience_to_next_level = self.calculate_exp_for_level(self.level + 1);

        self.max_health += 10.0;
        self.health = self.max_health;
        self.max_mana += 5.0;
        self.mana = self.max_mana;
        self.base_damage += 2.0;
        self.defense += 1.0;
        self.skill_points += 1;
    }

    fn calculate_exp_for_level(&self, level: u32) -> u32 {
        (100.0 * (level as f32).powf(1.5)) as u32
    }

    pub fn take_damage(&mut self, damage: f32) -> f32 {
        let actual_damage = (damage * (1.0 - self.defense / 100.0)).max(1.0);
        self.health = (self.health - actual_damage).max(0.0);
        actual_damage
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn use_mana(&mut self, amount: f32) -> bool {
        if self.mana >= amount {
            self.mana -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore_mana(&mut self, amount: f32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    pub fn update(&mut self, delta_time: f32) {
        self.health = (self.health + self.health_regen * delta_time).min(self.max_health);
        self.mana = (self.mana + self.mana_regen * delta_time).min(self.max_mana);
        self.stamina = (self.stamina + 10.0 * delta_time).min(self.max_stamina);
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn get_total_damage(&self) -> f32 {
        self.base_damage * self.damage_multiplier
    }
}

#[derive(Clone)]
pub struct PlayerProgress {
    pub stats: PlayerStats,
    pub inventory: Inventory,
    pub skills: PlayerSkills,
    pub deaths: u32,
    pub enemies_killed: u32,
    pub items_collected: u32,
    pub total_damage_dealt: f32,
    pub total_damage_taken: f32,
    pub play_time: f32,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            stats: PlayerStats::default(),
            inventory: Inventory::default(),
            skills: PlayerSkills::default(),
            deaths: 0,
            enemies_killed: 0,
            items_collected: 0,
            total_damage_dealt: 0.0,
            total_damage_taken: 0.0,
            play_time: 0.0,
        }
    }
}
