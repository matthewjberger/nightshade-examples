use std::collections::HashMap;

use nightshade::prelude::*;

use crate::building::BuildingType;
use crate::city::{CHUNK_SIZE, CityChunkLayout};
use crate::interiors;

const MINIMAP_SIZE: f32 = 220.0;
const MINIMAP_MARGIN: f32 = 14.0;
const VIEW_RADIUS: f32 = 128.0;
const BORDER_WIDTH: f32 = 3.5;
const MASK_SEGMENTS: usize = 64;

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
    let diameter = MINIMAP_SIZE;
    let radius = diameter / 2.0;
    let content_radius = radius - BORDER_WIDTH;

    egui::Area::new(egui::Id::new("minimap"))
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::vec2(-MINIMAP_MARGIN, -MINIMAP_MARGIN),
        )
        .interactable(false)
        .show(ui_context, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(diameter, diameter), egui::Sense::hover());

            let outer_rect = response.rect;
            let center = outer_rect.center();
            let bg_color = egui::Color32::from_rgba_unmultiplied(8, 12, 24, 230);

            painter.circle_filled(center, radius, bg_color);

            let scale = content_radius / VIEW_RADIUS;
            let content_radius_sq = content_radius * content_radius;

            let world_to_map = |wx: f32, wz: f32| -> egui::Pos2 {
                let dx = wx - state.camera_x;
                let dz = wz - state.camera_z;
                egui::pos2(center.x + dx * scale, center.y + dz * scale)
            };

            let center_in_circle = |pos: egui::Pos2| -> bool {
                let dx = pos.x - center.x;
                let dy = pos.y - center.y;
                dx * dx + dy * dy <= content_radius_sq
            };

            let chunk_radius = (VIEW_RADIUS / CHUNK_SIZE).ceil() as i32 + 1;
            let camera_chunk_x = (state.camera_x / CHUNK_SIZE).floor() as i32;
            let camera_chunk_z = (state.camera_z / CHUNK_SIZE).floor() as i32;

            let visible_min_x = (camera_chunk_x - chunk_radius).max(state.city_min);
            let visible_max_x = (camera_chunk_x + chunk_radius).min(state.city_max);
            let visible_min_z = (camera_chunk_z - chunk_radius).max(state.city_min);
            let visible_max_z = (camera_chunk_z + chunk_radius).min(state.city_max);

            let road_color = egui::Color32::from_rgb(35, 38, 48);
            let sidewalk_color = egui::Color32::from_rgb(48, 50, 56);
            for chunk_x in visible_min_x..=visible_max_x {
                for chunk_z in visible_min_z..=visible_max_z {
                    if let Some(layout) = layouts.get(&(chunk_x, chunk_z)) {
                        for segment in &layout.road_segments {
                            let seg_center = world_to_map(segment.x, segment.z);
                            if !center_in_circle(seg_center) {
                                continue;
                            }
                            let min_pos = world_to_map(
                                segment.x - segment.width / 2.0,
                                segment.z - segment.depth / 2.0,
                            );
                            let max_pos = world_to_map(
                                segment.x + segment.width / 2.0,
                                segment.z + segment.depth / 2.0,
                            );
                            let color = if segment.is_sidewalk {
                                sidewalk_color
                            } else {
                                road_color
                            };
                            let mut rect = egui::Rect::from_min_max(min_pos, max_pos);
                            if rect.width() < 0.8 {
                                rect = rect.expand2(egui::vec2(0.4, 0.0));
                            }
                            if rect.height() < 0.8 {
                                rect = rect.expand2(egui::vec2(0.0, 0.4));
                            }
                            painter.rect_filled(rect, 0.0, color);
                        }
                    }
                }
            }

            for chunk_x in visible_min_x..=visible_max_x {
                for chunk_z in visible_min_z..=visible_max_z {
                    if let Some(layout) = layouts.get(&(chunk_x, chunk_z)) {
                        for building in &layout.buildings {
                            let building_center = world_to_map(building.x, building.z);
                            if !center_in_circle(building_center) {
                                continue;
                            }
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
                            if rect.width() < 1.5 {
                                rect = rect.expand2(egui::vec2(0.75, 0.0));
                            }
                            if rect.height() < 1.5 {
                                rect = rect.expand2(egui::vec2(0.0, 0.75));
                            }
                            painter.rect_filled(rect, 0.5, color);

                            if interiors::building_is_enterable(building) {
                                painter.rect_stroke(
                                    rect.expand(1.0),
                                    0.5,
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 50)),
                                    egui::StrokeKind::Outside,
                                );
                            }
                        }
                    }
                }
            }

            draw_circle_mask(&painter, center, content_radius, outer_rect, bg_color);

            painter.circle_stroke(
                center,
                radius - BORDER_WIDTH / 2.0,
                egui::Stroke::new(
                    BORDER_WIDTH,
                    egui::Color32::from_rgba_unmultiplied(60, 80, 110, 200),
                ),
            );

            let fov_length = 22.0;
            let half_fov = 30.0_f32.to_radians();
            let forward_angle = state.camera_forward_z.atan2(state.camera_forward_x);
            let left_angle = forward_angle + half_fov;
            let right_angle = forward_angle - half_fov;

            let left_end = egui::pos2(
                center.x + left_angle.cos() * fov_length,
                center.y + left_angle.sin() * fov_length,
            );
            let right_end = egui::pos2(
                center.x + right_angle.cos() * fov_length,
                center.y + right_angle.sin() * fov_length,
            );

            let fov_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30);
            painter.line_segment([center, left_end], egui::Stroke::new(1.0, fov_color));
            painter.line_segment([center, right_end], egui::Stroke::new(1.0, fov_color));
            painter.line_segment([left_end, right_end], egui::Stroke::new(0.5, fov_color));

            painter.circle_filled(center, 3.5, egui::Color32::WHITE);
            painter.circle_stroke(
                center,
                3.5,
                egui::Stroke::new(0.8, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120)),
            );
        });
}

