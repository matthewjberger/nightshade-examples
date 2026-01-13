struct RenderUniforms {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    sun_direction: vec4<f32>,
    height_scale: f32,
    fog_start: f32,
    fog_end: f32,
    _padding: f32,
}

struct TerrainVertex {
    position: vec4<f32>,
    normal: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) height: f32,
}

@group(0) @binding(0) var<uniform> uniforms: RenderUniforms;
@group(0) @binding(1) var<storage, read> vertices: array<TerrainVertex>;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let vertex = vertices[vertex_index];

    var output: VertexOutput;
    output.clip_position = uniforms.view_projection * vertex.position;
    output.world_position = vertex.position.xyz;
    output.normal = vertex.normal.xyz;
    output.height = vertex.position.y;

    return output;
}

fn height_color(height: f32, height_scale: f32) -> vec3<f32> {
    let normalized_height = (height / height_scale + 1.0) * 0.5;

    let water_color = vec3<f32>(0.1, 0.3, 0.5);
    let sand_color = vec3<f32>(0.76, 0.7, 0.5);
    let grass_color = vec3<f32>(0.2, 0.5, 0.15);
    let rock_color = vec3<f32>(0.5, 0.45, 0.4);
    let snow_color = vec3<f32>(0.95, 0.95, 0.97);

    var color: vec3<f32>;
    if (normalized_height < 0.3) {
        let t = normalized_height / 0.3;
        color = mix(water_color, sand_color, t);
    } else if (normalized_height < 0.4) {
        let t = (normalized_height - 0.3) / 0.1;
        color = mix(sand_color, grass_color, t);
    } else if (normalized_height < 0.7) {
        let t = (normalized_height - 0.4) / 0.3;
        color = mix(grass_color, rock_color, t);
    } else {
        let t = (normalized_height - 0.7) / 0.3;
        color = mix(rock_color, snow_color, t);
    }

    return color;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = height_color(input.height, uniforms.height_scale);

    let normal = normalize(input.normal);
    let light_dir = normalize(uniforms.sun_direction.xyz);
    let ndotl = max(dot(normal, light_dir), 0.0);

    let ambient = 0.3;
    let diffuse = ndotl * 0.7;
    let lit_color = base_color * (ambient + diffuse);

    let dist = distance(input.world_position, uniforms.camera_position.xyz);
    let fog_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    let fog_color = vec3<f32>(0.7, 0.8, 0.9);

    let final_color = mix(lit_color, fog_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}

fn grid_line(coord: f32, line_width: f32) -> f32 {
    let d = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5);
    return 1.0 - smoothstep(0.0, d * line_width, grid);
}

@fragment
fn fragment_wireframe(input: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(input.world_position, uniforms.camera_position.xyz);

    var grid_size: f32;
    if (dist < 50.0) {
        grid_size = 0.5;
    } else if (dist < 100.0) {
        grid_size = 1.0;
    } else if (dist < 200.0) {
        grid_size = 2.0;
    } else if (dist < 400.0) {
        grid_size = 4.0;
    } else {
        grid_size = 8.0;
    }

    let scaled_x = input.world_position.x / grid_size;
    let scaled_z = input.world_position.z / grid_size;

    let line_x = grid_line(scaled_x, 1.5);
    let line_z = grid_line(scaled_z, 1.5);
    let line_intensity = max(line_x, line_z);

    let base_color = height_color(input.height, uniforms.height_scale);

    let normal = normalize(input.normal);
    let light_dir = normalize(uniforms.sun_direction.xyz);
    let ndotl = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;
    let diffuse = ndotl * 0.7;
    let lit_base = base_color * (ambient + diffuse) * 0.3;

    let wireframe_color = vec3<f32>(0.2, 0.9, 0.3);
    let final_color = mix(lit_base, wireframe_color, line_intensity);

    let fog_factor = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    let fog_color = vec3<f32>(0.7, 0.8, 0.9);
    let fogged_color = mix(final_color, fog_color, fog_factor);

    return vec4<f32>(fogged_color, 1.0);
}
