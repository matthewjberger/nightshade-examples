use nightshade::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Warmup,
    Escalation,
    Crescendo,
}

impl std::fmt::Display for Act {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Act::Warmup => write!(formatter, "Act I: Warmup"),
            Act::Escalation => write!(formatter, "Act II: Escalation"),
            Act::Crescendo => write!(formatter, "Act III: Crescendo"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BombardmentState {
    pub current_act: Act,
    pub boulder_timer: f32,
    pub boulder_interval: f32,
    pub total_boulders_fired: u32,
    pub rng_seed: u64,
}

impl Default for BombardmentState {
    fn default() -> Self {
        Self {
            current_act: Act::Warmup,
            boulder_timer: 3.0,
            boulder_interval: 10.0,
            total_boulders_fired: 0,
            rng_seed: 12345,
        }
    }
}

impl BombardmentState {
    pub fn next_random(&mut self) -> f32 {
        self.rng_seed = self
            .rng_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_seed >> 33) as f32) / (u32::MAX as f32)
    }

    pub fn next_random_range(&mut self, min: f32, max: f32) -> f32 {
        let value = self.next_random();
        min + value * (max - min)
    }

    pub fn update_act(&mut self, elapsed: f32) {
        let (new_act, min_interval, max_interval) = if elapsed < 45.0 {
            (Act::Warmup, 8.0, 12.0)
        } else if elapsed < 225.0 {
            (Act::Escalation, 3.0, 5.0)
        } else {
            (Act::Crescendo, 1.0, 2.0)
        };

        if self.current_act != new_act {
            self.current_act = new_act;
        }

        self.boulder_interval = self.next_random_range(min_interval, max_interval);
    }

    pub fn pick_target(&mut self) -> Vec3 {
        let choice = self.next_random();
        if choice < 0.4 {
            let side = (self.next_random() * 4.0) as usize;
            let offset = self.next_random_range(-12.0, 12.0);
            match side {
                0 => nalgebra_glm::vec3(offset, 0.0, -15.0),
                1 => nalgebra_glm::vec3(offset, 0.0, 15.0),
                2 => nalgebra_glm::vec3(15.0, 0.0, offset),
                _ => nalgebra_glm::vec3(-15.0, 0.0, offset),
            }
        } else if choice < 0.6 {
            nalgebra_glm::vec3(self.next_random_range(-2.0, 2.0), 0.0, 15.0)
        } else {
            nalgebra_glm::vec3(
                self.next_random_range(-10.0, 10.0),
                0.0,
                self.next_random_range(-10.0, 10.0),
            )
        }
    }

    pub fn pick_spawn_position(&mut self, target: &Vec3) -> Vec3 {
        let angle = self.next_random_range(0.5, 2.6);
        let distance = 40.0;
        nalgebra_glm::vec3(
            target.x + angle.cos() * distance,
            25.0 + self.next_random_range(0.0, 10.0),
            target.z + angle.sin() * distance,
        )
    }
}
