struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    inverse_view_proj: mat4x4<f32>,
    viewport_size: vec2<f32>,
    time: f32,
    tile_count: u32,
    hex_width: f32,
    hex_depth: f32,
    _pad0: f32,
    _pad1: f32,
};

struct TilePositions {
    data: array<vec4<f32>, 256>,
};

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var depth_texture: texture_depth_2d;

@group(0) @binding(2)
var tex_sampler: sampler;

@group(0) @binding(3)
var<uniform> uniforms: Uniforms;

@group(0) @binding(4)
var<storage, read> tiles: TilePositions;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

fn world_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc_x = uv.x * 2.0 - 1.0;
    let ndc_y = -(uv.y * 2.0 - 1.0);
    let clip = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    let world = uniforms.inverse_view_proj * clip;
    return world.xyz / world.w;
}

fn point_in_flat_top_hex(px: f32, pz: f32, cx: f32, cz: f32) -> f32 {
    let inset = 0.92;
    let half_w = uniforms.hex_width * 0.5 * inset;
    let half_h = uniforms.hex_depth * 0.5 * inset;
    let quarter_w = uniforms.hex_width * 0.25 * inset;

    let dx = abs(px - cx);
    let dz = abs(pz - cz);

    if dx > half_w || dz > half_h {
        return -1.0;
    }

    if dx <= quarter_w {
        let edge_dist = min(half_w - dx, half_h - dz);
        return edge_dist / half_w;
    }

    let max_dz = half_h * (half_w - dx) / (half_w - quarter_w);
    if dz > max_dz {
        return -1.0;
    }

    let edge_dist = max_dz - dz;
    return edge_dist / half_h;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSampleLevel(scene_texture, tex_sampler, in.uv, 0.0);

    if uniforms.tile_count == 0u {
        return scene_color;
    }

    let pixel = vec2<i32>(in.position.xy);
    let depth = textureLoad(depth_texture, pixel, 0);

    if depth <= 0.0 {
        return scene_color;
    }

    let world_pos = world_from_depth(in.uv, depth);

    var best_dist: f32 = -1.0;
    for (var index = 0u; index < uniforms.tile_count; index++) {
        let tile = tiles.data[index];
        let dist = point_in_flat_top_hex(world_pos.x, world_pos.z, tile.x, tile.z);
        if dist > best_dist {
            best_dist = dist;
        }
    }

    if best_dist < 0.0 {
        return scene_color;
    }

    let pulse = sin(uniforms.time * 3.0) * 0.5 + 0.5;
    let edge_fade = smoothstep(0.0, 0.15, best_dist);
    let strength = (0.12 + 0.06 * pulse) * edge_fade;

    let tint = vec3<f32>(0.6, 0.85, 1.0);
    let tinted = scene_color.rgb + tint * strength;

    return vec4<f32>(tinted, scene_color.a);
}
