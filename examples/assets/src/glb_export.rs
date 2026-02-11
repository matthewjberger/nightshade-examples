use gltf_json as json;
use json::validation::USize64;
use nightshade::ecs::animation::components::{
    AnimationClip, AnimationProperty, AnimationSamplerOutput,
};
use nightshade::ecs::prefab::{GltfSkin, Prefab, PrefabNode};
use std::collections::HashMap;

pub struct GlbExportModel {
    pub prefab: Prefab,
    pub skins: Vec<GltfSkin>,
    pub meshes: HashMap<String, nightshade::ecs::mesh::Mesh>,
    pub textures: HashMap<String, (Vec<u8>, u32, u32)>,
}

pub fn build_glb(
    model: &GlbExportModel,
    animations: &[AnimationClip],
    scale_factor: f32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer_data: Vec<u8> = Vec::new();

    let mut accessors: Vec<json::Accessor> = Vec::new();
    let mut buffer_views: Vec<json::buffer::View> = Vec::new();
    let mut meshes: Vec<json::Mesh> = Vec::new();
    let mut gltf_nodes: Vec<json::Node> = Vec::new();
    let mut skins: Vec<json::Skin> = Vec::new();
    let mut images: Vec<json::Image> = Vec::new();
    let mut gltf_textures: Vec<json::Texture> = Vec::new();
    let mut materials: Vec<json::Material> = Vec::new();
    let mut samplers: Vec<json::texture::Sampler> = Vec::new();
    let mut texture_name_to_index: HashMap<String, u32> = HashMap::new();

    let mut node_index_map: HashMap<usize, usize> = HashMap::new();

    struct PrefabNodeInfo {
        gltf_index: usize,
        child_gltf_indices: Vec<usize>,
        name: Option<String>,
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    }

    fn assign_indices(
        prefab_node: &PrefabNode,
        node_index_map: &mut HashMap<usize, usize>,
        next_index: &mut usize,
        node_infos: &mut Vec<PrefabNodeInfo>,
        scale_factor: f32,
    ) -> usize {
        let current_index = *next_index;
        *next_index += 1;

        if let Some(prefab_idx) = prefab_node.node_index {
            node_index_map.insert(prefab_idx, current_index);
        }

        let mut child_gltf_indices = Vec::new();
        for child in &prefab_node.children {
            let child_index =
                assign_indices(child, node_index_map, next_index, node_infos, scale_factor);
            child_gltf_indices.push(child_index);
        }

        node_infos.push(PrefabNodeInfo {
            gltf_index: current_index,
            child_gltf_indices,
            name: prefab_node.components.name.as_ref().map(|n| n.0.clone()),
            translation: [
                prefab_node.local_transform.translation.x * scale_factor,
                prefab_node.local_transform.translation.y * scale_factor,
                prefab_node.local_transform.translation.z * scale_factor,
            ],
            rotation: [
                prefab_node.local_transform.rotation.i,
                prefab_node.local_transform.rotation.j,
                prefab_node.local_transform.rotation.k,
                prefab_node.local_transform.rotation.w,
            ],
            scale: [
                prefab_node.local_transform.scale.x,
                prefab_node.local_transform.scale.y,
                prefab_node.local_transform.scale.z,
            ],
        });

        current_index
    }

    let mut node_infos: Vec<PrefabNodeInfo> = Vec::new();
    let mut next_index = 0usize;

    for root_node in &model.prefab.root_nodes {
        assign_indices(
            root_node,
            &mut node_index_map,
            &mut next_index,
            &mut node_infos,
            scale_factor,
        );
    }

    node_infos.sort_by_key(|info| info.gltf_index);

    for info in &node_infos {
        gltf_nodes.push(json::Node {
            name: info.name.clone(),
            translation: Some(info.translation),
            rotation: Some(json::scene::UnitQuaternion(info.rotation)),
            scale: Some(info.scale),
            mesh: None,
            skin: None,
            children: if info.child_gltf_indices.is_empty() {
                None
            } else {
                Some(
                    info.child_gltf_indices
                        .iter()
                        .map(|index| json::Index::new(*index as u32))
                        .collect(),
                )
            },
            ..Default::default()
        });
    }

    if !model.textures.is_empty() {
        samplers.push(json::texture::Sampler {
            mag_filter: Some(json::validation::Checked::Valid(
                json::texture::MagFilter::Linear,
            )),
            min_filter: Some(json::validation::Checked::Valid(
                json::texture::MinFilter::LinearMipmapLinear,
            )),
            wrap_s: json::validation::Checked::Valid(json::texture::WrappingMode::Repeat),
            wrap_t: json::validation::Checked::Valid(json::texture::WrappingMode::Repeat),
            name: None,
            extensions: None,
            extras: Default::default(),
        });
    }

    for (texture_name, (rgba_data, width, height)) in &model.textures {
        let png_data = {
            let mut png_buffer = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut png_buffer);
            let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
            image::ImageEncoder::write_image(
                encoder,
                rgba_data,
                *width,
                *height,
                image::ExtendedColorType::Rgba8,
            )?;
            png_buffer
        };

        while !buffer_data.len().is_multiple_of(4) {
            buffer_data.push(0);
        }

        let image_start = buffer_data.len();
        buffer_data.extend_from_slice(&png_data);
        let image_length = buffer_data.len() - image_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(image_start)),
            byte_length: USize64::from(image_length),
            byte_stride: None,
            target: None,
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let image_index = images.len() as u32;
        images.push(json::Image {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            mime_type: Some(json::image::MimeType("image/png".to_string())),
            uri: None,
            name: Some(texture_name.clone()),
            extensions: None,
            extras: Default::default(),
        });

        let texture_index = gltf_textures.len() as u32;
        gltf_textures.push(json::Texture {
            sampler: Some(json::Index::new(0)),
            source: json::Index::new(image_index),
            name: Some(texture_name.clone()),
            extensions: None,
            extras: Default::default(),
        });

        texture_name_to_index.insert(texture_name.clone(), texture_index);
    }

    if !model.textures.is_empty() {
        let base_color_texture = model
            .textures
            .keys()
            .next()
            .and_then(|name| texture_name_to_index.get(name).copied());

        materials.push(json::Material {
            name: Some("DefaultMaterial".to_string()),
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                base_color_texture: base_color_texture.map(|idx| json::texture::Info {
                    index: json::Index::new(idx),
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                }),
                metallic_factor: json::material::StrengthFactor(0.0),
                roughness_factor: json::material::StrengthFactor(0.5),
                metallic_roughness_texture: None,
                extensions: None,
                extras: Default::default(),
            },
            alpha_mode: json::validation::Checked::Valid(json::material::AlphaMode::Opaque),
            alpha_cutoff: None,
            double_sided: false,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: json::material::EmissiveFactor([0.0, 0.0, 0.0]),
            extensions: None,
            extras: Default::default(),
        });
    } else {
        materials.push(json::Material {
            name: Some("DefaultMaterial".to_string()),
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor([0.8, 0.8, 0.8, 1.0]),
                base_color_texture: None,
                metallic_factor: json::material::StrengthFactor(0.0),
                roughness_factor: json::material::StrengthFactor(0.5),
                metallic_roughness_texture: None,
                extensions: None,
                extras: Default::default(),
            },
            alpha_mode: json::validation::Checked::Valid(json::material::AlphaMode::Opaque),
            alpha_cutoff: None,
            double_sided: false,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: json::material::EmissiveFactor([0.0, 0.0, 0.0]),
            extensions: None,
            extras: Default::default(),
        });
    }
    let default_material_index = Some(0u32);

    for skin in &model.skins {
        let ibm_start = buffer_data.len();
        for ibm in &skin.inverse_bind_matrices {
            for col in 0..4 {
                for row in 0..4 {
                    let value = ibm[(row, col)];
                    let scaled_value = if col == 3 && row < 3 {
                        value * scale_factor
                    } else {
                        value
                    };
                    buffer_data.extend_from_slice(&scaled_value.to_le_bytes());
                }
            }
        }
        let ibm_length = buffer_data.len() - ibm_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(ibm_start)),
            byte_length: USize64::from(ibm_length),
            byte_stride: None,
            target: None,
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let ibm_accessor_index = accessors.len() as u32;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(skin.inverse_bind_matrices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Mat4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        });

        let first_valid_node = node_index_map.values().next().copied().unwrap_or(0);

        let joint_indices: Vec<json::Index<json::Node>> = skin
            .joints
            .iter()
            .map(|&joint_idx| {
                if let Some(&gltf_idx) = node_index_map.get(&joint_idx) {
                    json::Index::new(gltf_idx as u32)
                } else {
                    json::Index::new(first_valid_node as u32)
                }
            })
            .collect();

        skins.push(json::Skin {
            inverse_bind_matrices: Some(json::Index::new(ibm_accessor_index)),
            joints: joint_indices,
            skeleton: None,
            name: skin.name.clone(),
            extensions: None,
            extras: Default::default(),
        });
    }

    for (mesh_name, mesh) in &model.meshes {
        let has_skin = mesh.skin_data.is_some();

        let (vertices_to_use, skinned_data) = if let Some(ref skin_data) = mesh.skin_data {
            (None, Some(skin_data))
        } else {
            (Some(&mesh.vertices), None)
        };

        let vertex_count =
            skinned_data.map_or_else(|| mesh.vertices.len(), |sd| sd.skinned_vertices.len());

        let positions_start = buffer_data.len();
        let mut min_pos = [f32::MAX; 3];
        let mut max_pos = [f32::MIN; 3];

        if let Some(sd) = skinned_data {
            for vertex in &sd.skinned_vertices {
                let scaled_pos = [
                    vertex.position[0] * scale_factor,
                    vertex.position[1] * scale_factor,
                    vertex.position[2] * scale_factor,
                ];
                buffer_data.extend_from_slice(&scaled_pos[0].to_le_bytes());
                buffer_data.extend_from_slice(&scaled_pos[1].to_le_bytes());
                buffer_data.extend_from_slice(&scaled_pos[2].to_le_bytes());
                for idx in 0..3 {
                    min_pos[idx] = min_pos[idx].min(scaled_pos[idx]);
                    max_pos[idx] = max_pos[idx].max(scaled_pos[idx]);
                }
            }
        } else if let Some(verts) = vertices_to_use {
            for vertex in verts {
                let scaled_pos = [
                    vertex.position[0] * scale_factor,
                    vertex.position[1] * scale_factor,
                    vertex.position[2] * scale_factor,
                ];
                buffer_data.extend_from_slice(&scaled_pos[0].to_le_bytes());
                buffer_data.extend_from_slice(&scaled_pos[1].to_le_bytes());
                buffer_data.extend_from_slice(&scaled_pos[2].to_le_bytes());
                for idx in 0..3 {
                    min_pos[idx] = min_pos[idx].min(scaled_pos[idx]);
                    max_pos[idx] = max_pos[idx].max(scaled_pos[idx]);
                }
            }
        }
        let positions_length = buffer_data.len() - positions_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(positions_start)),
            byte_length: USize64::from(positions_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let position_accessor_index = accessors.len() as u32;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(vertex_count),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::Array(
                min_pos.iter().map(|v| json::Value::from(*v)).collect(),
            )),
            max: Some(json::Value::Array(
                max_pos.iter().map(|v| json::Value::from(*v)).collect(),
            )),
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        });

        let normals_start = buffer_data.len();
        if let Some(sd) = skinned_data {
            for vertex in &sd.skinned_vertices {
                buffer_data.extend_from_slice(&vertex.normal[0].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.normal[1].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.normal[2].to_le_bytes());
            }
        } else if let Some(verts) = vertices_to_use {
            for vertex in verts {
                buffer_data.extend_from_slice(&vertex.normal[0].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.normal[1].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.normal[2].to_le_bytes());
            }
        }
        let normals_length = buffer_data.len() - normals_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(normals_start)),
            byte_length: USize64::from(normals_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let normal_accessor_index = accessors.len() as u32;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(vertex_count),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        });

        let texcoords_start = buffer_data.len();
        if let Some(sd) = skinned_data {
            for vertex in &sd.skinned_vertices {
                buffer_data.extend_from_slice(&vertex.tex_coords[0].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.tex_coords[1].to_le_bytes());
            }
        } else if let Some(verts) = vertices_to_use {
            for vertex in verts {
                buffer_data.extend_from_slice(&vertex.tex_coords[0].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.tex_coords[1].to_le_bytes());
            }
        }
        let texcoords_length = buffer_data.len() - texcoords_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(texcoords_start)),
            byte_length: USize64::from(texcoords_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let texcoord_accessor_index = accessors.len() as u32;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(vertex_count),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        });

        let mut joints_accessor_index = None;
        let mut weights_accessor_index = None;

        if let Some(sd) = skinned_data {
            let joints_start = buffer_data.len();
            for vertex in &sd.skinned_vertices {
                buffer_data.extend_from_slice(&(vertex.joint_indices[0] as u16).to_le_bytes());
                buffer_data.extend_from_slice(&(vertex.joint_indices[1] as u16).to_le_bytes());
                buffer_data.extend_from_slice(&(vertex.joint_indices[2] as u16).to_le_bytes());
                buffer_data.extend_from_slice(&(vertex.joint_indices[3] as u16).to_le_bytes());
            }
            let joints_length = buffer_data.len() - joints_start;

            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(joints_start)),
                byte_length: USize64::from(joints_length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
                name: None,
                extensions: None,
                extras: Default::default(),
            });

            joints_accessor_index = Some(accessors.len() as u32);
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(sd.skinned_vertices.len()),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::U16),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            });

            let weights_start = buffer_data.len();
            for vertex in &sd.skinned_vertices {
                buffer_data.extend_from_slice(&vertex.joint_weights[0].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.joint_weights[1].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.joint_weights[2].to_le_bytes());
                buffer_data.extend_from_slice(&vertex.joint_weights[3].to_le_bytes());
            }
            let weights_length = buffer_data.len() - weights_start;

            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(weights_start)),
                byte_length: USize64::from(weights_length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
                name: None,
                extensions: None,
                extras: Default::default(),
            });

            weights_accessor_index = Some(accessors.len() as u32);
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(sd.skinned_vertices.len()),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            });
        }

        let indices_start = buffer_data.len();
        for index in &mesh.indices {
            buffer_data.extend_from_slice(&(*index).to_le_bytes());
        }
        let indices_length = buffer_data.len() - indices_start;

        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(indices_start)),
            byte_length: USize64::from(indices_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ElementArrayBuffer,
            )),
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let indices_accessor_index = accessors.len() as u32;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(mesh.indices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
            extensions: None,
            extras: Default::default(),
        });

        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            json::validation::Checked::Valid(json::mesh::Semantic::Positions),
            json::Index::new(position_accessor_index),
        );
        attributes.insert(
            json::validation::Checked::Valid(json::mesh::Semantic::Normals),
            json::Index::new(normal_accessor_index),
        );
        attributes.insert(
            json::validation::Checked::Valid(json::mesh::Semantic::TexCoords(0)),
            json::Index::new(texcoord_accessor_index),
        );

        if let Some(joints_idx) = joints_accessor_index {
            attributes.insert(
                json::validation::Checked::Valid(json::mesh::Semantic::Joints(0)),
                json::Index::new(joints_idx),
            );
        }
        if let Some(weights_idx) = weights_accessor_index {
            attributes.insert(
                json::validation::Checked::Valid(json::mesh::Semantic::Weights(0)),
                json::Index::new(weights_idx),
            );
        }

        meshes.push(json::Mesh {
            primitives: vec![json::mesh::Primitive {
                attributes,
                indices: Some(json::Index::new(indices_accessor_index)),
                material: default_material_index.map(json::Index::new),
                mode: json::validation::Checked::Valid(json::mesh::Mode::Triangles),
                targets: None,
                extensions: None,
                extras: Default::default(),
            }],
            name: Some(mesh_name.clone()),
            weights: None,
            extensions: None,
            extras: Default::default(),
        });

        let skin_index = if has_skin {
            skinned_data.and_then(|sd| sd.skin_index).or(Some(0))
        } else {
            None
        };

        gltf_nodes.push(json::Node {
            mesh: Some(json::Index::new(meshes.len() as u32 - 1)),
            skin: skin_index.map(|idx| json::Index::new(idx as u32)),
            name: Some(mesh_name.clone()),
            ..Default::default()
        });
    }

    let mut gltf_animations: Vec<json::Animation> = Vec::new();

    let mut node_name_to_gltf_index: HashMap<String, usize> = HashMap::new();
    for (gltf_idx, node) in gltf_nodes.iter().enumerate() {
        if let Some(ref name) = node.name {
            node_name_to_gltf_index.insert(name.clone(), gltf_idx);
        }
    }

    for anim in animations {
        let mut anim_samplers: Vec<json::animation::Sampler> = Vec::new();
        let mut channels: Vec<json::animation::Channel> = Vec::new();

        for channel in &anim.channels {
            let target_node_idx = if let Some(ref target_name) = channel.target_bone_name {
                node_name_to_gltf_index.get(target_name).copied()
            } else {
                node_index_map.get(&channel.target_node).copied()
            };

            let Some(target_node_idx) = target_node_idx else {
                continue;
            };

            let times_start = buffer_data.len();
            for time in &channel.sampler.input {
                buffer_data.extend_from_slice(&time.to_le_bytes());
            }
            let times_length = buffer_data.len() - times_start;

            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(times_start)),
                byte_length: USize64::from(times_length),
                byte_stride: None,
                target: None,
                name: None,
                extensions: None,
                extras: Default::default(),
            });

            let time_accessor_index = accessors.len() as u32;
            let min_time = channel.sampler.input.first().copied().unwrap_or(0.0);
            let max_time = channel.sampler.input.last().copied().unwrap_or(0.0);

            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(channel.sampler.input.len()),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Scalar),
                min: Some(json::Value::Array(vec![json::Value::from(min_time)])),
                max: Some(json::Value::Array(vec![json::Value::from(max_time)])),
                name: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            });

            let values_start = buffer_data.len();
            let (accessor_type, path, value_count) =
                match (&channel.target_property, &channel.sampler.output) {
                    (AnimationProperty::Translation, AnimationSamplerOutput::Vec3(values)) => {
                        for v in values {
                            buffer_data.extend_from_slice(&(v.x * scale_factor).to_le_bytes());
                            buffer_data.extend_from_slice(&(v.y * scale_factor).to_le_bytes());
                            buffer_data.extend_from_slice(&(v.z * scale_factor).to_le_bytes());
                        }
                        (
                            json::accessor::Type::Vec3,
                            json::animation::Property::Translation,
                            values.len(),
                        )
                    }
                    (AnimationProperty::Rotation, AnimationSamplerOutput::Quat(values)) => {
                        for q in values {
                            buffer_data.extend_from_slice(&q.i.to_le_bytes());
                            buffer_data.extend_from_slice(&q.j.to_le_bytes());
                            buffer_data.extend_from_slice(&q.k.to_le_bytes());
                            buffer_data.extend_from_slice(&q.w.to_le_bytes());
                        }
                        (
                            json::accessor::Type::Vec4,
                            json::animation::Property::Rotation,
                            values.len(),
                        )
                    }
                    (AnimationProperty::Scale, AnimationSamplerOutput::Vec3(values)) => {
                        for v in values {
                            buffer_data.extend_from_slice(&v.x.to_le_bytes());
                            buffer_data.extend_from_slice(&v.y.to_le_bytes());
                            buffer_data.extend_from_slice(&v.z.to_le_bytes());
                        }
                        (
                            json::accessor::Type::Vec3,
                            json::animation::Property::Scale,
                            values.len(),
                        )
                    }
                    _ => continue,
                };
            let values_length = buffer_data.len() - values_start;

            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(values_start)),
                byte_length: USize64::from(values_length),
                byte_stride: None,
                target: None,
                name: None,
                extensions: None,
                extras: Default::default(),
            });

            let value_accessor_index = accessors.len() as u32;
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(buffer_views.len() as u32 - 1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(value_count),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(accessor_type),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            });

            let sampler_index = anim_samplers.len();
            anim_samplers.push(json::animation::Sampler {
                input: json::Index::new(time_accessor_index),
                output: json::Index::new(value_accessor_index),
                interpolation: json::validation::Checked::Valid(
                    json::animation::Interpolation::Linear,
                ),
                extensions: None,
                extras: Default::default(),
            });

            channels.push(json::animation::Channel {
                sampler: json::Index::new(sampler_index as u32),
                target: json::animation::Target {
                    node: json::Index::new(target_node_idx as u32),
                    path: json::validation::Checked::Valid(path),
                    extensions: None,
                    extras: Default::default(),
                },
                extensions: None,
                extras: Default::default(),
            });
        }

        if !channels.is_empty() {
            gltf_animations.push(json::Animation {
                name: Some(anim.name.clone()),
                channels,
                samplers: anim_samplers,
                extensions: None,
                extras: Default::default(),
            });
        }
    }

    while !buffer_data.len().is_multiple_of(4) {
        buffer_data.push(0);
    }

    let root_nodes: Vec<json::Index<json::Node>> = (0..model.prefab.root_nodes.len())
        .map(|index| json::Index::new(index as u32))
        .collect();

    let mesh_root_start = gltf_nodes.len() - model.meshes.len();
    let mesh_node_indices: Vec<json::Index<json::Node>> = (mesh_root_start..gltf_nodes.len())
        .map(|index| json::Index::new(index as u32))
        .collect();

    let all_root_nodes: Vec<json::Index<json::Node>> =
        root_nodes.into_iter().chain(mesh_node_indices).collect();

    let root = json::Root {
        asset: json::Asset {
            generator: Some("nightshade-asset-browser".to_string()),
            version: "2.0".to_string(),
            ..Default::default()
        },
        accessors,
        buffer_views,
        buffers: vec![json::Buffer {
            byte_length: USize64::from(buffer_data.len()),
            uri: None,
            name: None,
            extensions: None,
            extras: Default::default(),
        }],
        images,
        samplers,
        textures: gltf_textures,
        materials,
        meshes,
        nodes: gltf_nodes,
        skins,
        animations: gltf_animations,
        scenes: vec![json::Scene {
            nodes: all_root_nodes,
            name: Some("Scene".to_string()),
            extensions: None,
            extras: Default::default(),
        }],
        scene: Some(json::Index::new(0)),
        ..Default::default()
    };

    let json_string = json::serialize::to_string_pretty(&root)?;
    let json_bytes = json_string.as_bytes();

    let mut json_chunk = json_bytes.to_vec();
    while json_chunk.len() % 4 != 0 {
        json_chunk.push(0x20);
    }

    let mut glb: Vec<u8> = Vec::new();

    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    let total_length = 12 + 8 + json_chunk.len() + 8 + buffer_data.len();
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    glb.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(&json_chunk);

    glb.extend_from_slice(&(buffer_data.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(&buffer_data);

    Ok(glb)
}
