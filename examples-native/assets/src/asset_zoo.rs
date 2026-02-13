use nightshade::ecs::animation::components::AnimationClip;
use std::path::{Path, PathBuf};

use crate::glb_export::{self, GlbExportModel};

pub struct PackGenerationResult {
    pub total_bytes: usize,
    pub model_count: usize,
}

pub fn generate_pack_glb(
    model_files: &[PathBuf],
    animation_files: &[PathBuf],
    output_path: &Path,
    scale_factor: f32,
) -> Result<PackGenerationResult, String> {
    let mut total_bytes = 0;
    let mut model_count = 0;

    for model_path in model_files {
        let model_name = model_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let result = nightshade::ecs::prefab::import_fbx_from_path(model_path)
            .map_err(|error| format!("Failed to import {}: {}", model_name, error))?;

        let prefab = result
            .prefabs
            .into_iter()
            .next()
            .ok_or_else(|| format!("No prefab found in {}", model_name))?;

        let mut all_clips: Vec<AnimationClip> = Vec::new();
        for anim_path in animation_files {
            if let Ok(clips) = nightshade::ecs::prefab::import_fbx_animations_from_path(anim_path) {
                let anim_name = anim_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                for mut clip in clips {
                    clip.name = anim_name.clone();
                    all_clips.push(clip);
                }
            }
        }

        for clip in &result.animations {
            if !all_clips.iter().any(|existing| existing.name == clip.name) {
                all_clips.push(clip.clone());
            }
        }

        let export_model = GlbExportModel {
            prefab,
            skins: result.skins,
            meshes: result.meshes,
            textures: result.textures,
        };

        let glb_bytes = glb_export::build_glb(&export_model, &all_clips, scale_factor)
            .map_err(|error| format!("Failed to build GLB for {}: {}", model_name, error))?;

        let output_file = output_path.join(format!("{}.glb", model_name));
        std::fs::write(&output_file, &glb_bytes)
            .map_err(|error| format!("Failed to write {}: {}", output_file.display(), error))?;

        total_bytes += glb_bytes.len();
        model_count += 1;
    }

    Ok(PackGenerationResult {
        total_bytes,
        model_count,
    })
}
