use std::collections::HashMap;

use nightshade::prelude::*;

use crate::building::BuildingType;
use crate::city::{CHUNK_SIZE, CityChunkLayout};

const MINIMAP_SIZE: f32 = 200.0;
const MINIMAP_PADDING: f32 = 4.0;
const MINIMAP_MARGIN: f32 = 10.0;

pub struct MinimapState {
    pub camera_x: f32,
    pub camera_z: f32,
    pub camera_forward_x: f32,
    pub camera_forward_z: f32,
    pub city_min: i32,
    pub city_max: i32,
}

pub fn draw(
    ui_context: &egui::Context,
    layouts: &HashMap<(i32, i32), CityChunkLayout>,
    state: &MinimapState,
) {
    let outer_size = MINIMAP_SIZE + MINIMAP_PADDING * 2.0;

    egui::Area::new(egui::Id::new("minimap"))
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::vec2(-MINIMAP_MARGIN, -MINIMAP_MARGIN),
        )
        .interactable(false)
        .show(ui_context, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(outer_size, outer_size), egui::Sense::hover());

            let outer_rect = response.rect;
            let map_rect = outer_rect.shrink(MINIMAP_PADDING);

            painter.rect_filled(
                outer_rect,
                6.0,
                egui::Color32::from_rgba_unmultiplied(8, 10, 18, 220),
            );
            painter.rect_stroke(
                outer_rect,
                6.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(60, 70, 90, 180)),
                egui::StrokeKind::Inside,
            );

            painter.rect_filled(map_rect, 2.0, egui::Color32::from_rgb(18, 30, 48));

            let world_min = state.city_min as f32 * CHUNK_SIZE;
            let world_span = (state.city_max - state.city_min + 1) as f32 * CHUNK_SIZE;

            let world_to_map = |wx: f32, wz: f32| -> egui::Pos2 {
                let nx = (wx - world_min) / world_span;
                let nz = (wz - world_min) / world_span;
                egui::pos2(
                    map_rect.min.x + nx * map_rect.width(),
                    map_rect.min.y + nz * map_rect.height(),
                )
            };

            let ground_min = world_to_map(world_min, world_min);
            let ground_max = world_to_map(world_min + world_span, world_min + world_span);
            painter.rect_filled(
                egui::Rect::from_min_max(ground_min, ground_max),
                0.0,
                egui::Color32::from_rgb(28, 28, 34),
            );

            let grid_color = egui::Color32::from_rgba_unmultiplied(45, 45, 55, 60);
            for grid_x in state.city_min..=state.city_max + 1 {
                let x_world = grid_x as f32 * CHUNK_SIZE;
                let top = world_to_map(x_world, world_min);
                let bottom = world_to_map(x_world, world_min + world_span);
                painter.line_segment([top, bottom], egui::Stroke::new(0.5, grid_color));
            }
            for grid_z in state.city_min..=state.city_max + 1 {
                let z_world = grid_z as f32 * CHUNK_SIZE;
                let left = world_to_map(world_min, z_world);
                let right = world_to_map(world_min + world_span, z_world);
                painter.line_segment([left, right], egui::Stroke::new(0.5, grid_color));
            }

            for layout in layouts.values() {
                for building in &layout.buildings {
                    let color = building_color(building.building_type, building.height);
                    let min_pos = world_to_map(
                        building.x - building.width / 2.0,
                        building.z - building.depth / 2.0,
                    );
                    let max_pos = world_to_map(
                        building.x + building.width / 2.0,
                        building.z + building.depth / 2.0,
                    );
                    let mut rect = egui::Rect::from_min_max(min_pos, max_pos);
                    if rect.width() < 1.0 {
                        rect = rect.expand2(egui::vec2(0.5, 0.0));
                    }
                    if rect.height() < 1.0 {
                        rect = rect.expand2(egui::vec2(0.0, 0.5));
                    }
                    painter.rect_filled(rect, 0.0, color);
                }
            }

            let cam_raw = world_to_map(state.camera_x, state.camera_z);
            let cam_pos = egui::pos2(
                cam_raw.x.clamp(map_rect.min.x + 2.0, map_rect.max.x - 2.0),
                cam_raw.y.clamp(map_rect.min.y + 2.0, map_rect.max.y - 2.0),
            );

            let fov_length = 14.0;
            let half_fov = 30.0_f32.to_radians();
            let forward_angle = state.camera_forward_z.atan2(state.camera_forward_x);
            let left_angle = forward_angle + half_fov;
            let right_angle = forward_angle - half_fov;

            let left_end = egui::pos2(
                cam_pos.x + left_angle.cos() * fov_length,
                cam_pos.y + left_angle.sin() * fov_length,
            );
            let right_end = egui::pos2(
                cam_pos.x + right_angle.cos() * fov_length,
                cam_pos.y + right_angle.sin() * fov_length,
            );

            let fov_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
            painter.line_segment([cam_pos, left_end], egui::Stroke::new(1.0, fov_color));
            painter.line_segment([cam_pos, right_end], egui::Stroke::new(1.0, fov_color));
            painter.line_segment([left_end, right_end], egui::Stroke::new(0.5, fov_color));

            painter.circle_filled(cam_pos, 3.5, egui::Color32::WHITE);
        });
}

fn building_color(building_type: BuildingType, height: f32) -> egui::Color32 {
    let brightness = (height / 50.0).clamp(0.4, 1.0);
    let scale = |base: u8| -> u8 { (base as f32 * brightness) as u8 };

    match building_type {
        BuildingType::Skyscraper => egui::Color32::from_rgb(scale(95), scale(140), scale(205)),
        BuildingType::OfficeTower => egui::Color32::from_rgb(scale(155), scale(155), scale(170)),
        BuildingType::LowRiseOffice => egui::Color32::from_rgb(scale(170), scale(160), scale(140)),
        BuildingType::ApartmentBlock => egui::Color32::from_rgb(scale(175), scale(110), scale(90)),
        BuildingType::House => egui::Color32::from_rgb(scale(195), scale(175), scale(135)),
        BuildingType::Warehouse => egui::Color32::from_rgb(scale(115), scale(115), scale(125)),
        BuildingType::Park => egui::Color32::from_rgb(45, 135, 45),
    }
}
