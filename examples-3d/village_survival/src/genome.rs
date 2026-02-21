use rand::Rng;

#[derive(Clone)]
pub struct Genome {
    pub boldness: f32,
    pub sociability: f32,
    pub metabolism: f32,
    pub wander_range: f32,
    pub home_investment: f32,
}

impl Genome {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            boldness: rng.random_range(0.2..0.8),
            sociability: rng.random_range(0.2..0.8),
            metabolism: rng.random_range(0.2..0.8),
            wander_range: rng.random_range(0.2..0.8),
            home_investment: rng.random_range(0.2..0.8),
        }
    }

    pub fn crossover(parent_a: &Genome, parent_b: &Genome, rng: &mut impl Rng) -> Self {
        Self {
            boldness: if rng.random_bool(0.5) {
                parent_a.boldness
            } else {
                parent_b.boldness
            },
            sociability: if rng.random_bool(0.5) {
                parent_a.sociability
            } else {
                parent_b.sociability
            },
            metabolism: if rng.random_bool(0.5) {
                parent_a.metabolism
            } else {
                parent_b.metabolism
            },
            wander_range: if rng.random_bool(0.5) {
                parent_a.wander_range
            } else {
                parent_b.wander_range
            },
            home_investment: if rng.random_bool(0.5) {
                parent_a.home_investment
            } else {
                parent_b.home_investment
            },
        }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) {
        let mutation_chance = 0.2;
        let noise_scale = 0.15;

        if rng.random::<f32>() < mutation_chance {
            self.boldness = (self.boldness + gaussian_noise(rng, noise_scale)).clamp(0.0, 1.0);
        }
        if rng.random::<f32>() < mutation_chance {
            self.sociability =
                (self.sociability + gaussian_noise(rng, noise_scale)).clamp(0.0, 1.0);
        }
        if rng.random::<f32>() < mutation_chance {
            self.metabolism = (self.metabolism + gaussian_noise(rng, noise_scale)).clamp(0.0, 1.0);
        }
        if rng.random::<f32>() < mutation_chance {
            self.wander_range =
                (self.wander_range + gaussian_noise(rng, noise_scale)).clamp(0.0, 1.0);
        }
        if rng.random::<f32>() < mutation_chance {
            self.home_investment =
                (self.home_investment + gaussian_noise(rng, noise_scale)).clamp(0.0, 1.0);
        }
    }

    pub fn trait_averages(genomes: &[Genome]) -> [f32; 5] {
        if genomes.is_empty() {
            return [0.0; 5];
        }
        let count = genomes.len() as f32;
        let mut sums = [0.0f32; 5];
        for genome in genomes {
            sums[0] += genome.boldness;
            sums[1] += genome.sociability;
            sums[2] += genome.metabolism;
            sums[3] += genome.wander_range;
            sums[4] += genome.home_investment;
        }
        [
            sums[0] / count,
            sums[1] / count,
            sums[2] / count,
            sums[3] / count,
            sums[4] / count,
        ]
    }
}

fn gaussian_noise(rng: &mut impl Rng, scale: f32) -> f32 {
    let u1: f32 = rng.random::<f32>().max(1e-10);
    let u2: f32 = rng.random::<f32>();
    let normal = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    normal * scale
}

pub fn tournament_select<'a>(
    agents: &'a [(Genome, f32)],
    tournament_size: usize,
    rng: &mut impl Rng,
) -> &'a Genome {
    let mut best_index = rng.random_range(0..agents.len());
    let mut best_fitness = agents[best_index].1;

    for _ in 1..tournament_size {
        let candidate_index = rng.random_range(0..agents.len());
        if agents[candidate_index].1 > best_fitness {
            best_index = candidate_index;
            best_fitness = agents[candidate_index].1;
        }
    }

    &agents[best_index].0
}

pub fn produce_next_generation(
    agents: &[(Genome, f32)],
    count: usize,
    best_q_table: &crate::qlearning::QTable,
    rng: &mut impl Rng,
) -> Vec<(Genome, crate::qlearning::QTable)> {
    let mut offspring = Vec::with_capacity(count);

    for _ in 0..count {
        let parent_a = tournament_select(agents, 3, rng);
        let parent_b = tournament_select(agents, 3, rng);
        let mut child = Genome::crossover(parent_a, parent_b, rng);
        child.mutate(rng);
        let mut inherited_table = best_q_table.clone();
        inherited_table.reset_epsilon();
        offspring.push((child, inherited_table));
    }

    offspring
}
