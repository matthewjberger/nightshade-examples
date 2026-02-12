use std::collections::HashMap;

use nightshade::ecs::prefab::resources::{mesh_cache_insert, mesh_cache_lookup_id};
use nightshade::prelude::*;

use crate::materials::{apply_material, spawn_city_mesh, spawn_point_light};
use crate::stroke_font;
use crate::tube_mesh;

pub struct MeshDescriptor {
    pub mesh: &'static str,
    pub position: Vec3,
    pub scale: Vec3,
    pub material: &'static str,
    pub casts_shadow: bool,
    pub rotation: Option<nalgebra_glm::Quat>,
}

pub struct LightDescriptor {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
}

pub enum ParticleKind {
    Smoke,
    Fire,
    Embers,
    Sparks,
}

pub struct ParticleDescriptor {
    pub position: Vec3,
    pub kind: ParticleKind,
}

pub struct NeonSignDescriptor {
    pub text: &'static str,
    pub position: Vec3,
    pub material: &'static str,
    pub scale: f32,
    pub rotation: nalgebra_glm::Quat,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct InstanceGroupKey {
    mesh: &'static str,
    material: &'static str,
}

struct InstanceGroupData {
    key: InstanceGroupKey,
    instances: Vec<InstanceTransform>,
}

#[derive(Default)]
pub struct ChunkData {
    pub meshes: Vec<MeshDescriptor>,
    pub lights: Vec<LightDescriptor>,
    pub neon_signs: Vec<NeonSignDescriptor>,
    pub particles: Vec<ParticleDescriptor>,
    instance_group_map: HashMap<InstanceGroupKey, usize>,
    instance_groups: Vec<InstanceGroupData>,
}

const NEON_TUBE_RADIUS: f32 = 0.04;

impl ChunkData {
    pub fn mesh(
        &mut self,
        mesh: &'static str,
        position: Vec3,
        scale: Vec3,
        material: &'static str,
    ) {
        self.meshes.push(MeshDescriptor {
            mesh,
            position,
            scale,
            material,
            casts_shadow: false,
            rotation: None,
        });
    }

    pub fn mesh_shadow(
        &mut self,
        mesh: &'static str,
        position: Vec3,
        scale: Vec3,
        material: &'static str,
    ) {
        self.meshes.push(MeshDescriptor {
            mesh,
            position,
            scale,
            material,
            casts_shadow: true,
            rotation: None,
        });
    }

    pub fn mesh_rotated(
        &mut self,
        mesh: &'static str,
        position: Vec3,
        scale: Vec3,
        material: &'static str,
        rotation: nalgebra_glm::Quat,
    ) {
        self.meshes.push(MeshDescriptor {
            mesh,
            position,
            scale,
            material,
            casts_shadow: false,
            rotation: Some(rotation),
        });
    }

    pub fn instance(
        &mut self,
        mesh: &'static str,
        position: Vec3,
        scale: Vec3,
        material: &'static str,
        rotation: Option<nalgebra_glm::Quat>,
    ) {
        let key = InstanceGroupKey { mesh, material };
        let transform = match rotation {
            Some(rot) => InstanceTransform::new(position, rot, scale),
            None => InstanceTransform::from_translation_scale(position, scale),
        };
        if let Some(&group_index) = self.instance_group_map.get(&key) {
            self.instance_groups[group_index].instances.push(transform);
        } else {
            let group_index = self.instance_groups.len();
            self.instance_group_map.insert(key.clone(), group_index);
            self.instance_groups.push(InstanceGroupData {
                key,
                instances: vec![transform],
            });
        }
    }

    pub fn light(&mut self, position: Vec3, color: Vec3, intensity: f32, range: f32) {
        self.lights.push(LightDescriptor {
            position,
            color,
            intensity,
            range,
        });
    }

    pub fn smoke(&mut self, position: Vec3) {
        self.particles.push(ParticleDescriptor {
            position,
            kind: ParticleKind::Smoke,
        });
    }

    pub fn campfire(&mut self, position: Vec3) {
        self.particles.push(ParticleDescriptor {
            position,
            kind: ParticleKind::Fire,
        });
        self.particles.push(ParticleDescriptor {
            position: Vec3::new(position.x, position.y + 0.5, position.z),
            kind: ParticleKind::Smoke,
        });
        self.particles.push(ParticleDescriptor {
            position,
            kind: ParticleKind::Embers,
        });
        self.light(
            Vec3::new(position.x, position.y + 0.3, position.z),
            Vec3::new(1.0, 0.65, 0.25),
            6.0,
            15.0,
        );
    }

    pub fn neon_sign(
        &mut self,
        text: &'static str,
        position: Vec3,
        material: &'static str,
        scale: f32,
        rotation: nalgebra_glm::Quat,
    ) {
        self.neon_signs.push(NeonSignDescriptor {
            text,
            position,
            material,
            scale,
            rotation,
        });
    }

    pub fn sparks(&mut self, position: Vec3) {
        self.particles.push(ParticleDescriptor {
            position,
            kind: ParticleKind::Sparks,
        });
    }

    pub fn total_count(&self) -> usize {
        self.meshes.len()
            + self.neon_signs.len()
            + self.instance_groups.len()
            + self.lights.len()
            + self.particles.len()
    }

    pub fn mesh_and_instance_counts_in_range(&self, start: usize, count: usize) -> (usize, usize) {
        let mesh_end = self.meshes.len();
        let neon_end = mesh_end + self.neon_signs.len();
        let instance_end = neon_end + self.instance_groups.len();
        let total = instance_end + self.lights.len() + self.particles.len();
        let range_end = (start + count).min(total);

        let regular_mesh_count = if start < neon_end {
            neon_end.min(range_end).saturating_sub(start)
        } else {
            0
        };

        let instance_group_count = if range_end > neon_end && start < instance_end {
            instance_end.min(range_end) - neon_end.max(start)
        } else {
            0
        };

        (regular_mesh_count, instance_group_count)
    }

