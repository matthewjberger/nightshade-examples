use nightshade::prelude::*;

use crate::materials::{apply_material, spawn_city_mesh, spawn_point_light};

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

pub struct SmokeDescriptor {
    pub position: Vec3,
}

#[derive(Default)]
pub struct ChunkData {
    pub meshes: Vec<MeshDescriptor>,
    pub lights: Vec<LightDescriptor>,
    pub smoke_emitters: Vec<SmokeDescriptor>,
}

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

    pub fn light(&mut self, position: Vec3, color: Vec3, intensity: f32, range: f32) {
        self.lights.push(LightDescriptor {
            position,
            color,
            intensity,
            range,
        });
    }

    pub fn smoke(&mut self, position: Vec3) {
        self.smoke_emitters.push(SmokeDescriptor { position });
    }

    pub fn total_count(&self) -> usize {
        self.meshes.len() + self.lights.len() + self.smoke_emitters.len()
    }

    pub fn mesh_count_in_range(&self, start: usize, count: usize) -> usize {
        let mesh_end = self.meshes.len();
        let total = mesh_end + self.lights.len() + self.smoke_emitters.len();
        let range_end = (start + count).min(total);
        mesh_end.saturating_sub(start).min(range_end.saturating_sub(start))
    }

    pub fn instantiate_range(&self, world: &mut World, start: usize, count: usize) -> Vec<Entity> {
        let mesh_end = self.meshes.len();
        let light_end = mesh_end + self.lights.len();
        let total = light_end + self.smoke_emitters.len();

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
            } else if index < light_end {
                let desc = &self.lights[index - mesh_end];
                entities.push(spawn_point_light(
                    world,
                    desc.position,
                    desc.color,
                    desc.intensity,
                    desc.range,
                ));
            } else {
                let desc = &self.smoke_emitters[index - light_end];
                let entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
                let mut emitter = ParticleEmitter::smoke(desc.position);
                emitter.spawn_rate = 15.0;
                emitter.size_start = 0.3;
                emitter.size_end = 1.5;
                world.set_particle_emitter(entity, emitter);
                entities.push(entity);
            }
        }

        entities
    }
}
