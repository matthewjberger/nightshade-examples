use nightshade::prelude::*;

use crate::ecs::World as GameWorld;
use crate::events::DayChangedEvent;
use crate::types::{
    DAY_LENGTH_SECONDS, DAYS_PER_SEASON, Season, Weather, get_sun_color, get_sun_intensity,
};

pub fn advance(game: &mut GameWorld, world: &World) -> Option<DayChangedEvent> {
    let delta = world.resources.window.timing.delta_time;
    let time_advance = delta * (24.0 / DAY_LENGTH_SECONDS);
    game.resources.hour += time_advance;

    if game.resources.hour >= 24.0 {
        game.resources.hour = 6.0;
        game.resources.day += 1;

        let new_season = if game.resources.day.is_multiple_of(DAYS_PER_SEASON) {
            game.resources.season = game.resources.season.next();
            Some(game.resources.season)
        } else {
            None
        };

        game.resources.weather = roll_weather(game.resources.season);

        return Some(DayChangedEvent {
            new_day: game.resources.day,
            new_season,
        });
    }

    None
}

fn roll_weather(season: Season) -> Weather {
    let mut rng = rand::rng();
    let roll: f32 = rng.random();

    match season {
        Season::Spring => {
            if roll < 0.5 {
                Weather::Sunny
            } else if roll < 0.75 {
                Weather::Cloudy
            } else {
                Weather::Rainy
            }
        }
        Season::Summer => {
            if roll < 0.7 {
                Weather::Sunny
            } else if roll < 0.9 {
                Weather::Cloudy
            } else {
                Weather::Stormy
            }
        }
        Season::Fall => {
            if roll < 0.4 {
                Weather::Sunny
            } else if roll < 0.7 {
                Weather::Cloudy
            } else {
                Weather::Rainy
            }
        }
        Season::Winter => {
            if roll < 0.3 {
                Weather::Sunny
            } else if roll < 0.5 {
                Weather::Cloudy
            } else {
                Weather::Snowy
            }
        }
    }
}

pub fn update_sun(game: &GameWorld, world: &mut World) {
    let Some(sun) = game.resources.visuals.sun else {
        return;
    };

    let hour = game.resources.hour;
    let hour_angle = (hour - 6.0) / 24.0 * std::f32::consts::TAU;
    let orbit_radius = 50.0;
    let sun_x = hour_angle.cos() * orbit_radius;
    let sun_y = hour_angle.sin() * orbit_radius;
    let sun_position = Vec3::new(sun_x, sun_y.abs().max(5.0), 0.0);

    if let Some(transform) = world.get_local_transform_mut(sun) {
        transform.translation = sun_position;
        let direction = nalgebra_glm::normalize(&(-sun_position));
        let up = Vec3::y();
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
        let corrected_up = nalgebra_glm::cross(&right, &direction);
        transform.rotation = nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
            right,
            corrected_up,
            -direction,
        ]));
    }
    mark_local_transform_dirty(world, sun);

    if let Some(light) = world.get_light_mut(sun) {
        light.color = get_sun_color(hour);
        light.intensity = get_sun_intensity(hour);
    }
}
