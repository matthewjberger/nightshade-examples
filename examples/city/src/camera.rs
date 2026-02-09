use nightshade::prelude::*;
use rand::{Rng, SeedableRng};

const BLOCK_SIZE: f32 = 16.0;
const CITY_HALF_EXTENT: f32 = 8.0 * 64.0;

const DRIVE_SPEED: f32 = 8.0;
const DRIVE_HEIGHT: f32 = 2.5;
const DRIVE_LOOK_DOWN: f32 = -0.03;
const DRIVE_LOOK_BLEND: f32 = 3.0;

const FLYOVER_SPEED: f32 = 18.0;
const FLYOVER_HEIGHT: f32 = 25.0;
const FLYOVER_LOOK_DOWN: f32 = -0.15;
const FLYOVER_LOOK_BLEND: f32 = 2.0;
const FLYOVER_TURN_CHANCE: f32 = 0.25;

#[derive(Clone, Copy, PartialEq)]
pub enum CinematicMode {
    Drive,
    Flyover,
    Orbit,
}

impl CinematicMode {
    pub const ALL: &[CinematicMode] = &[
        CinematicMode::Drive,
        CinematicMode::Flyover,
        CinematicMode::Orbit,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Drive => "Drive",
            Self::Flyover => "Flyover",
            Self::Orbit => "Orbit",
        }
    }
}

struct DriveState {
    road_x: f32,
    road_z: f32,
    direction_x: f32,
    direction_z: f32,
    segment_progress: f32,
    look_x: f32,
    look_z: f32,
    rng: rand::rngs::StdRng,
}

pub struct CameraController {
    mode: CinematicMode,
    time: f32,
    drive: DriveState,
}

impl CameraController {
    pub fn new(mode: CinematicMode, camera_position: Vec3) -> Self {
        let road_x = (camera_position.x / BLOCK_SIZE).round() * BLOCK_SIZE;
        let road_z = (camera_position.z / BLOCK_SIZE).round() * BLOCK_SIZE;
        Self {
            mode,
            time: 0.0,
            drive: DriveState {
                road_x,
                road_z,
                direction_x: 1.0,
                direction_z: 0.0,
                segment_progress: 0.0,
                look_x: 1.0,
                look_z: 0.0,
                rng: rand::rngs::StdRng::seed_from_u64(42),
            },
        }
    }

    pub fn mode(&self) -> CinematicMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: CinematicMode, camera_position: Vec3) {
        self.mode = mode;
        self.time = 0.0;
        if matches!(mode, CinematicMode::Drive | CinematicMode::Flyover) {
            self.drive.road_x = (camera_position.x / BLOCK_SIZE).round() * BLOCK_SIZE;
            self.drive.road_z = (camera_position.z / BLOCK_SIZE).round() * BLOCK_SIZE;
            self.drive.segment_progress = 0.0;
            self.drive.direction_x = 1.0;
            self.drive.direction_z = 0.0;
            self.drive.look_x = 1.0;
            self.drive.look_z = 0.0;
        }
    }

    pub fn update(&mut self, delta_time: f32) -> (Vec3, nalgebra_glm::Qua<f32>) {
        self.time += delta_time;
        match self.mode {
            CinematicMode::Drive => self.update_street(
                delta_time,
                DRIVE_SPEED,
                DRIVE_HEIGHT,
                DRIVE_LOOK_DOWN,
                DRIVE_LOOK_BLEND,
                0.3,
            ),
            CinematicMode::Flyover => self.update_street(
                delta_time,
                FLYOVER_SPEED,
                FLYOVER_HEIGHT,
                FLYOVER_LOOK_DOWN,
                FLYOVER_LOOK_BLEND,
                FLYOVER_TURN_CHANCE,
            ),
            CinematicMode::Orbit => self.update_orbit(),
        }
    }

    fn advance_road(&mut self, delta_time: f32, speed: f32, turn_chance: f32) {
        self.drive.segment_progress += (speed * delta_time) / BLOCK_SIZE;

        while self.drive.segment_progress >= 1.0 {
            self.drive.segment_progress -= 1.0;
            self.drive.road_x += self.drive.direction_x * BLOCK_SIZE;
            self.drive.road_z += self.drive.direction_z * BLOCK_SIZE;

            if self.drive.rng.random_range(0.0f32..1.0) < turn_chance {
                let (new_dx, new_dz) = if self.drive.rng.random_bool(0.5) {
                    (self.drive.direction_z, -self.drive.direction_x)
                } else {
                    (-self.drive.direction_z, self.drive.direction_x)
                };
                self.drive.direction_x = new_dx;
                self.drive.direction_z = new_dz;
            }

            let bound = CITY_HALF_EXTENT - BLOCK_SIZE * 3.0;
            if self.drive.road_x.abs() > bound || self.drive.road_z.abs() > bound {
                self.drive.direction_x = -self.drive.direction_x;
                self.drive.direction_z = -self.drive.direction_z;
                self.drive.road_x += self.drive.direction_x * BLOCK_SIZE;
                self.drive.road_z += self.drive.direction_z * BLOCK_SIZE;
            }
        }
    }

    fn update_street(
        &mut self,
        delta_time: f32,
        speed: f32,
        height: f32,
        look_down: f32,
        look_blend_rate: f32,
        turn_chance: f32,
    ) -> (Vec3, nalgebra_glm::Qua<f32>) {
        self.advance_road(delta_time, speed, turn_chance);

        let position = Vec3::new(
            self.drive.road_x + self.drive.direction_x * BLOCK_SIZE * self.drive.segment_progress,
            height,
            self.drive.road_z + self.drive.direction_z * BLOCK_SIZE * self.drive.segment_progress,
        );

        let blend = (delta_time * look_blend_rate).min(1.0);
        self.drive.look_x += (self.drive.direction_x - self.drive.look_x) * blend;
        self.drive.look_z += (self.drive.direction_z - self.drive.look_z) * blend;
        let len =
            (self.drive.look_x * self.drive.look_x + self.drive.look_z * self.drive.look_z).sqrt();
        if len > 0.001 {
            self.drive.look_x /= len;
            self.drive.look_z /= len;
        }

        let forward =
            nalgebra_glm::normalize(&Vec3::new(self.drive.look_x, look_down, self.drive.look_z));
        (position, look_rotation(&forward))
    }

    fn update_orbit(&self) -> (Vec3, nalgebra_glm::Qua<f32>) {
        let time = self.time;

        let radius = 350.0;
        let altitude = 80.0;
        let angular_speed = 0.08;
        let angle = time * angular_speed;

        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let y = altitude + 8.0 * (time * 0.15).sin();

        let position = Vec3::new(x, y, z);

        let target = Vec3::new(0.0, 15.0, 0.0);
        let forward = nalgebra_glm::normalize(&(target - position));

        (position, look_rotation(&forward))
    }
}

fn look_rotation(forward: &Vec3) -> nalgebra_glm::Qua<f32> {
    let forward = nalgebra_glm::normalize(forward);
    let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&forward, &Vec3::y()));
    let up = nalgebra_glm::cross(&right, &forward);
    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, up, -forward]))
}
