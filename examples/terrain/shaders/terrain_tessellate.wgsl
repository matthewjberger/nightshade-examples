struct TerrainUniforms {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    height_scale: f32,
    noise_frequency: f32,
    noise_octaves: u32,
    patch_count: u32,
    lod_distances: vec4<f32>,
    lod_distance_4: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

struct PatchInput {
    world_x: f32,
    world_z: f32,
    patch_size: f32,
    _padding: f32,
}

struct TerrainVertex {
    position: vec4<f32>,
    normal: vec4<f32>,
}

struct Counters {
    vertex_count: atomic<u32>,
    index_count: atomic<u32>,
}

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

@group(0) @binding(0) var<uniform> uniforms: TerrainUniforms;
@group(0) @binding(1) var<storage, read> patches: array<PatchInput>;
@group(0) @binding(2) var<storage, read_write> vertices: array<TerrainVertex>;
@group(0) @binding(3) var<storage, read_write> indices: array<u32>;
@group(0) @binding(4) var<storage, read_write> counters: Counters;
@group(0) @binding(5) var<storage, read_write> draw_indirect: DrawIndexedIndirect;

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4<f32>) -> vec4<f32> {
    return mod289_4(((x * 34.0) + 1.0) * x);
}

fn taylor_inv_sqrt(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn simplex_noise_2d(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    var i1: vec2<f32>;
    if (x0.x > x0.y) {
        i1 = vec2<f32>(1.0, 0.0);
    } else {
        i1 = vec2<f32>(0.0, 1.0);
    }

    var x12 = x0.xyxy + C.xxzz;
    x12 = vec4<f32>(x12.xy - i1, x12.zw);

    i = i - floor(i * (1.0 / 289.0)) * 289.0;
    let p = permute(permute(i.y + vec4<f32>(0.0, i1.y, 1.0, 0.0)) + i.x + vec4<f32>(0.0, i1.x, 1.0, 0.0));

    var m = max(0.5 - vec4<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw), 0.0), vec4<f32>(0.0));
    m = m * m;
    m = m * m;

    let x = 2.0 * fract(p * C.wwww) - 1.0;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

    let g0 = a0.x * x0.x + h.x * x0.y;
    let g1 = a0.y * x12.x + h.y * x12.y;
    let g2 = a0.z * x12.z + h.z * x12.w;

    return 130.0 * dot(m.xyz, vec3<f32>(g0, g1, g2));
}

fn fbm_2d(p: vec2<f32>, octaves: u32, base_frequency: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = base_frequency;
    var max_value = 0.0;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * simplex_noise_2d(p * frequency);
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value / max_value;
}

fn sample_height(world_x: f32, world_z: f32) -> f32 {
    let p = vec2<f32>(world_x, world_z);
    let noise = fbm_2d(p, uniforms.noise_octaves, uniforms.noise_frequency);
    return noise * uniforms.height_scale;
}

fn calculate_normal(world_x: f32, world_z: f32) -> vec3<f32> {
    let epsilon = 0.5;

    let h_left = sample_height(world_x - epsilon, world_z);
    let h_right = sample_height(world_x + epsilon, world_z);
    let h_back = sample_height(world_x, world_z - epsilon);
    let h_front = sample_height(world_x, world_z + epsilon);

    return normalize(vec3<f32>(h_left - h_right, 2.0 * epsilon, h_back - h_front));
}

fn calculate_lod(patch_center: vec3<f32>) -> u32 {
    let dist = distance(patch_center, uniforms.camera_position.xyz);

    if (dist < uniforms.lod_distances.x) {
        return 16u;
    } else if (dist < uniforms.lod_distances.y) {
        return 8u;
    } else if (dist < uniforms.lod_distances.z) {
        return 4u;
    } else if (dist < uniforms.lod_distances.w) {
        return 2u;
    }
    return 2u;
}

@compute @workgroup_size(64)
fn tessellate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let patch_index = global_id.x;
    if (patch_index >= uniforms.patch_count) {
        return;
    }

    let patch_data = patches[patch_index];
    let patch_size = patch_data.patch_size;

    let patch_center_x = patch_data.world_x + patch_size * 0.5;
    let patch_center_z = patch_data.world_z + patch_size * 0.5;
    let patch_center_y = sample_height(patch_center_x, patch_center_z);
    let patch_center = vec3<f32>(patch_center_x, patch_center_y, patch_center_z);

    let subdivisions = calculate_lod(patch_center);
    let step = patch_size / f32(subdivisions);
    let verts_per_side = subdivisions + 1u;
    let verts_count = verts_per_side * verts_per_side;
    let quads_count = subdivisions * subdivisions;
    let indices_count = quads_count * 6u;

    let base_vertex = atomicAdd(&counters.vertex_count, verts_count);
    let base_index = atomicAdd(&counters.index_count, indices_count);

    let max_vertices = 2000000u;
    let max_indices = 6000000u;

    if (base_vertex + verts_count > max_vertices || base_index + indices_count > max_indices) {
        return;
    }

    for (var z = 0u; z <= subdivisions; z++) {
        for (var x = 0u; x <= subdivisions; x++) {
            let world_x = patch_data.world_x + f32(x) * step;
            let world_z = patch_data.world_z + f32(z) * step;
            let height = sample_height(world_x, world_z);
            let normal = calculate_normal(world_x, world_z);

            let vertex_index = base_vertex + z * verts_per_side + x;
            vertices[vertex_index] = TerrainVertex(
                vec4<f32>(world_x, height, world_z, 1.0),
                vec4<f32>(normal, 0.0)
            );
        }
    }

    var index_offset = base_index;
    for (var z = 0u; z < subdivisions; z++) {
        for (var x = 0u; x < subdivisions; x++) {
            let top_left = base_vertex + z * verts_per_side + x;
            let top_right = top_left + 1u;
            let bottom_left = top_left + verts_per_side;
            let bottom_right = bottom_left + 1u;

            indices[index_offset + 0u] = top_left;
            indices[index_offset + 1u] = bottom_left;
            indices[index_offset + 2u] = top_right;
            indices[index_offset + 3u] = top_right;
            indices[index_offset + 4u] = bottom_left;
            indices[index_offset + 5u] = bottom_right;

            index_offset += 6u;
        }
    }
}

@compute @workgroup_size(1)
fn reset_counters() {
    atomicStore(&counters.vertex_count, 0u);
    atomicStore(&counters.index_count, 0u);
}

@compute @workgroup_size(1)
fn finalize_indirect() {
    draw_indirect.index_count = atomicLoad(&counters.index_count);
    draw_indirect.instance_count = 1u;
    draw_indirect.first_index = 0u;
    draw_indirect.base_vertex = 0i;
    draw_indirect.first_instance = 0u;
}