    pub fn instantiate_range(&self, world: &mut World, start: usize, count: usize) -> Vec<Entity> {
        let mesh_end = self.meshes.len();
        let neon_end = mesh_end + self.neon_signs.len();
        let instance_end = neon_end + self.instance_groups.len();
        let light_end = instance_end + self.lights.len();
        let total = light_end + self.particles.len();

        let range_end = (start + count).min(total);
        let mut entities = Vec::with_capacity(range_end.saturating_sub(start));

        for index in start..range_end {
            if index < mesh_end {
                let desc = &self.meshes[index];
                let entity = spawn_city_mesh(world, desc.mesh, desc.position, desc.scale);
                apply_material(world, entity, desc.material);
                if desc.casts_shadow {
                    world.set_casts_shadow(entity, CastsShadow);
                }
                if let Some(rotation) = desc.rotation {
                    if let Some(transform) = world.get_local_transform_mut(entity) {
                        transform.rotation = rotation;
                    }
                    mark_local_transform_dirty(world, entity);
                }
                entities.push(entity);
            } else if index < neon_end {
                let desc = &self.neon_signs[index - mesh_end];
                let entity = instantiate_neon_sign(world, desc);
                entities.push(entity);
            } else if index < instance_end {
                let group = &self.instance_groups[index - neon_end];
                let entity = spawn_instanced_mesh_with_material(
                    world,
                    group.key.mesh,
                    group.instances.clone(),
                    group.key.material,
                );
                world.remove_casts_shadow(entity);
                entities.push(entity);
            } else if index < light_end {
                let desc = &self.lights[index - instance_end];
                entities.push(spawn_point_light(
                    world,
                    desc.position,
                    desc.color,
                    desc.intensity,
                    desc.range,
                ));
            } else {
                let desc = &self.particles[index - light_end];
                let entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
                let emitter = match desc.kind {
                    ParticleKind::Smoke => {
                        let mut emitter = ParticleEmitter::smoke(desc.position);
                        emitter.spawn_rate = 15.0;
                        emitter.size_start = 0.3;
                        emitter.size_end = 1.5;
                        emitter
                    }
                    ParticleKind::Fire => {
                        let mut emitter = ParticleEmitter::fire(desc.position);
                        emitter.spawn_rate = 30.0;
                        emitter.size_start = 0.2;
                        emitter.size_end = 0.6;
                        emitter
                    }
                    ParticleKind::Embers => {
                        let mut emitter = ParticleEmitter::sparks(desc.position);
                        emitter.spawn_rate = 5.0;
                        emitter.initial_velocity_min = 1.0;
                        emitter.initial_velocity_max = 3.0;
                        emitter.particle_lifetime_min = 0.5;
                        emitter.particle_lifetime_max = 1.5;
                        emitter.size_start = 0.05;
                        emitter.size_end = 0.02;
                        emitter.emissive_strength = 3.0;
                        emitter
                    }
                    ParticleKind::Sparks => {
                        let mut emitter = ParticleEmitter::sparks(desc.position);
                        emitter.spawn_rate = 1.5;
                        emitter.initial_velocity_min = 0.5;
                        emitter.initial_velocity_max = 2.0;
                        emitter.particle_lifetime_min = 0.3;
                        emitter.particle_lifetime_max = 0.8;
                        emitter.size_start = 0.03;
                        emitter.size_end = 0.01;
                        emitter.emissive_strength = 5.0;
                        emitter
                    }
                };
                world.set_particle_emitter(entity, emitter);
                entities.push(entity);
            }
        }

        entities
    }
}

fn build_text_3d_polylines(text: &str) -> Vec<Vec<Vec3>> {
    let text_width = stroke_font::measure_text(text);
    let x_offset = -text_width / 2.0;
    let y_offset = -stroke_font::CHAR_HEIGHT / 2.0;

    let mut polylines_3d = Vec::new();
    let mut cursor_x = 0.0;

    for character in text.chars() {
        if character == ' ' {
            cursor_x += stroke_font::WORD_SPACING;
            continue;
        }

        let strokes = stroke_font::get_character_strokes(character);
        for stroke in strokes {
            let polyline_3d: Vec<Vec3> = stroke
                .iter()
                .map(|point| Vec3::new(x_offset + cursor_x + point.x, y_offset + point.y, 0.0))
                .collect();
            polylines_3d.push(polyline_3d);
        }

        cursor_x += stroke_font::CHAR_WIDTH + stroke_font::CHAR_SPACING;
    }

    polylines_3d
}

fn ensure_neon_mesh_cached(world: &mut World, text: &str) -> String {
    let mesh_name = format!("neon_{text}");

    if mesh_cache_lookup_id(&world.resources.mesh_cache, &mesh_name).is_some() {
        return mesh_name;
    }

    let polylines = build_text_3d_polylines(text);
    let mesh = tube_mesh::build_neon_tube_mesh(&polylines, NEON_TUBE_RADIUS);
    mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

    mesh_name
}

fn instantiate_neon_sign(world: &mut World, desc: &NeonSignDescriptor) -> Entity {
    let mesh_name = ensure_neon_mesh_cached(world, desc.text);

    let scale_vec = Vec3::new(desc.scale, desc.scale, desc.scale);
    let entity = spawn_city_mesh(world, &mesh_name, desc.position, scale_vec);
    apply_material(world, entity, desc.material);

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.rotation = desc.rotation;
    }
    mark_local_transform_dirty(world, entity);

    entity
}
