struct Uniforms {
    time: f32,
    mode: u32,
    resolution: vec2<f32>,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

fn hash_1d(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let dist = dot(centered, centered);
    return uv + centered * dist * amount;
}

fn crt_effect(uv: vec2<f32>, time: f32, resolution: vec2<f32>) -> vec4<f32> {
    let distorted_uv = barrel_distort(uv, 0.2);

    if distorted_uv.x < 0.0 || distorted_uv.x > 1.0 || distorted_uv.y < 0.0 || distorted_uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let aberration = 0.003;
    let r = textureSample(input_texture, input_sampler, distorted_uv + vec2(aberration, 0.0)).r;
    let g = textureSample(input_texture, input_sampler, distorted_uv).g;
    let b = textureSample(input_texture, input_sampler, distorted_uv - vec2(aberration, 0.0)).b;
    var color = vec3<f32>(r, g, b);

    let scanline_y = distorted_uv.y * resolution.y;
    let scanline = 0.75 + 0.25 * sin(scanline_y * 3.14159 * 2.0);
    color *= scanline;

    let sub_pixel = floor(distorted_uv.x * resolution.x * 3.0) % 3.0;
    var rgb_mask = vec3<f32>(0.8, 0.8, 0.8);
    if sub_pixel < 1.0 {
        rgb_mask.x = 1.0;
    } else if sub_pixel < 2.0 {
        rgb_mask.y = 1.0;
    } else {
        rgb_mask.z = 1.0;
    }
    color *= rgb_mask;

    color = color * vec3<f32>(0.85, 1.1, 0.85);

    let centered = distorted_uv - 0.5;
    let vignette = 1.0 - dot(centered, centered) * 2.0;
    color *= clamp(vignette, 0.0, 1.0);

    let flicker = 0.96 + 0.04 * sin(time * 7.3);
    color *= flicker;

    let roll = fract(time * 0.03);
    let roll_line = smoothstep(0.0, 0.02, abs(distorted_uv.y - roll));
    color *= 0.7 + 0.3 * roll_line;

    return vec4<f32>(color, 1.0);
}

fn underwater_effect(uv: vec2<f32>, time: f32) -> vec4<f32> {
    let wave_x = sin(uv.y * 25.0 + time * 2.5) * 0.008 + sin(uv.y * 40.0 - time * 1.8) * 0.004;
    let wave_y = cos(uv.x * 20.0 + time * 2.0) * 0.005 + cos(uv.x * 35.0 + time * 1.2) * 0.003;
    let distorted_uv = clamp(uv + vec2(wave_x, wave_y), vec2(0.0), vec2(1.0));

    let blur_size = 0.0015;
    var color = textureSample(input_texture, input_sampler, distorted_uv).rgb;
    color += textureSample(input_texture, input_sampler, distorted_uv + vec2(blur_size, 0.0)).rgb;
    color += textureSample(input_texture, input_sampler, distorted_uv - vec2(blur_size, 0.0)).rgb;
    color += textureSample(input_texture, input_sampler, distorted_uv + vec2(0.0, blur_size)).rgb;
    color += textureSample(input_texture, input_sampler, distorted_uv - vec2(0.0, blur_size)).rgb;
    color /= 5.0;

    color = color * vec3<f32>(0.65, 0.85, 1.25);

    let caustic1 = sin(uv.x * 30.0 + time * 1.5) * sin(uv.y * 25.0 + time * 1.2);
    let caustic2 = sin((uv.x + uv.y) * 20.0 - time * 1.0) * sin((uv.x - uv.y) * 22.0 + time * 0.8);
    let caustic3 = sin(length(uv - 0.5) * 40.0 + time * 2.0) * 0.5;
    let caustic = (caustic1 + caustic2 + caustic3) * 0.04;
    color += vec3<f32>(caustic * 0.4, caustic * 0.7, caustic);

    let depth = 1.0 - length(uv - 0.5) * 0.4;
    color *= depth;

    let god_ray = smoothstep(0.3, 0.0, abs(uv.x - 0.5 - sin(time * 0.3) * 0.2));
    let god_ray_fade = smoothstep(0.0, 1.0, uv.y);
    color += vec3<f32>(0.02, 0.04, 0.06) * god_ray * god_ray_fade;

    return vec4<f32>(color, 1.0);
}

fn glitch_effect(uv: vec2<f32>, time: f32) -> vec4<f32> {
    var glitch_uv = uv;

    let time_seed = floor(time * 8.0);
    let strong_glitch = step(0.85, hash_1d(time_seed * 13.37));

    let line_seed = floor(uv.y * 100.0) + time_seed * 100.0;
    let line_intensity = step(0.92 - strong_glitch * 0.3, hash_1d(line_seed));
    let displacement = (hash_1d(line_seed + 0.1) - 0.5) * 0.15 * line_intensity;
    glitch_uv.x += displacement;

    let block_size = 8.0 + strong_glitch * 4.0;
    let block_x = floor(uv.x * block_size);
    let block_y = floor(uv.y * block_size);
    let block_seed = block_x + block_y * block_size + time_seed * 100.0;
    let block_intensity = step(0.96 - strong_glitch * 0.15, hash_1d(block_seed));
    let block_offset = vec2(
        (hash_1d(block_seed + 0.2) - 0.5) * 0.12,
        (hash_1d(block_seed + 0.3) - 0.5) * 0.06
    );
    glitch_uv += block_offset * block_intensity;

    let wave_offset = sin(uv.y * 100.0 + time * 30.0) * 0.002 * strong_glitch;
    glitch_uv.x += wave_offset;

    glitch_uv = clamp(glitch_uv, vec2(0.0), vec2(1.0));

    let separation = 0.006 + line_intensity * 0.015 + strong_glitch * 0.01;
    let r = textureSample(input_texture, input_sampler, glitch_uv + vec2(separation, 0.0)).r;
    let g = textureSample(input_texture, input_sampler, glitch_uv).g;
    let b = textureSample(input_texture, input_sampler, glitch_uv - vec2(separation, 0.0)).b;
    var color = vec3<f32>(r, g, b);

    let noise_band = step(0.96, hash_1d(floor(uv.y * 300.0) + time * 15.0));
    let noise_color = vec3(hash_1d(uv.x * 200.0 + time * 3.0));
    color = mix(color, noise_color, noise_band * 0.6);

    let tear_pos = hash_1d(time_seed + 7.7) * strong_glitch;
    let tear = smoothstep(0.0, 0.01, abs(uv.y - tear_pos));
    color *= 0.5 + 0.5 * tear;

    let scanline = 0.92 + 0.08 * sin(uv.y * 800.0);
    color *= scanline;

    return vec4<f32>(color, 1.0);
}

fn plasma_effect(uv: vec2<f32>, time: f32) -> vec4<f32> {
    let scene = textureSample(input_texture, input_sampler, uv).rgb;

    let scaled = (uv - 0.5) * 8.0;
    let v1 = sin(scaled.x + time * 1.3);
    let v2 = sin(scaled.y + time * 0.9);
    let v3 = sin((scaled.x + scaled.y) * 0.7 + time * 0.7);
    let v4 = sin(length(scaled) * 1.2 - time * 1.1);
    let v5 = sin(scaled.x * sin(time * 0.3) + scaled.y * cos(time * 0.5));
    let v = (v1 + v2 + v3 + v4 + v5) * 0.2;

    let plasma_r = sin(v * 3.14159 + time * 0.5) * 0.5 + 0.5;
    let plasma_g = sin(v * 3.14159 + 2.094 + time * 0.3) * 0.5 + 0.5;
    let plasma_b = sin(v * 3.14159 + 4.189 + time * 0.7) * 0.5 + 0.5;
    let plasma = vec3<f32>(plasma_r, plasma_g, plasma_b);

    let blend_strength = 0.3 + 0.1 * sin(time * 0.2);
    let result = scene + plasma * blend_strength;

    let pulse = 0.95 + 0.05 * sin(length(uv - 0.5) * 10.0 - time * 3.0);
    return vec4<f32>(result * pulse, 1.0);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    switch uniforms.mode {
        case 1u: { return crt_effect(in.uv, uniforms.time, uniforms.resolution); }
        case 2u: { return underwater_effect(in.uv, uniforms.time); }
        case 3u: { return glitch_effect(in.uv, uniforms.time); }
        case 4u: { return plasma_effect(in.uv, uniforms.time); }
        default: { return textureSample(input_texture, input_sampler, in.uv); }
    }
}