fn draw_circle_mask(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_radius: f32,
    bounding_rect: egui::Rect,
    color: egui::Color32,
) {
    let corners = [
        bounding_rect.left_top(),
        bounding_rect.right_top(),
        bounding_rect.right_bottom(),
        bounding_rect.left_bottom(),
    ];

    let corner_angles: [(f32, f32); 4] = [
        (std::f32::consts::PI, 1.5 * std::f32::consts::PI),
        (1.5 * std::f32::consts::PI, 2.0 * std::f32::consts::PI),
        (0.0, 0.5 * std::f32::consts::PI),
        (0.5 * std::f32::consts::PI, std::f32::consts::PI),
    ];

    let segs_per_corner = MASK_SEGMENTS / 4;

    for (corner_index, (start_angle, end_angle)) in corner_angles.iter().enumerate() {
        let corner = corners[corner_index];
        let next_corner = corners[(corner_index + 1) % 4];

        let mut mesh = egui::Mesh::default();

        let circle_start = egui::pos2(
            center.x + start_angle.cos() * inner_radius,
            center.y + start_angle.sin() * inner_radius,
        );
        mesh.vertices.push(egui::epaint::Vertex {
            pos: corner,
            uv: egui::epaint::WHITE_UV,
            color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: circle_start,
            uv: egui::epaint::WHITE_UV,
            color,
        });

        for segment_index in 1..=segs_per_corner {
            let t = segment_index as f32 / segs_per_corner as f32;
            let angle = start_angle + t * (end_angle - start_angle);
            let circle_point = egui::pos2(
                center.x + angle.cos() * inner_radius,
                center.y + angle.sin() * inner_radius,
            );
            let vertex_index = mesh.vertices.len() as u32;
            mesh.vertices.push(egui::epaint::Vertex {
                pos: circle_point,
                uv: egui::epaint::WHITE_UV,
                color,
            });
            mesh.indices.push(0);
            mesh.indices.push(vertex_index - 1);
            mesh.indices.push(vertex_index);
        }

        let edge_vertex_index = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: next_corner,
            uv: egui::epaint::WHITE_UV,
            color,
        });
        mesh.indices.push(0);
        mesh.indices.push(edge_vertex_index - 1);
        mesh.indices.push(edge_vertex_index);

        painter.add(egui::Shape::mesh(mesh));
    }
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
