struct Uniforms {
    view_projection: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    camera_position: vec4<f32>,
    time: f32,
    atlas_width: f32,
    atlas_height: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var atlas_texture: texture_2d<u32>;

@group(0) @binding(2)
var palette_texture: texture_2d<f32>;

@group(0) @binding(3)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) atlas_uv: vec2<f32>,
    @location(2) tile_uv: vec2<f32>,
    @location(3) tile_size: vec2<f32>,
    @location(4) local_x: f32,
    @location(5) light: f32,
    @location(6) num_frames: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) atlas_uv: vec2<f32>,
    @location(1) tile_uv: vec2<f32>,
    @location(2) tile_size: vec2<f32>,
    @location(3) light: f32,
    @location(4) dist: f32,
}

const ANIM_FPS: f32 = 8.0 / 35.0;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let right = vec3<f32>(uniforms.view_matrix[0][0], uniforms.view_matrix[1][0], uniforms.view_matrix[2][0]);
    let pos = input.position + right * input.local_x;

    let projected = uniforms.view_projection * vec4<f32>(pos, 1.0);
    output.clip_position = projected;
    output.dist = projected.w;

    var atlas_uv = input.atlas_uv;
    let num_frames = i32(input.num_frames);
    if num_frames > 1 {
        let frame_index = floor(fract(uniforms.time / ANIM_FPS / f32(num_frames)) * f32(num_frames));

        var atlas_u = input.atlas_uv.x + frame_index * input.tile_size.x;
        let rows_down = ceil((atlas_u + input.tile_size.x) / uniforms.atlas_width) - 1.0;
        atlas_u = atlas_u + (uniforms.atlas_width - input.atlas_uv.x) % input.tile_size.x * rows_down;

        let atlas_v = input.atlas_uv.y + rows_down * input.tile_size.y;
        atlas_uv = vec2<f32>(atlas_u, atlas_v);
    }

    output.atlas_uv = atlas_uv;
    output.tile_uv = input.tile_uv;
    output.tile_size = input.tile_size;
    output.light = input.light;
    return output;
}

const LIGHT_SCALE: f32 = 2.5;
const DIST_SCALE: f32 = 1.0;
const NUM_COLORMAPS: f32 = 31.0;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tile_u = clamp(input.tile_uv.x, 0.0, input.tile_size.x - 0.001);
    let tile_v = clamp(input.tile_uv.y, 0.0, input.tile_size.y - 0.001);

    let atlas_x = input.atlas_uv.x + tile_u;
    let atlas_y = input.atlas_uv.y + tile_v;

    let texel_coords = vec2<i32>(i32(atlas_x), i32(atlas_y));
    let palette_index = textureLoad(atlas_texture, texel_coords, 0).r;

    if palette_index >= 0xff00u {
        discard;
    }

    let dist_term = min(1.0, 1.0 - DIST_SCALE / (input.dist + DIST_SCALE));
    let light = min(input.light, clamp(input.light * LIGHT_SCALE - dist_term, 0.0, 1.0));
    let colormap_row = i32((1.0 - light) * NUM_COLORMAPS);

    let palette_coords = vec2<i32>(i32(palette_index), colormap_row);
    let color = textureLoad(palette_texture, palette_coords, 0);

    return vec4<f32>(color.rgb, 1.0);
}
