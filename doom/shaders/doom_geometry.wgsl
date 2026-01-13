struct Uniforms {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    time: f32,
    atlas_width: f32,
    atlas_height: f32,
    _padding2: f32,
    _padding3: vec4<f32>,
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
    @location(4) light: f32,
    @location(5) num_frames: f32,
    @location(6) scroll_rate: f32,
    @location(7) row_height: f32,
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
    let projected = uniforms.view_projection * vec4<f32>(input.position, 1.0);
    output.clip_position = projected;
    output.dist = projected.w;

    var tile_uv = input.tile_uv + vec2<f32>(uniforms.time * input.scroll_rate, 0.0);

    var atlas_uv = input.atlas_uv;
    let num_frames = i32(input.num_frames);
    if num_frames > 1 {
        let frame_index = floor(fract(uniforms.time / ANIM_FPS / f32(num_frames)) * f32(num_frames));

        var atlas_u = input.atlas_uv.x + frame_index * input.tile_size.x;
        let rows_down = ceil((atlas_u + input.tile_size.x) / uniforms.atlas_width) - 1.0;
        atlas_u = atlas_u + (uniforms.atlas_width - input.atlas_uv.x) % input.tile_size.x * rows_down;

        let atlas_v = input.atlas_uv.y + rows_down * input.row_height;
        atlas_uv = vec2<f32>(atlas_u, atlas_v);
    }

    output.atlas_uv = atlas_uv;
    output.tile_uv = tile_uv;
    output.tile_size = input.tile_size;
    output.light = input.light;
    return output;
}

const LIGHT_SCALE: f32 = 2.5;
const DIST_SCALE: f32 = 0.9;
const NUM_COLORMAPS: f32 = 31.0;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tile_size = input.tile_size;

    var tile_u = input.tile_uv.x;
    var tile_v = input.tile_uv.y;

    tile_u = tile_u % tile_size.x;
    tile_v = tile_v % tile_size.y;

    if tile_u < 0.0 {
        tile_u = tile_u + tile_size.x;
    }
    if tile_v < 0.0 {
        tile_v = tile_v + tile_size.y;
    }

    let atlas_x = input.atlas_uv.x + tile_u;
    let atlas_y = input.atlas_uv.y + tile_v;

    let texel_coords = vec2<i32>(i32(atlas_x), i32(atlas_y));
    let palette_index = textureLoad(atlas_texture, texel_coords, 0).r;

    if palette_index >= 0xff00u {
        discard;
    }

    let dist_term = min(1.0, 1.0 - DIST_SCALE / (input.dist + DIST_SCALE));
    let light = clamp(input.light * LIGHT_SCALE - dist_term, 0.0, 1.0);
    let colormap_row = i32((1.0 - light) * NUM_COLORMAPS);

    let palette_coords = vec2<i32>(i32(palette_index), colormap_row);
    let color = textureLoad(palette_texture, palette_coords, 0);

    return vec4<f32>(color.rgb, 1.0);
}
