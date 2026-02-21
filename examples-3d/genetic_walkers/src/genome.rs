use rand::Rng;
use std::f32::consts::TAU;

pub const JOINT_COUNT: usize = 9;

pub const L_SHOULDER: usize = 0;
pub const R_SHOULDER: usize = 1;
pub const L_ELBOW: usize = 2;
pub const R_ELBOW: usize = 3;
pub const L_HIP: usize = 4;
pub const R_HIP: usize = 5;
pub const L_KNEE: usize = 6;
pub const R_KNEE: usize = 7;
pub const JUMP: usize = 8;

pub const JOINT_LABELS: [&str; JOINT_COUNT] = [
    "L.Sh", "R.Sh", "L.El", "R.El", "L.Hi", "R.Hi", "L.Kn", "R.Kn", "Jump",
];

#[derive(Clone, Copy, PartialEq)]
pub enum JointSource {
    ParentA,
    ParentB,
}

#[derive(Clone)]
pub enum BreedingOrigin {
    Elite(usize),
    Crossover {
        parent_a_fitness: f32,
        parent_b_fitness: f32,
    },
}

#[derive(Clone)]
pub struct BreedingRecord {
    pub origin: BreedingOrigin,
    pub parent_source: [JointSource; JOINT_COUNT],
    pub mutated_joints: [bool; JOINT_COUNT],
}

#[derive(Clone)]
pub struct JointGene {
    pub frequency: f32,
    pub phase: f32,
    pub amplitude: f32,
}

impl JointGene {
    pub fn evaluate(&self, time: f32) -> f32 {
        self.amplitude * (TAU * self.frequency * time + self.phase).sin()
    }
}

#[derive(Clone)]
pub struct Genome {
    pub joints: [JointGene; JOINT_COUNT],
}

impl Genome {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            joints: std::array::from_fn(|index| {
                let (freq_min, freq_max, amp_max) = match index {
                    L_HIP | R_HIP => (0.5, 4.0, 1.2),
                    L_KNEE | R_KNEE => (0.5, 4.0, 1.0),
                    L_SHOULDER | R_SHOULDER => (0.5, 4.0, 0.8),
                    L_ELBOW | R_ELBOW => (0.5, 4.0, 0.8),
                    JUMP => (0.3, 2.0, 1.0),
                    _ => (0.5, 4.0, 1.0),
                };
                JointGene {
                    frequency: rng.random_range(freq_min..freq_max),
                    phase: rng.random_range(0.0..TAU),
                    amplitude: rng.random_range(0.0..amp_max),
                }
            }),
        }
    }

    pub fn crossover(
        parent_a: &Genome,
        parent_b: &Genome,
        rng: &mut impl Rng,
    ) -> (Self, [JointSource; JOINT_COUNT]) {
        let mut sources = [JointSource::ParentA; JOINT_COUNT];
        let genome = Self {
            joints: std::array::from_fn(|index| {
                if rng.random_bool(0.5) {
                    parent_a.joints[index].clone()
                } else {
                    sources[index] = JointSource::ParentB;
                    parent_b.joints[index].clone()
                }
            }),
        };
        (genome, sources)
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) -> [bool; JOINT_COUNT] {
        let mutation_rate = 0.3;
        let mutation_strength = 0.2;
        let mut mutated = [false; JOINT_COUNT];

        for (index, joint) in self.joints.iter_mut().enumerate() {
            let mut was_mutated = false;
            if rng.random_bool(mutation_rate) {
                joint.frequency += rng.random_range(-0.5..0.5) * mutation_strength * 5.0;
                joint.frequency = joint.frequency.clamp(0.2, 6.0);
                was_mutated = true;
            }
            if rng.random_bool(mutation_rate) {
                joint.phase += rng.random_range(-0.5..0.5) * mutation_strength * TAU;
                joint.phase = joint.phase.rem_euclid(TAU);
                was_mutated = true;
            }
            if rng.random_bool(mutation_rate) {
                let amp_max = match index {
                    L_HIP | R_HIP => 1.5,
                    L_KNEE | R_KNEE => 1.2,
                    JUMP => 1.0,
                    _ => 1.0,
                };
                joint.amplitude += rng.random_range(-0.3..0.3) * mutation_strength;
                joint.amplitude = joint.amplitude.clamp(0.0, amp_max);
                was_mutated = true;
            }
            mutated[index] = was_mutated;
        }
        mutated
    }
}

pub fn select_and_breed(
    genomes: &[Genome],
    fitnesses: &[f32],
    population_size: usize,
    rng: &mut impl Rng,
) -> Vec<(Genome, BreedingRecord)> {
    let mut indexed: Vec<(usize, f32)> = fitnesses.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let elite_count = 3;
    let mut results: Vec<(Genome, BreedingRecord)> = indexed
        .iter()
        .take(elite_count)
        .enumerate()
        .map(|(rank, &(index, _))| {
            let record = BreedingRecord {
                origin: BreedingOrigin::Elite(rank),
                parent_source: [JointSource::ParentA; JOINT_COUNT],
                mutated_joints: [false; JOINT_COUNT],
            };
            (genomes[index].clone(), record)
        })
        .collect();

    while results.len() < population_size {
        let parent_a_index = tournament_select(&indexed, rng);
        let parent_b_index = tournament_select(&indexed, rng);

        let parent_a_fitness = indexed[parent_a_index].1;
        let parent_b_fitness = indexed[parent_b_index].1;

        let (mut child, parent_source) = Genome::crossover(
            &genomes[indexed[parent_a_index].0],
            &genomes[indexed[parent_b_index].0],
            rng,
        );
        let mutated_joints = child.mutate(rng);

        let record = BreedingRecord {
            origin: BreedingOrigin::Crossover {
                parent_a_fitness,
                parent_b_fitness,
            },
            parent_source,
            mutated_joints,
        };
        results.push((child, record));
    }

    results
}

fn tournament_select(indexed: &[(usize, f32)], rng: &mut impl Rng) -> usize {
    let tournament_size = 3;
    let mut best = rng.random_range(0..indexed.len());
    for _ in 1..tournament_size {
        let candidate = rng.random_range(0..indexed.len());
        if indexed[candidate].1 > indexed[best].1 {
            best = candidate;
        }
    }
    best
}
