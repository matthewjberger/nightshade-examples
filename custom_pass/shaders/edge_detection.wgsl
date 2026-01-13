struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = textureDimensions(input_texture);
    let texel_size = vec2<f32>(1.0 / f32(texture_size.x), 1.0 / f32(texture_size.y));

    let tl = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, -texel_size.y)).rgb);
    let tm = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, -texel_size.y)).rgb);
    let tr = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, -texel_size.y)).rgb);

    let ml = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, 0.0)).rgb);
    let mr = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, 0.0)).rgb);

    let bl = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, texel_size.y)).rgb);
    let bm = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, texel_size.y)).rgb);
    let br = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, texel_size.y)).rgb);

    let gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
    let gy = -tl - 2.0 * tm - tr + bl + 2.0 * bm + br;

    let edge_strength = sqrt(gx * gx + gy * gy);

    let original = textureSample(input_texture, input_sampler, in.uv).rgb;

    let result = original + vec3<f32>(edge_strength);

    return vec4<f32>(result, 1.0);
}
