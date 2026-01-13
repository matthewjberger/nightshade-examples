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

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = textureDimensions(input_texture);
    let texel_size = 1.0 / f32(texture_size.x);

    let weights = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

    var result = textureSample(input_texture, input_sampler, in.uv).rgb * weights[0];

    for (var i: i32 = 1; i < 5; i++) {
        let offset = f32(i) * texel_size;
        result += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset, 0.0)).rgb * weights[i];
        result += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(offset, 0.0)).rgb * weights[i];
    }

    return vec4<f32>(result, 1.0);
}
