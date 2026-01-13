struct Uniforms {
    view_projection: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    camera_position: vec4<f32>,
    time: f32,
    tiled_band_size: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var sky_texture: texture_2d<u32>;

@group(0) @binding(2)
var palette_texture: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

const PI: f32 = 3.14159265359;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = uniforms.view_projection * vec4<f32>(input.position, 1.0);
    output.world_pos = input.position;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sky_dims = vec2<f32>(textureDimensions(sky_texture));

    let view_dir = normalize(input.world_pos - uniforms.camera_position.xyz);
    let yaw = atan2(view_dir.x, view_dir.z);
    let pitch = asin(clamp(view_dir.y, -1.0, 1.0));

    let u = yaw / PI;
    var v = 0.5 - pitch / PI;

    let tiled_band = uniforms.tiled_band_size;
    if v < 0.0 {
        v = abs(v);
    } else if v >= 1.0 {
        v = 2.0 - v;
    }
    v = clamp(v, 0.0, 0.999);

    var tex_x = i32(u * sky_dims.x) % i32(sky_dims.x);
    if tex_x < 0 {
        tex_x = tex_x + i32(sky_dims.x);
    }
    let tex_y = i32(v * sky_dims.y);

    let palette_index = textureLoad(sky_texture, vec2<i32>(tex_x, tex_y), 0).r;

    let palette_coords = vec2<i32>(i32(palette_index), 0);
    let color = textureLoad(palette_texture, palette_coords, 0);

    return vec4<f32>(color.rgb, 1.0);
}
