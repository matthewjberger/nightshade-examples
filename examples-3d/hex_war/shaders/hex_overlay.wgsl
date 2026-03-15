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

fn point_in_flat_top_hex(px: f32, pz: f32, cx: f32, cz: f32) -> bool {
    let dx = abs(px - cx);
    let dz = abs(pz - cz);
    let half_w = uniforms.hex_width * 0.5;
    let half_h = uniforms.hex_depth * 0.5;
    if dx > half_w || dz > half_h {
        return false;
    }
    let quarter_w = uniforms.hex_width * 0.25;
    if dx <= quarter_w {
        return true;
    }
    let slope = half_h / (half_w - quarter_w);
    return dz <= slope * (half_w - dx);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSampleLevel(scene_texture, tex_sampler, in.uv, 0.0);

    let pixel = vec2<i32>(in.position.xy);
    let depth = textureLoad(depth_texture, pixel, 0);

    if depth <= 0.0 || uniforms.tile_count == 0u {
        return scene_color;
    }

    let world_pos = world_from_depth(in.uv, depth);

    var inside = false;
    for (var index = 0u; index < uniforms.tile_count; index++) {
        let tile = tiles.data[index];
        if point_in_flat_top_hex(world_pos.x, world_pos.z, tile.x, tile.z) {
            inside = true;
            break;
        }
    }

    if !inside {
        return scene_color;
    }

    let pulse = sin(uniforms.time * 3.0) * 0.5 + 0.5;
    let overlay_alpha = 0.2 + 0.1 * pulse;
    let overlay_rgb = vec3<f32>(0.3, 0.6, 1.0);

    return vec4<f32>(
        mix(scene_color.rgb, overlay_rgb, overlay_alpha),
        scene_color.a,
    );
}
