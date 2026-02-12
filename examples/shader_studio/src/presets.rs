use crate::shader_pass::ChannelSource;

pub struct ShaderPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
    pub is_geometry: bool,
    pub category: &'static str,
    pub slider_labels: &'static [(usize, &'static str)],
    pub slider_defaults: &'static [(usize, f32)],
    pub buffer_a_source: Option<&'static str>,
    pub buffer_b_source: Option<&'static str>,
    pub buffer_c_source: Option<&'static str>,
    pub buffer_d_source: Option<&'static str>,
    pub common_source: Option<&'static str>,
    pub channel_bindings: Option<[[ChannelSource; 4]; 5]>,
}

macro_rules! uniform_preamble {
    () => {
        r"
struct Uniforms {
    time: f32,
    delta_time: f32,
    frame: u32,
    _pad0: u32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    custom: array<vec4<f32>, 4>,
    camera_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
"
    };
}

macro_rules! texture_preamble {
    () => {
        r"
@group(1) @binding(0) var texture_0: texture_2d<f32>;
@group(1) @binding(1) var sampler_0: sampler;
@group(1) @binding(2) var texture_1: texture_2d<f32>;
@group(1) @binding(3) var sampler_1: sampler;
@group(1) @binding(4) var texture_2: texture_2d<f32>;
@group(1) @binding(5) var sampler_2: sampler;
@group(1) @binding(6) var texture_3: texture_2d<f32>;
@group(1) @binding(7) var sampler_3: sampler;
"
    };
}

macro_rules! fullscreen_vertex {
    () => {
        r"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}
"
    };
}

macro_rules! geometry_vertex {
    () => {
        r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}
"
    };
}

pub const PRESETS: &[ShaderPreset] = &[
    ShaderPreset {
        name: "Plasma",
        description: "Animated plasma color waves",
        source: PLASMA,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Mandelbrot",
        description: "Fractal zoom into the Mandelbrot set",
        source: MANDELBROT,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Voronoi",
        description: "Animated cellular noise pattern",
        source: VORONOI,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Warp Tunnel",
        description: "Hypnotic warp-speed tunnel effect",
        source: WARP_TUNNEL,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Fractal Noise",
        description: "Layered simplex noise landscape",
        source: FRACTAL_NOISE,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Kaleidoscope",
        description: "Mirrored rotating kaleidoscope",
        source: KALEIDOSCOPE,
        is_geometry: false,
        category: "2D Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Ray March SDF",
        description: "Sphere tracing with signed distance fields",
        source: RAY_MARCH,
        is_geometry: false,
        category: "Raymarching",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "2D SDF Shapes (IQ)",
        description: "Inigo Quilez 2D distance functions gallery",
        source: SDF_2D_SHAPES,
        is_geometry: false,
        category: "Raymarching",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "IQ Raymarcher",
        description: "3D SDF raymarching with soft shadows & AO",
        source: IQ_RAYMARCHER,
        is_geometry: false,
        category: "Raymarching",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Ocean Surface",
        description: "Animated ocean waves with fake reflections",
        source: OCEAN_SURFACE,
        is_geometry: false,
        category: "Raymarching",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Lit Mesh",
        description: "Diffuse + specular lighting on geometry",
        source: LIT_MESH,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Color R"), (1, "Color G"), (2, "Color B")],
        slider_defaults: &[(0, 0.7), (1, 0.3), (2, 0.2)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Normal Map",
        description: "Visualize surface normals as RGB colors",
        source: NORMAL_MAP,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "UV Checker",
        description: "Checkerboard UV visualization",
        source: UV_CHECKER,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Wireframe Glow",
        description: "Edge-highlighted wireframe effect on mesh",
        source: WIREFRAME_GLOW,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Dissolve",
        description: "Noise-driven dissolve/disintegration effect",
        source: DISSOLVE,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Hologram",
        description: "Scanline holographic projection effect",
        source: HOLOGRAM,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "01: Hello Color",
        description: "Your first shader - output a solid color",
        source: TUTORIAL_HELLO_COLOR,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "02: UV Coordinates",
        description: "Understand the UV coordinate system",
        source: TUTORIAL_UV_GRADIENT,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "03: Animation",
        description: "Animate with time, sin(), and cos()",
        source: TUTORIAL_ANIMATION,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "04: Mouse & Aspect",
        description: "Mouse interaction and aspect-correct shapes",
        source: TUTORIAL_MOUSE,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "05: Slider Controls",
        description: "Control shader parameters with custom sliders",
        source: TUTORIAL_SLIDERS,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[
            (0, "Color R"),
            (1, "Color G"),
            (2, "Color B"),
            (4, "Ring Density"),
            (5, "Speed"),
        ],
        slider_defaults: &[(0, 0.7), (1, 0.3), (2, 0.2)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "06: Patterns & Math",
        description: "Checkerboards, circles, and shader math",
        source: TUTORIAL_PATTERNS,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "07: Texture Sampling",
        description: "Load and sample textures with drag & drop",
        source: TUTORIAL_TEXTURES,
        is_geometry: false,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "08: 3D Geometry",
        description: "Switch to geometry mode and explore normals",
        source: TUTORIAL_GEOMETRY_INTRO,
        is_geometry: true,
        category: "Tutorial",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "09: 3D Lighting",
        description: "Blinn-Phong lighting with ambient, diffuse, specular",
        source: TUTORIAL_LIGHTING,
        is_geometry: true,
        category: "Tutorial",
        slider_labels: &[(0, "Color R"), (1, "Color G"), (2, "Color B")],
        slider_defaults: &[(0, 0.7), (1, 0.3), (2, 0.2)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "10: All Together",
        description: "Combine everything: textures, lighting, sliders, animation",
        source: TUTORIAL_EVERYTHING,
        is_geometry: true,
        category: "Tutorial",
        slider_labels: &[
            (0, "Color R"),
            (1, "Color G"),
            (2, "Color B"),
            (3, "Emission"),
            (4, "Rim Intensity"),
        ],
        slider_defaults: &[(0, 0.7), (1, 0.3), (2, 0.2), (3, 0.0), (4, 0.0)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Game of Life",
        description: "Cellular automaton with Buffer A feedback",
        source: GAME_OF_LIFE_IMAGE,
        is_geometry: false,
        category: "Multipass",
        slider_labels: &[],
        slider_defaults: &[],
        buffer_a_source: Some(GAME_OF_LIFE_BUFFER_A),
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: Some(GAME_OF_LIFE_BINDINGS),
    },
    ShaderPreset {
        name: "Feedback Blur",
        description: "Accumulative blur with frame feedback",
        source: FEEDBACK_BLUR_IMAGE,
        is_geometry: false,
        category: "Multipass",
        slider_labels: &[(0, "Decay"), (1, "Blur Amt")],
        slider_defaults: &[(0, 0.5), (1, 0.3)],
        buffer_a_source: Some(FEEDBACK_BLUR_BUFFER_A),
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: Some(FEEDBACK_BLUR_BINDINGS),
    },
    ShaderPreset {
        name: "Reaction Diffusion",
        description: "Gray-Scott reaction-diffusion simulation",
        source: REACTION_DIFFUSION_IMAGE,
        is_geometry: false,
        category: "Multipass",
        slider_labels: &[(0, "Feed Rate"), (1, "Kill Rate")],
        slider_defaults: &[(0, 0.34), (1, 0.50)],
        buffer_a_source: Some(REACTION_DIFFUSION_BUFFER_A),
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: Some(REACTION_DIFFUSION_BINDINGS),
    },
    ShaderPreset {
        name: "Toon Shading",
        description: "Cel-shaded cartoon look with outlines",
        source: TOON_SHADING,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Color R"), (1, "Color G"), (2, "Color B"), (3, "Bands")],
        slider_defaults: &[(0, 0.8), (1, 0.3), (2, 0.2), (3, 0.4)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Fresnel Glow",
        description: "Edge-lit glow with fresnel falloff",
        source: FRESNEL_GLOW,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Color R"), (1, "Color G"), (2, "Color B"), (3, "Power")],
        slider_defaults: &[(0, 0.1), (1, 0.5), (2, 1.0), (3, 0.4)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Force Field",
        description: "Hexagonal energy shield effect",
        source: FORCE_FIELD,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[
            (0, "Color R"),
            (1, "Color G"),
            (2, "Color B"),
            (3, "Hex Size"),
        ],
        slider_defaults: &[(0, 0.2), (1, 0.6), (2, 1.0), (3, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "X-Ray",
        description: "Translucent x-ray view with edge enhancement",
        source: XRAY,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[
            (0, "Color R"),
            (1, "Color G"),
            (2, "Color B"),
            (3, "Opacity"),
        ],
        slider_defaults: &[(0, 0.1), (1, 0.7), (2, 0.9), (3, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Lava",
        description: "Animated flowing lava surface",
        source: LAVA,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Heat"), (1, "Flow Speed"), (2, "Crack Width")],
        slider_defaults: &[(0, 0.7), (1, 0.4), (2, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Ice Crystal",
        description: "Frozen crystalline surface with refraction",
        source: ICE_CRYSTAL,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Frost"), (1, "Sparkle"), (2, "Tint")],
        slider_defaults: &[(0, 0.6), (1, 0.5), (2, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Matcap",
        description: "Material capture sphere-mapped shading",
        source: MATCAP,
        is_geometry: true,
        category: "Materials",
        slider_labels: &[(0, "Warm"), (1, "Cool"), (2, "Metallic")],
        slider_defaults: &[(0, 0.7), (1, 0.3), (2, 0.5)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Glitch",
        description: "Digital glitch with vertex displacement and RGB split",
        source: GLITCH,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Intensity"), (1, "Speed"), (2, "Slice Size")],
        slider_defaults: &[(0, 0.5), (1, 0.4), (2, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Shockwave",
        description: "Expanding ring wave deforming mesh surface",
        source: SHOCKWAVE,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Amplitude"), (1, "Width"), (2, "Speed")],
        slider_defaults: &[(0, 0.5), (1, 0.4), (2, 0.5)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Melt",
        description: "Gravity-driven melting and dripping deformation",
        source: MELT,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Amount"), (1, "Speed"), (2, "Drip")],
        slider_defaults: &[(0, 0.5), (1, 0.4), (2, 0.5)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Twist",
        description: "Twisting mesh geometry around the vertical axis",
        source: TWIST,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Amount"), (1, "Speed")],
        slider_defaults: &[(0, 0.5), (1, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Inflate",
        description: "Breathing inflation along surface normals",
        source: INFLATE,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Amount"), (1, "Speed"), (2, "Noise")],
        slider_defaults: &[(0, 0.4), (1, 0.5), (2, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
    ShaderPreset {
        name: "Spikes",
        description: "Noise-driven spike extrusion along normals",
        source: SPIKES,
        is_geometry: true,
        category: "Mesh Effects",
        slider_labels: &[(0, "Height"), (1, "Sharpness"), (2, "Speed")],
        slider_defaults: &[(0, 0.5), (1, 0.5), (2, 0.3)],
        buffer_a_source: None,
        buffer_b_source: None,
        buffer_c_source: None,
        buffer_d_source: None,
        common_source: None,
        channel_bindings: None,
    },
];

const PLASMA: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = uniforms.time;

    var color = vec3<f32>(0.0);
    let p = uv * 8.0 - vec2<f32>(4.0);

    var v = 0.0;
    v += sin(p.x + time);
    v += sin((p.y + time) * 0.5);
    v += sin((p.x + p.y + time) * 0.5);
    let cx = p.x + 0.5 * sin(time * 0.333);
    let cy = p.y + 0.5 * cos(time * 0.5);
    v += sin(sqrt(cx * cx + cy * cy + 1.0) + time);

    color.x = sin(v * 3.14159) * 0.5 + 0.5;
    color.y = sin(v * 3.14159 + 2.094) * 0.5 + 0.5;
    color.z = sin(v * 3.14159 + 4.188) * 0.5 + 0.5;

    return vec4<f32>(color, 1.0);
}
"
);

const MANDELBROT: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let zoom = 2.0 + 1.5 * sin(uniforms.time * 0.1);
    let center = vec2<f32>(-0.745, 0.186);

    var uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
    uv = uv / zoom + center;

    var z = vec2<f32>(0.0);
    let c = uv;
    var iteration = 0u;
    let max_iterations = 256u;

    for (var index = 0u; index < max_iterations; index++) {
        if dot(z, z) > 4.0 {
            break;
        }
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        iteration = index;
    }

    if dot(z, z) <= 4.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let smooth_iter = f32(iteration) + 1.0 - log2(log2(dot(z, z)));
    let t = smooth_iter / f32(max_iterations);

    let r = 0.5 + 0.5 * cos(6.28318 * (t + 0.0));
    let g = 0.5 + 0.5 * cos(6.28318 * (t + 0.33));
    let b = 0.5 + 0.5 * cos(6.28318 * (t + 0.67));

    return vec4<f32>(r, g, b, 1.0);
}
"
);

const RAY_MARCH: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn sdf_sphere(p: vec3<f32>, radius: f32) -> f32 {
    return length(p) - radius;
}

fn sdf_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdf_scene(p: vec3<f32>) -> f32 {
    let sphere_dist = sdf_sphere(p - vec3<f32>(0.0, 0.0, 0.0), 1.0);

    let angle = uniforms.time * 0.5;
    let box_pos = vec3<f32>(cos(angle) * 2.0, sin(uniforms.time * 0.7) * 0.5, sin(angle) * 2.0);
    let box_dist = sdf_box(p - box_pos, vec3<f32>(0.4));

    let plane_dist = p.y + 1.0;

    return min(min(sphere_dist, box_dist), plane_dist);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let epsilon = 0.001;
    let dx = vec3<f32>(epsilon, 0.0, 0.0);
    let dy = vec3<f32>(0.0, epsilon, 0.0);
    let dz = vec3<f32>(0.0, 0.0, epsilon);
    return normalize(vec3<f32>(
        sdf_scene(p + dx) - sdf_scene(p - dx),
        sdf_scene(p + dy) - sdf_scene(p - dy),
        sdf_scene(p + dz) - sdf_scene(p - dz),
    ));
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);

    let camera_pos = vec3<f32>(3.0 * sin(uniforms.time * 0.3), 2.0, 3.0 * cos(uniforms.time * 0.3));
    let look_at = vec3<f32>(0.0, 0.0, 0.0);
    let forward = normalize(look_at - camera_pos);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, forward);

    let ray_dir = normalize(forward + uv.x * right + uv.y * up);

    var total_dist = 0.0;
    var hit = false;
    var position = camera_pos;

    for (var step = 0u; step < 128u; step++) {
        let dist = sdf_scene(position);
        if dist < 0.001 {
            hit = true;
            break;
        }
        if total_dist > 50.0 {
            break;
        }
        total_dist += dist;
        position = camera_pos + ray_dir * total_dist;
    }

    if !hit {
        let sky = mix(vec3<f32>(0.1, 0.1, 0.2), vec3<f32>(0.3, 0.4, 0.7), in.uv.y);
        return vec4<f32>(sky, 1.0);
    }

    let normal = calc_normal(position);
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.5));
    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = 0.15;

    let view_dir = normalize(camera_pos - position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 32.0);

    let base_color = vec3<f32>(0.8, 0.3, 0.2);
    let color = base_color * (ambient + diffuse * 0.8) + vec3<f32>(1.0) * specular * 0.5;

    let fog = exp(-total_dist * 0.05);
    let final_color = mix(vec3<f32>(0.1, 0.1, 0.2), color, fog);

    return vec4<f32>(final_color, 1.0);
}
"
);

const VORONOI: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn hash2(p: vec2<f32>) -> vec2<f32> {
    let q = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(q) * 43758.5453);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = 6.0;
    let uv = in.uv * scale;
    let cell = floor(uv);
    let frac = fract(uv);

    var min_dist = 10.0;
    var min_dist2 = 10.0;
    var nearest_point = vec2<f32>(0.0);

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let neighbor = vec2<f32>(f32(x), f32(y));
            let point = hash2(cell + neighbor);
            let animated_point = 0.5 + 0.5 * sin(uniforms.time * 0.8 + point * 6.28318);
            let diff = neighbor + animated_point - frac;
            let dist = length(diff);

            if dist < min_dist {
                min_dist2 = min_dist;
                min_dist = dist;
                nearest_point = point;
            } else if dist < min_dist2 {
                min_dist2 = dist;
            }
        }
    }

    let edge = min_dist2 - min_dist;
    let cell_color = vec3<f32>(
        0.5 + 0.5 * sin(nearest_point.x * 12.0 + uniforms.time),
        0.5 + 0.5 * sin(nearest_point.y * 8.0 + uniforms.time * 1.3 + 2.0),
        0.5 + 0.5 * sin((nearest_point.x + nearest_point.y) * 10.0 + uniforms.time * 0.7 + 4.0),
    );

    let edge_glow = smoothstep(0.0, 0.05, edge);
    let color = cell_color * edge_glow;

    return vec4<f32>(color, 1.0);
}
"
);

const SDF_2D_SHAPES: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var radius = r;
    if p.x > 0.0 {
        radius = vec4<f32>(radius.z, radius.w, radius.x, radius.y);
    }
    if p.y > 0.0 {
        radius = vec4<f32>(radius.w, radius.z, radius.y, radius.x);
    }
    let q = abs(p) - b + radius.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius.x;
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn sd_equilateral_triangle(p: vec2<f32>, r: f32) -> f32 {
    let k = sqrt(3.0);
    var q = vec2<f32>(abs(p.x) - r, p.y + r / k);
    if q.x + k * q.y > 0.0 {
        q = vec2<f32>(q.x - k * q.y, -k * q.x - q.y) * 0.5;
    }
    q.x -= clamp(q.x, -2.0 * r, 0.0);
    return -length(q) * sign(q.y);
}

fn sd_pentagon(p: vec2<f32>, r: f32) -> f32 {
    let k = vec3<f32>(0.809016994, 0.587785252, 0.726542528);
    var q = vec2<f32>(abs(p.x), p.y);
    q -= 2.0 * min(dot(vec2<f32>(-k.x, k.y), q), 0.0) * vec2<f32>(-k.x, k.y);
    q -= 2.0 * min(dot(vec2<f32>(k.x, k.y), q), 0.0) * vec2<f32>(k.x, k.y);
    q -= vec2<f32>(clamp(q.x, -r * k.z, r * k.z), r);
    return length(q) * sign(q.y);
}

fn sd_hexagon(p: vec2<f32>, r: f32) -> f32 {
    let k = vec3<f32>(-0.866025404, 0.5, 0.577350269);
    var q = abs(p);
    q -= 2.0 * min(dot(k.xy, q), 0.0) * k.xy;
    q -= vec2<f32>(clamp(q.x, -k.z * r, k.z * r), r);
    return length(q) * sign(q.y);
}

fn sd_star5(p: vec2<f32>, r: f32, rf: f32) -> f32 {
    let k1 = vec2<f32>(0.809016994375, -0.587785252292);
    let k2 = vec2<f32>(-k1.x, k1.y);
    var q = vec2<f32>(abs(p.x), p.y);
    q -= 2.0 * max(dot(k1, q), 0.0) * k1;
    q -= 2.0 * max(dot(k2, q), 0.0) * k2;
    q.x = abs(q.x);
    q.y -= r;
    let ba = rf * vec2<f32>(-k1.y, k1.x) - vec2<f32>(0.0, 1.0);
    let h = clamp(dot(q, ba) / dot(ba, ba), 0.0, r);
    return length(q - ba * h) * sign(q.y * ba.x - q.x * ba.y);
}

fn sd_vesica(p: vec2<f32>, r: f32, d: f32) -> f32 {
    let q = abs(p);
    let b = sqrt(r * r - d * d);
    if (q.y - b) * d > q.x * b {
        return length(q - vec2<f32>(0.0, b));
    }
    return length(q - vec2<f32>(-d, 0.0)) - r;
}

fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn rot2(angle: f32) -> mat2x2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat2x2<f32>(c, s, -s, c);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0) * 5.0;
    let time = uniforms.time;

    var scene = 1e10;
    var closest_color = vec3<f32>(0.15, 0.15, 0.2);

    let p0 = uv - vec2<f32>(-3.5, 1.5);
    let d0 = sd_circle(p0, 0.45 + 0.1 * sin(time * 2.0));
    if d0 < scene { closest_color = vec3<f32>(0.9, 0.3, 0.2); }
    scene = min(scene, d0);

    let p1 = uv - vec2<f32>(-1.75, 1.5);
    let d1 = sd_box(rot2(time * 0.5) * p1, vec2<f32>(0.4, 0.3));
    if d1 < scene { closest_color = vec3<f32>(0.2, 0.7, 0.9); }
    scene = min(scene, d1);

    let p2 = uv - vec2<f32>(0.0, 1.5);
    let d2 = sd_rounded_box(p2, vec2<f32>(0.4, 0.3), vec4<f32>(0.15, 0.05, 0.2, 0.1));
    if d2 < scene { closest_color = vec3<f32>(0.9, 0.8, 0.2); }
    scene = min(scene, d2);

    let p3 = uv - vec2<f32>(1.75, 1.5);
    let d3 = sd_equilateral_triangle(rot2(time * 0.3) * p3, 0.5);
    if d3 < scene { closest_color = vec3<f32>(0.3, 0.9, 0.4); }
    scene = min(scene, d3);

    let p4 = uv - vec2<f32>(3.5, 1.5);
    let d4 = sd_pentagon(p4, 0.45);
    if d4 < scene { closest_color = vec3<f32>(0.8, 0.4, 0.9); }
    scene = min(scene, d4);

    let p5 = uv - vec2<f32>(-3.5, -0.5);
    let d5 = sd_hexagon(rot2(time * 0.2) * p5, 0.45);
    if d5 < scene { closest_color = vec3<f32>(0.9, 0.5, 0.3); }
    scene = min(scene, d5);

    let p6 = uv - vec2<f32>(-1.75, -0.5);
    let d6 = sd_star5(p6, 0.4, 0.45 + 0.05 * sin(time));
    if d6 < scene { closest_color = vec3<f32>(1.0, 0.85, 0.2); }
    scene = min(scene, d6);

    let p7 = uv - vec2<f32>(0.0, -0.5);
    let d7 = sd_segment(p7,
        vec2<f32>(-0.4, -0.3),
        vec2<f32>(0.4 * cos(time), 0.3 * sin(time))
    ) - 0.06;
    if d7 < scene { closest_color = vec3<f32>(0.4, 0.8, 1.0); }
    scene = min(scene, d7);

    let p8 = uv - vec2<f32>(1.75, -0.5);
    let d8 = sd_vesica(rot2(time * 0.4) * p8, 0.6, 0.3);
    if d8 < scene { closest_color = vec3<f32>(0.7, 0.3, 0.5); }
    scene = min(scene, d8);

    let merge_center = uv - vec2<f32>(3.5, -0.5);
    let dm1 = sd_circle(merge_center - vec2<f32>(0.2 * sin(time), 0.0), 0.3);
    let dm2 = sd_circle(merge_center + vec2<f32>(0.2 * sin(time), 0.0), 0.3);
    let d9 = op_smooth_union(dm1, dm2, 0.25);
    if d9 < scene { closest_color = vec3<f32>(0.2, 0.9, 0.7); }
    scene = min(scene, d9);

    var color = vec3<f32>(0.08, 0.08, 0.12);
    let border_glow = 1.0 / (abs(scene) * 80.0 + 1.0);
    color += closest_color * border_glow * 0.5;

    if scene < 0.0 {
        let interior_fade = smoothstep(0.0, -0.15, scene);
        color = mix(closest_color * 0.4, closest_color, interior_fade);
        let pattern = 0.5 + 0.5 * sin(scene * 40.0 + time * 3.0);
        color = mix(color, color * 1.3, pattern * 0.15);
    }

    let bands = 0.5 + 0.5 * cos(scene * 30.0);
    color += vec3<f32>(0.03) * bands * step(0.0, scene);

    return vec4<f32>(color, 1.0);
}
"
);

const IQ_RAYMARCHER: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_box3(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_round_box(p: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}

fn sd_torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
    let q = vec2<f32>(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

fn sd_capped_cylinder(p: vec3<f32>, h: f32, r: f32) -> f32 {
    let d = abs(vec2<f32>(length(p.xz), p.y)) - vec2<f32>(r, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)));
}

fn op_smooth_union3(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn op_smooth_subtraction(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

fn op_smooth_intersection(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}

fn op_rep_xz(p: vec3<f32>, spacing: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(
        p.x - spacing.x * round(p.x / spacing.x),
        p.y,
        p.z - spacing.y * round(p.z / spacing.y),
    );
}

fn rot_y(p: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(c * p.x + s * p.z, p.y, -s * p.x + c * p.z);
}

fn rot_x(p: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(p.x, c * p.y - s * p.z, s * p.y + c * p.z);
}

fn sdf_scene(p: vec3<f32>) -> vec2<f32> {
    let time = uniforms.time;

    let sphere_p = p - vec3<f32>(0.0, 0.6 + 0.3 * sin(time * 1.5), 0.0);
    let sphere = sd_sphere(sphere_p, 0.6);

    let box_p = rot_y(p - vec3<f32>(1.8, 0.5, 0.0), time * 0.7);
    let rounded_box = sd_round_box(box_p, vec3<f32>(0.35, 0.35, 0.35), 0.08);

    let torus_p = rot_x(p - vec3<f32>(-1.8, 0.45, 0.0), time * 0.6);
    let torus = sd_torus(torus_p, vec2<f32>(0.4, 0.15));

    let blob1 = sd_sphere(p - vec3<f32>(0.5 * sin(time), 1.2, 0.5 * cos(time)), 0.25);
    let blob2 = sd_sphere(p - vec3<f32>(-0.5 * cos(time * 1.3), 1.0, -0.5 * sin(time * 0.7)), 0.3);
    let blob3 = sd_sphere(p - vec3<f32>(0.0, 0.8 + 0.4 * sin(time * 0.9), 0.0), 0.4);
    var merged = op_smooth_union3(blob1, blob2, 0.4);
    merged = op_smooth_union3(merged, blob3, 0.3);

    let cyl_p = p - vec3<f32>(0.0, 0.0, -2.0);
    let cyl = sd_capped_cylinder(rot_y(cyl_p, time * 0.4), 0.6, 0.25);
    let cut_sphere = sd_sphere(cyl_p, 0.45);
    let carved = op_smooth_subtraction(cut_sphere, cyl, 0.1);

    let plane = p.y;

    var result = vec2<f32>(plane, 0.0);
    if sphere < result.x { result = vec2<f32>(sphere, 1.0); }
    if rounded_box < result.x { result = vec2<f32>(rounded_box, 2.0); }
    if torus < result.x { result = vec2<f32>(torus, 3.0); }
    if merged < result.x { result = vec2<f32>(merged, 4.0); }
    if carved < result.x { result = vec2<f32>(carved, 5.0); }

    return result;
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.0005, 0.0);
    return normalize(vec3<f32>(
        sdf_scene(p + e.xyy).x - sdf_scene(p - e.xyy).x,
        sdf_scene(p + e.yxy).x - sdf_scene(p - e.yxy).x,
        sdf_scene(p + e.yyx).x - sdf_scene(p - e.yyx).x,
    ));
}

fn calc_ao(pos: vec3<f32>, nor: vec3<f32>) -> f32 {
    var occ = 0.0;
    var sca = 1.0;
    for (var step = 0u; step < 5u; step++) {
        let h = 0.01 + 0.12 * f32(step) / 4.0;
        let d = sdf_scene(pos + h * nor).x;
        occ += (h - d) * sca;
        sca *= 0.95;
        if occ > 0.35 { break; }
    }
    return clamp(1.0 - 3.0 * occ, 0.0, 1.0) * (0.5 + 0.5 * nor.y);
}

fn calc_soft_shadow(ro: vec3<f32>, rd: vec3<f32>, mint: f32, tmax: f32) -> f32 {
    var res = 1.0;
    var t = mint;
    var ph = 1e20;
    for (var step = 0u; step < 32u; step++) {
        let h = sdf_scene(ro + rd * t).x;
        if h < 0.001 {
            return 0.0;
        }
        let y = h * h / (2.0 * ph);
        let d = sqrt(h * h - y * y);
        res = min(res, 10.0 * d / max(0.0, t - y));
        ph = h;
        t += h;
        if t > tmax { break; }
    }
    return clamp(res, 0.0, 1.0);
}

fn get_material_color(mat_id: f32) -> vec3<f32> {
    if mat_id < 0.5 { return vec3<f32>(0.45, 0.42, 0.4); }
    if mat_id < 1.5 { return vec3<f32>(0.8, 0.2, 0.15); }
    if mat_id < 2.5 { return vec3<f32>(0.15, 0.6, 0.85); }
    if mat_id < 3.5 { return vec3<f32>(0.9, 0.6, 0.1); }
    if mat_id < 4.5 { return vec3<f32>(0.4, 0.8, 0.4); }
    return vec3<f32>(0.7, 0.3, 0.7);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
    let time = uniforms.time;

    let camera_angle = time * 0.25;
    let camera_pos = vec3<f32>(
        6.0 * sin(camera_angle),
        2.5 + 0.5 * sin(time * 0.3),
        6.0 * cos(camera_angle),
    );
    let look_at = vec3<f32>(0.0, 0.5, 0.0);
    let forward = normalize(look_at - camera_pos);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, forward);
    let ray_dir = normalize(forward * 1.5 + uv.x * right + uv.y * up);

    var total_dist = 0.0;
    var material_id = -1.0;
    var hit = false;
    var position = camera_pos;

    for (var step = 0u; step < 128u; step++) {
        let result = sdf_scene(position);
        if result.x < 0.0005 {
            hit = true;
            material_id = result.y;
            break;
        }
        if total_dist > 40.0 { break; }
        total_dist += result.x;
        position = camera_pos + ray_dir * total_dist;
    }

    if !hit {
        let sky_grad = 0.5 + 0.5 * ray_dir.y;
        let sky = mix(vec3<f32>(0.5, 0.7, 0.9), vec3<f32>(0.1, 0.25, 0.55), sky_grad);
        let sun_dir = normalize(vec3<f32>(0.8, 0.4, 0.5));
        let sun = pow(max(dot(ray_dir, sun_dir), 0.0), 128.0);
        return vec4<f32>(sky + vec3<f32>(1.0, 0.9, 0.7) * sun * 2.0, 1.0);
    }

    let normal = calc_normal(position);
    let base_color = get_material_color(material_id);

    if material_id < 0.5 {
        let checker = step(0.0, sin(position.x * 3.14159 * 2.0) * sin(position.z * 3.14159 * 2.0));
        var floor_color = mix(vec3<f32>(0.35, 0.32, 0.3), vec3<f32>(0.55, 0.52, 0.5), checker);
        let light_dir = normalize(vec3<f32>(0.8, 0.4, 0.5));
        let diff = max(dot(normal, light_dir), 0.0);
        let shadow = calc_soft_shadow(position + normal * 0.002, light_dir, 0.02, 10.0);
        let ao = calc_ao(position, normal);
        floor_color *= (0.2 + 0.8 * diff * shadow) * ao;
        let fog = exp(-total_dist * 0.04);
        let final_color = mix(vec3<f32>(0.5, 0.7, 0.9) * 0.5, floor_color, fog);
        return vec4<f32>(final_color, 1.0);
    }

    let light_dir = normalize(vec3<f32>(0.8, 0.4, 0.5));
    let diffuse = max(dot(normal, light_dir), 0.0);
    let shadow = calc_soft_shadow(position + normal * 0.002, light_dir, 0.02, 10.0);
    let ao = calc_ao(position, normal);

    let view_dir = normalize(camera_pos - position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 64.0);

    let back_light = max(dot(normal, normalize(vec3<f32>(-0.5, 0.2, -0.3))), 0.0);

    var color = base_color * (0.15 * ao);
    color += base_color * diffuse * shadow * 0.8;
    color += vec3<f32>(1.0, 0.95, 0.85) * specular * shadow * 0.6;
    color += base_color * back_light * 0.15;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 4.0);
    color += vec3<f32>(0.3, 0.5, 0.7) * fresnel * 0.2 * ao;

    color *= ao;

    let fog = exp(-total_dist * 0.04);
    color = mix(vec3<f32>(0.5, 0.7, 0.9) * 0.5, color, fog);

    color = pow(color, vec3<f32>(0.4545));

    return vec4<f32>(color, 1.0);
}
"
);

const WARP_TUNNEL: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);

    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    let tunnel_u = 0.5 / (radius + 0.001) + uniforms.time * 2.0;
    let tunnel_v = angle / 3.14159 + uniforms.time * 0.1;

    let pattern1 = sin(tunnel_u * 4.0) * cos(tunnel_v * 6.0);
    let pattern2 = sin(tunnel_u * 8.0 + uniforms.time) * sin(tunnel_v * 4.0 - uniforms.time * 0.5);

    let intensity = 0.5 + 0.25 * pattern1 + 0.25 * pattern2;
    let glow = exp(-radius * 3.0);

    let r = intensity * (0.5 + 0.5 * sin(tunnel_u * 0.5));
    let g = intensity * (0.5 + 0.5 * sin(tunnel_u * 0.5 + 2.094));
    let b = intensity * (0.5 + 0.5 * sin(tunnel_u * 0.5 + 4.188));

    let color = vec3<f32>(r, g, b) * (0.3 + glow * 0.7);

    return vec4<f32>(color, 1.0);
}
"
);

const OCEAN_SURFACE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn wave(p: vec2<f32>, time: f32) -> f32 {
    var height = 0.0;
    var freq = 1.0;
    var amp = 0.5;
    for (var octave = 0u; octave < 5u; octave++) {
        height += sin(dot(p * freq, vec2<f32>(0.7, 0.3)) + time * (0.5 + f32(octave) * 0.2)) * amp;
        height += sin(dot(p * freq, vec2<f32>(-0.3, 0.8)) + time * (0.3 + f32(octave) * 0.15)) * amp * 0.7;
        freq *= 2.0;
        amp *= 0.5;
    }
    return height;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);

    let camera_height = 3.0;
    let look_down = 0.6;
    let ray_y = -look_down + uv.y;
    let ray_xz = normalize(vec2<f32>(uv.x, 1.0));

    if ray_y > -0.01 {
        let sky_t = ray_y * 2.0 + 0.3;
        let sky = mix(vec3<f32>(0.7, 0.8, 0.95), vec3<f32>(0.3, 0.5, 0.9), sky_t);
        let sun_dir = vec2<f32>(0.3, 0.8);
        let sun = pow(max(dot(normalize(vec2<f32>(uv.x, ray_y)), sun_dir), 0.0), 64.0);
        return vec4<f32>(sky + vec3<f32>(1.0, 0.9, 0.7) * sun, 1.0);
    }

    let t = -camera_height / ray_y;
    let hit_pos = vec2<f32>(uv.x, 1.0) * t;
    let world_pos = hit_pos * 0.3;

    let height = wave(world_pos, uniforms.time);

    let epsilon = 0.01;
    let dx = wave(world_pos + vec2<f32>(epsilon, 0.0), uniforms.time) - height;
    let dz = wave(world_pos + vec2<f32>(0.0, epsilon), uniforms.time) - height;
    let normal = normalize(vec3<f32>(-dx / epsilon, 1.0, -dz / epsilon));

    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let diffuse = max(dot(normal, light_dir), 0.0);

    let view_dir = vec3<f32>(0.0, 1.0, 0.0);
    let reflect_dir = reflect(-light_dir, normal);
    let specular = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);

    let deep_color = vec3<f32>(0.0, 0.05, 0.15);
    let shallow_color = vec3<f32>(0.0, 0.3, 0.4);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    let water_color = mix(deep_color, shallow_color, 0.3 + 0.7 * fresnel);

    let color = water_color * (0.3 + diffuse * 0.5) + vec3<f32>(1.0, 0.95, 0.8) * specular * 0.8;

    let fog = exp(-length(hit_pos) * 0.01);
    let final_color = mix(vec3<f32>(0.5, 0.6, 0.7), color, fog);

    return vec4<f32>(final_color, 1.0);
}
"
);

const FRACTAL_NOISE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let frac = fract(p);
    let u = frac * frac * (3.0 - 2.0 * frac);

    let a = hash(cell);
    let b = hash(cell + vec2<f32>(1.0, 0.0));
    let c = hash(cell + vec2<f32>(0.0, 1.0));
    let d = hash(cell + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    for (var octave = 0u; octave < 6u; octave++) {
        value += amplitude * noise(pos * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
        pos += vec2<f32>(1.7, 9.2);
    }
    return value;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 4.0;
    let time = uniforms.time * 0.2;

    let warp = vec2<f32>(
        fbm(uv + vec2<f32>(time, 0.0)),
        fbm(uv + vec2<f32>(0.0, time)),
    );

    let value = fbm(uv + warp * 2.0);

    let color1 = vec3<f32>(0.1, 0.0, 0.2);
    let color2 = vec3<f32>(0.0, 0.3, 0.5);
    let color3 = vec3<f32>(0.9, 0.5, 0.1);
    let color4 = vec3<f32>(1.0, 0.9, 0.8);

    var color: vec3<f32>;
    if value < 0.33 {
        color = mix(color1, color2, value / 0.33);
    } else if value < 0.66 {
        color = mix(color2, color3, (value - 0.33) / 0.33);
    } else {
        color = mix(color3, color4, (value - 0.66) / 0.34);
    }

    return vec4<f32>(color, 1.0);
}
"
);

const KALEIDOSCOPE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var p = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);

    let segments = 8.0;
    var angle = atan2(p.y, p.x);
    let radius = length(p);

    angle = ((angle / 3.14159 * 0.5 + 0.5) * segments) % 1.0;
    if angle > 0.5 {
        angle = 1.0 - angle;
    }
    angle = (angle - 0.5) * 2.0 * 3.14159 / segments;

    p = vec2<f32>(cos(angle), sin(angle)) * radius;
    p += vec2<f32>(uniforms.time * 0.1, uniforms.time * 0.07);

    let scale = 3.0;
    let pattern = sin(p.x * scale * 10.0) * cos(p.y * scale * 10.0 + uniforms.time);
    let pattern2 = sin(length(p * scale) * 8.0 - uniforms.time * 2.0);
    let pattern3 = cos(p.x * scale * 5.0 + p.y * scale * 7.0 + uniforms.time * 0.5);

    let r = 0.5 + 0.5 * sin(pattern * 2.0 + uniforms.time);
    let g = 0.5 + 0.5 * sin(pattern2 * 2.0 + uniforms.time * 1.3 + 2.0);
    let b = 0.5 + 0.5 * sin(pattern3 * 2.0 + uniforms.time * 0.7 + 4.0);

    let brightness = 0.7 + 0.3 * sin(radius * 10.0 - uniforms.time * 3.0);
    let color = vec3<f32>(r, g, b) * brightness;

    return vec4<f32>(color, 1.0);
}
"
);

const LIT_MESH: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = uniforms.custom[0].xyz;

    let light_pos = vec3<f32>(3.0, 4.0, 2.0);
    let light_dir = normalize(light_pos - in.world_position);
    let normal = normalize(in.world_normal);

    let ambient = 0.15;
    let diffuse = max(dot(normal, light_dir), 0.0);

    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 32.0);

    let color = base_color * (ambient + diffuse * 0.7) + vec3<f32>(1.0) * specular * 0.4;

    return vec4<f32>(color, 1.0);
}
"
);

const NORMAL_MAP: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let color = normal * 0.5 + 0.5;
    return vec4<f32>(color, 1.0);
}
"
);

const UV_CHECKER: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let checker_scale = 10.0;
    let checker = floor(in.uv.x * checker_scale) + floor(in.uv.y * checker_scale);
    let is_white = (checker % 2.0) == 0.0;

    let color1 = vec3<f32>(0.9, 0.9, 0.9);
    let color2 = vec3<f32>(0.2, 0.2, 0.2);
    let checker_color = select(color2, color1, is_white);

    let uv_color = vec3<f32>(in.uv.x, in.uv.y, 0.5);
    let color = mix(checker_color, uv_color, 0.3);

    let normal = normalize(in.world_normal);
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.5 + 0.5;

    return vec4<f32>(color * diffuse, 1.0);
}
"
);

const WIREFRAME_GLOW: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let edge_width = 0.02;
    let glow_width = 0.08;

    let uv_frac = fract(in.uv * 10.0);
    let dist_to_edge = min(min(uv_frac.x, 1.0 - uv_frac.x), min(uv_frac.y, 1.0 - uv_frac.y));

    let edge = 1.0 - smoothstep(0.0, edge_width, dist_to_edge);
    let glow = 1.0 - smoothstep(edge_width, glow_width, dist_to_edge);

    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);

    let wire_color = vec3<f32>(0.0, 0.8, 1.0);
    let glow_color = wire_color * 0.4;
    let surface_color = vec3<f32>(0.02, 0.02, 0.05);

    let color = surface_color + wire_color * edge + glow_color * glow + wire_color * fresnel * 0.3;

    let pulse = 0.8 + 0.2 * sin(uniforms.time * 2.0 + in.world_position.y * 3.0);
    return vec4<f32>(color * pulse, 1.0);
}
"
);

const DISSOLVE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
fn hash_dissolve(p: vec3<f32>) -> f32 {
    var q = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    q += dot(q, q.yzx + 19.19);
    return fract((q.x + q.y) * q.z);
}

fn noise_dissolve(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let frac = fract(p);
    let u = frac * frac * (3.0 - 2.0 * frac);

    let n000 = hash_dissolve(cell);
    let n100 = hash_dissolve(cell + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash_dissolve(cell + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash_dissolve(cell + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash_dissolve(cell + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash_dissolve(cell + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash_dissolve(cell + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash_dissolve(cell + vec3<f32>(1.0, 1.0, 1.0));

    let x0 = mix(n000, n100, u.x);
    let x1 = mix(n010, n110, u.x);
    let x2 = mix(n001, n101, u.x);
    let x3 = mix(n011, n111, u.x);

    let y0 = mix(x0, x1, u.y);
    let y1 = mix(x2, x3, u.y);

    return mix(y0, y1, u.z);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dissolve_threshold = (sin(uniforms.time * 0.5) * 0.5 + 0.5);

    let noise_value = noise_dissolve(in.world_position * 4.0);

    if noise_value < dissolve_threshold {
        discard;
    }

    let edge_width = 0.05;
    let edge_distance = noise_value - dissolve_threshold;
    let edge_factor = 1.0 - smoothstep(0.0, edge_width, edge_distance);

    let normal = normalize(in.world_normal);
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0);

    let base_color = vec3<f32>(0.6, 0.6, 0.7) * (0.2 + diffuse * 0.8);
    let edge_color = vec3<f32>(2.0, 0.5, 0.0);

    let color = mix(base_color, edge_color, edge_factor);

    return vec4<f32>(color, 1.0);
}
"
);

const HOLOGRAM: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

fn holo_hash(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let jitter_speed = 3.0;
    let jitter = sin(uniforms.time * jitter_speed + pos.y * 10.0) * 0.015;
    pos.x += jitter;
    pos.z += jitter * 0.7;
    pos.y += sin(uniforms.time * 0.5) * 0.03;

    let time_slot = floor(uniforms.time * 8.0);
    let glitch_band = floor(pos.y * 15.0);
    let glitch_seed = holo_hash(glitch_band + time_slot * 0.37);
    if glitch_seed > 0.85 {
        pos.x += (holo_hash(time_slot + glitch_band * 13.7) - 0.5) * 0.15;
    }

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.0);

    let scanline_freq = 80.0;
    let scanline = sin(in.world_position.y * scanline_freq + uniforms.time * 5.0) * 0.5 + 0.5;
    let scanline_mask = smoothstep(0.3, 0.7, scanline);

    let fine_lines = sin(in.world_position.y * 300.0) * 0.5 + 0.5;
    let fine_mask = smoothstep(0.2, 0.8, fine_lines);

    let flicker = 0.85 + 0.15 * sin(uniforms.time * 30.0);
    let time_slot = floor(uniforms.time * 8.0);
    let glitch_band = floor(in.world_position.y * 15.0);
    let glitch_line = step(0.85, holo_hash(glitch_band + time_slot * 0.37));

    let holo_color = vec3<f32>(0.1, 0.6, 1.0);
    let edge_color = vec3<f32>(0.3, 0.9, 1.0);

    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.3 + 0.2;

    var color = holo_color * diffuse;
    color += edge_color * fresnel * 1.5;
    color *= scanline_mask * 0.6 + 0.4;
    color *= fine_mask * 0.3 + 0.7;
    color += vec3<f32>(0.0, glitch_line * 0.6, glitch_line * 0.3);
    color *= flicker;

    let alpha = (fresnel * 0.6 + 0.25) * scanline_mask * flicker;

    return vec4<f32>(color, alpha);
}
"
);

const TUTORIAL_HELLO_COLOR: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 1: Hello Color
// =======================
// The fragment shader runs once per pixel on screen.
// It returns vec4<f32>(red, green, blue, alpha).
// Each component ranges from 0.0 (none) to 1.0 (full).
//
// TRY: Change the color values and watch the preview update!
//      What happens if alpha is 0.5?

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.6, 1.0, 1.0);
}
"
);

const TUTORIAL_UV_GRADIENT: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 2: UV Coordinates
// ==========================
// 'in.uv' gives this pixel's position, normalized to 0..1
//   uv.x = 0 at left,   1 at right
//   uv.y = 0 at bottom, 1 at top
//
// Here we map position directly to color:
//   Red channel   = horizontal position
//   Green channel = vertical position
//
// TRY: Swap .x and .y, multiply UVs, or use 1.0 - in.uv.x

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let red = in.uv.x;
    let green = in.uv.y;
    let blue = 0.5;
    return vec4<f32>(red, green, blue, 1.0);
}
"
);

const TUTORIAL_ANIMATION: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 3: Animation with Time
// ================================
// uniforms.time increases every second (try Pause/Reset/Speed)
// uniforms.delta_time = seconds since last frame
// uniforms.frame = frame counter
//
// sin() oscillates -1..1. Use '* 0.5 + 0.5' to remap to 0..1.
// Multiply time by different speeds for varied animation.
// Add offsets (like + 2.094) to shift phase between channels.
//
// TRY: Change speed multipliers, use cos(), add more patterns

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time;

    let r = sin(time * 1.0) * 0.5 + 0.5;
    let g = sin(time * 1.3 + 2.094) * 0.5 + 0.5;
    let b = sin(time * 0.7 + 4.188) * 0.5 + 0.5;

    let center = in.uv - 0.5;
    let dist = length(center);
    let pulse = sin(dist * 20.0 - time * 3.0) * 0.5 + 0.5;

    let color = vec3<f32>(r, g, b) * (0.5 + 0.5 * pulse);

    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_MOUSE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 4: Mouse & Aspect Ratio
// =================================
// uniforms.mouse = cursor position, normalized to 0..1
// uniforms.resolution = viewport size in pixels
//
// Problem: if you use raw UVs, circles become ovals on
// non-square viewports. Fix: scale x by aspect ratio.
//
// distance(a, b) = length of the vector between two points
// smoothstep(lo, hi, x) = smooth 0..1 transition
//
// TRY: Move your mouse over the preview area!

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
    let mouse = (uniforms.mouse - 0.5) * vec2<f32>(aspect, 1.0);

    let dist = distance(uv, mouse);

    let inner = smoothstep(0.12, 0.11, dist);
    let ring = smoothstep(0.16, 0.15, dist) - smoothstep(0.13, 0.12, dist);
    let glow = 0.02 / (dist + 0.01);

    let bg = vec3<f32>(0.05, 0.05, 0.1);
    let color = bg
        + vec3<f32>(1.0, 0.4, 0.1) * inner
        + vec3<f32>(1.0, 0.6, 0.2) * ring
        + vec3<f32>(0.3, 0.1, 0.0) * glow;

    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_SLIDERS: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 5: Custom Slider Controls
// ====================================
// The Uniforms panel has 16 sliders mapped to:
//   uniforms.custom[0].xyzw = sliders 0-3  (Color R/G/B/A)
//   uniforms.custom[1].xyzw = sliders 4-7  (Custom 4-7)
//   uniforms.custom[2].xyzw = sliders 8-11
//   uniforms.custom[3].xyzw = sliders 12-15
//
// TIP: Right-click any slider to change its range!
//      Click 'Show all 16 sliders' to reveal more.
//
// TRY: Adjust Color R/G/B and try Custom 4/5 sliders
//      (set Custom 4 range to 0..30 via right-click)

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let user_color = uniforms.custom[0].xyz;
    let ring_density = uniforms.custom[1].x * 30.0 + 5.0;
    let speed = uniforms.custom[1].y * 5.0 + 1.0;

    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
    let dist = length(uv);

    let rings = sin(dist * ring_density - uniforms.time * speed) * 0.5 + 0.5;
    let fade = smoothstep(0.8, 0.0, dist);

    let color = user_color * rings * fade;

    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_PATTERNS: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 6: Patterns & Shader Math
// ====================================
// Essential shader functions:
//   floor(x)           - rounds down (creates grid cells)
//   fract(x)           - fractional part (repeats 0..1)
//   step(edge, x)      - 0 if x<edge, 1 otherwise
//   smoothstep(a, b, x)- smooth 0..1 ramp between a and b
//   mix(a, b, t)       - linear blend: a*(1-t) + b*t
//   length(v)          - distance from origin
//   mod / %            - remainder (creates repeating patterns)
//
// TRY: Change checker_scale, combine patterns differently

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let checker_scale = 8.0;
    let cell = floor(uv * checker_scale);
    let checker = (cell.x + cell.y) % 2.0;

    let repeat_uv = fract(uv * checker_scale) - 0.5;
    let dots = smoothstep(0.3, 0.28, length(repeat_uv));

    let wave = sin((uv.x + uv.y) * 20.0 - uniforms.time * 2.0) * 0.5 + 0.5;

    let dark = vec3<f32>(0.1, 0.1, 0.15);
    let light = vec3<f32>(0.2, 0.2, 0.3);
    let highlight = vec3<f32>(0.0, 0.8, 1.0);

    var color = mix(dark, light, checker);
    color = mix(color, highlight, dots * 0.6);
    color += vec3<f32>(0.1, 0.05, 0.0) * wave;

    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_TEXTURES: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
// TUTORIAL 7: Texture Sampling
// ==============================
// Drag and drop an image file (PNG, JPG, BMP, TGA)
// onto the window to load it into a texture slot.
//
// 4 slots available in the Textures panel:
//   texture_0 / sampler_0  ..  texture_3 / sampler_3
//
// textureSample(texture, sampler, uv) returns vec4 (RGBA)
// The sampler uses linear filtering and repeat wrapping,
// so UVs outside 0..1 tile the texture seamlessly.
//
// Without a texture loaded, you'll see a magenta placeholder.
//
// TRY: Drop an image, then try scrolling/scaling the UVs!

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(texture_0, sampler_0, in.uv);

    let scroll_uv = in.uv * 2.0 + vec2<f32>(uniforms.time * 0.1, 0.0);
    let tex2 = textureSample(texture_1, sampler_1, scroll_uv);

    let center = in.uv - 0.5;
    let vignette = 1.0 - dot(center, center) * 1.5;

    let color = mix(tex.rgb, tex2.rgb, 0.3) * max(vignette, 0.0);

    return vec4<f32>(color, tex.a);
}
"
);

const TUTORIAL_GEOMETRY_INTRO: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
// TUTORIAL 8: 3D Geometry Mode
// ==============================
// This shader has a VertexInput struct with @location attributes,
// so the mode auto-switches to 3D Geometry on compile.
//
// vertex_main receives per-vertex: position, normal, uv
// It transforms vertices using model/view/projection matrices.
// The model matrix auto-rotates the mesh over time.
//
// fragment_main receives interpolated values:
//   in.world_position - vertex position in world space
//   in.world_normal   - surface normal direction
//   in.uv             - texture coordinates
//
// Drag the viewport to orbit the camera!
// Switch primitives in the Geometry panel (Cube, Sphere, Torus...)
//
// TRY: Change the visualization or mix different attributes!

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);

    let normal_color = normal * 0.5 + 0.5;

    let up_light = dot(normal, vec3<f32>(0.0, 1.0, 0.0)) * 0.3 + 0.7;

    let color = normal_color * up_light;
    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_LIGHTING: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
// TUTORIAL 9: Blinn-Phong Lighting
// ==================================
// The classic real-time lighting model has three parts:
//
// AMBIENT:  Constant base light so nothing is pure black
// DIFFUSE:  Brightness from how directly a surface faces the light
//           Uses: dot(normal, light_direction)
// SPECULAR: Shiny highlight where light bounces toward the camera
//           Uses the 'half vector' between light and view dirs
//           pow() controls shininess (higher = tighter highlight)
//
// Color R/G/B sliders control the material color!
//
// TRY: Move the light_pos, change the shininess exponent,
//      or try different geometry primitives.

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);

    let light_pos = vec3<f32>(3.0, 4.0, 2.0);
    let light_dir = normalize(light_pos - in.world_position);
    let light_color = vec3<f32>(1.0, 0.95, 0.9);

    let ambient = 0.08;

    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let diffuse = n_dot_l;

    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(normal, half_dir), 0.0);
    let specular = pow(n_dot_h, 64.0);

    let base_color = uniforms.custom[0].xyz;

    let color = base_color * (ambient + diffuse * 0.8) * light_color
              + light_color * specular * 0.5;

    return vec4<f32>(color, 1.0);
}
"
);

const TUTORIAL_EVERYTHING: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
// TUTORIAL 10: Putting It All Together
// ======================================
// This shader uses every Shader Studio feature:
//   - uniforms.time for animation (orbiting light)
//   - uniforms.mouse for interaction (light Y follows mouse)
//   - uniforms.custom for sliders (color, emission, rim)
//   - textureSample for surface detail (drag & drop an image!)
//   - 3D lighting with fresnel rim glow
//
// Sliders:
//   0-2 (Color R/G/B): Base material color
//   3   (Color A):     Emission pulse intensity
//   4   (Custom 4):    Rim light intensity
//
// TRY: Drop a texture, change geometry, adjust all the sliders!
//      Right-click Custom 4 slider and set range to 0..2.

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let time = uniforms.time;

    let light_y = 2.0 + uniforms.mouse.y * 4.0;
    let light_pos = vec3<f32>(
        3.0 * cos(time * 0.7),
        light_y,
        3.0 * sin(time * 0.7),
    );
    let light_dir = normalize(light_pos - in.world_position);

    let diffuse = max(dot(normal, light_dir), 0.0);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 32.0);

    let base = uniforms.custom[0].xyz;

    let tex = textureSample(texture_0, sampler_0, in.uv);
    let surface_color = base * (0.6 + 0.4 * tex.rgb);

    let emission = uniforms.custom[0].w;
    let pulse = sin(in.world_position.y * 8.0 + time * 3.0) * 0.5 + 0.5;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    let rim_intensity = uniforms.custom[1].x;

    var color = surface_color * (0.1 + diffuse * 0.7);
    color += vec3<f32>(1.0, 0.95, 0.9) * specular * 0.4;
    color += surface_color * emission * pulse * 0.5;
    color += vec3<f32>(0.3, 0.5, 1.0) * fresnel * (0.2 + rim_intensity);

    return vec4<f32>(color, 1.0);
}
"
);

const GAME_OF_LIFE_BUFFER_A: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.uv * uniforms.resolution);
    let res = vec2<i32>(uniforms.resolution);

    if uniforms.frame < 2u {
        let hash = fract(sin(dot(vec2<f32>(pixel), vec2<f32>(127.1, 311.7))) * 43758.5453);
        let alive = step(0.6, hash);
        return vec4<f32>(alive, alive, alive, 1.0);
    }

    let prev = textureSample(texture_0, sampler_0, in.uv);
    let current_state = step(0.5, prev.r);

    var neighbors = 0.0;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            if dx == 0 && dy == 0 { continue; }
            let neighbor_uv = (vec2<f32>(pixel) + vec2<f32>(f32(dx), f32(dy)) + 0.5) / uniforms.resolution;
            let neighbor = textureSample(texture_0, sampler_0, neighbor_uv);
            neighbors += step(0.5, neighbor.r);
        }
    }

    var next_state = 0.0;
    if current_state > 0.5 {
        if neighbors >= 2.0 && neighbors <= 3.0 {
            next_state = 1.0;
        }
    } else {
        if neighbors >= 2.5 && neighbors <= 3.5 {
            next_state = 1.0;
        }
    }

    return vec4<f32>(next_state, next_state, next_state, 1.0);
}
"
);

const GAME_OF_LIFE_IMAGE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cell = textureSample(texture_0, sampler_0, in.uv);
    let alive = step(0.5, cell.r);

    let color_alive = vec3<f32>(0.1, 0.8, 0.3);
    let color_dead = vec3<f32>(0.02, 0.02, 0.05);
    let color = mix(color_dead, color_alive, alive);

    return vec4<f32>(color, 1.0);
}
"
);

const GAME_OF_LIFE_BINDINGS: [[ChannelSource; 4]; 5] = [
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
];

const FEEDBACK_BLUR_BUFFER_A: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let decay = mix(0.85, 0.995, uniforms.custom[0].x);
    let blur_scale = 1.0 + uniforms.custom[0].y * 4.0;
    let blur_size = blur_scale / uniforms.resolution;

    var blurred = vec4<f32>(0.0);
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(-blur_size.x, -blur_size.y));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(0.0, -blur_size.y));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(blur_size.x, -blur_size.y));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(-blur_size.x, 0.0));
    blurred += textureSample(texture_0, sampler_0, in.uv);
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(blur_size.x, 0.0));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(-blur_size.x, blur_size.y));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(0.0, blur_size.y));
    blurred += textureSample(texture_0, sampler_0, in.uv + vec2<f32>(blur_size.x, blur_size.y));
    blurred /= 9.0;

    let previous = blurred * decay;

    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (in.uv - 0.5) * vec2<f32>(aspect, 1.0);
    let time = uniforms.time;

    let ball_pos = vec2<f32>(0.3 * cos(time * 1.5), 0.3 * sin(time * 2.0));
    let dist = length(uv - ball_pos);
    let ball = smoothstep(0.06, 0.04, dist);

    let ball2_pos = vec2<f32>(-0.2 * sin(time * 0.8), 0.25 * cos(time * 1.2));
    let dist2 = length(uv - ball2_pos);
    let ball2 = smoothstep(0.04, 0.02, dist2);

    let fresh_r = ball * (0.5 + 0.5 * sin(time));
    let fresh_g = ball * (0.5 + 0.5 * sin(time + 2.0));
    let fresh_b = ball * (0.5 + 0.5 * sin(time + 4.0));
    let fresh = vec4<f32>(fresh_r, fresh_g, fresh_b, 1.0)
              + vec4<f32>(0.0, ball2 * 0.8, ball2, 1.0) * ball2;

    return max(previous, fresh);
}
"
);

const FEEDBACK_BLUR_IMAGE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let buffer = textureSample(texture_0, sampler_0, in.uv);
    let color = pow(buffer.rgb, vec3<f32>(0.8));
    return vec4<f32>(color, 1.0);
}
"
);

const FEEDBACK_BLUR_BINDINGS: [[ChannelSource; 4]; 5] = [
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
];

const REACTION_DIFFUSION_BUFFER_A: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_size = 1.0 / uniforms.resolution;

    if uniforms.frame < 2u {
        let dist = length(in.uv - 0.5);
        if dist < 0.05 {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }
        let hash = fract(sin(dot(floor(in.uv * 100.0), vec2<f32>(127.1, 311.7))) * 43758.5453);
        if hash > 0.97 {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let center = textureSample(texture_0, sampler_0, in.uv);
    let a_val = center.r;
    let b_val = center.g;

    let left = textureSample(texture_0, sampler_0, in.uv + vec2<f32>(-pixel_size.x, 0.0));
    let right = textureSample(texture_0, sampler_0, in.uv + vec2<f32>(pixel_size.x, 0.0));
    let up = textureSample(texture_0, sampler_0, in.uv + vec2<f32>(0.0, pixel_size.y));
    let down = textureSample(texture_0, sampler_0, in.uv + vec2<f32>(0.0, -pixel_size.y));

    let laplacian_a = (left.r + right.r + up.r + down.r) - 4.0 * a_val;
    let laplacian_b = (left.g + right.g + up.g + down.g) - 4.0 * b_val;

    let feed = uniforms.custom[0].x * 0.08 + 0.01;
    let kill = uniforms.custom[0].y * 0.04 + 0.04;
    let diffusion_a = 1.0;
    let diffusion_b = 0.5;
    let dt = 1.0;

    let ab_squared = a_val * b_val * b_val;
    let new_a = a_val + (diffusion_a * laplacian_a - ab_squared + feed * (1.0 - a_val)) * dt;
    let new_b = b_val + (diffusion_b * laplacian_b + ab_squared - (kill + feed) * b_val) * dt;

    return vec4<f32>(clamp(new_a, 0.0, 1.0), clamp(new_b, 0.0, 1.0), 0.0, 1.0);
}
"
);

const REACTION_DIFFUSION_IMAGE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    fullscreen_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let data = textureSample(texture_0, sampler_0, in.uv);
    let a_val = data.r;
    let b_val = data.g;

    let color1 = vec3<f32>(0.05, 0.05, 0.15);
    let color2 = vec3<f32>(0.1, 0.4, 0.8);
    let color3 = vec3<f32>(0.9, 0.6, 0.1);
    let color4 = vec3<f32>(1.0, 0.95, 0.9);

    let t = 1.0 - a_val;
    var color: vec3<f32>;
    if t < 0.33 {
        color = mix(color1, color2, t / 0.33);
    } else if t < 0.66 {
        color = mix(color2, color3, (t - 0.33) / 0.33);
    } else {
        color = mix(color3, color4, (t - 0.66) / 0.34);
    }

    return vec4<f32>(color, 1.0);
}
"
);

const REACTION_DIFFUSION_BINDINGS: [[ChannelSource; 4]; 5] = [
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [
        ChannelSource::BufferA,
        ChannelSource::None,
        ChannelSource::None,
        ChannelSource::None,
    ],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
    [ChannelSource::None; 4],
];

const TOON_SHADING: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
    let view_dir = normalize(uniforms.camera_position - in.world_position);

    let base_color = vec3<f32>(
        uniforms.custom[0].x,
        uniforms.custom[0].y,
        uniforms.custom[0].z
    );
    let band_count = max(uniforms.custom[0].w * 8.0, 2.0);

    let ndotl = dot(normal, light_dir);
    let quantized = ceil(ndotl * band_count) / band_count;
    let diffuse = max(quantized, 0.15);

    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 32.0);
    let spec_band = step(0.5, spec);

    let color = base_color * diffuse + vec3<f32>(1.0) * spec_band * 0.5;

    let edge = 1.0 - smoothstep(0.0, 0.35, abs(dot(normal, view_dir)));

    let final_color = mix(color, vec3<f32>(0.02), edge);
    return vec4<f32>(final_color, 1.0);
}
"
);

const FRESNEL_GLOW: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let glow_color = vec3<f32>(
        uniforms.custom[0].x,
        uniforms.custom[0].y,
        uniforms.custom[0].z
    );
    let power = 1.0 + uniforms.custom[0].w * 5.0;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), power);
    let pulse = 0.7 + 0.3 * sin(uniforms.time * 2.0);

    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.15 + 0.05;
    let base = vec3<f32>(0.02, 0.02, 0.03);

    let color = base * diffuse + glow_color * fresnel * pulse * 3.0;
    return vec4<f32>(color, 1.0);
}
"
);

const FORCE_FIELD: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
fn hex_dist(p: vec2<f32>) -> f32 {
    let q = abs(p);
    return max(q.x * 0.866 + q.y * 0.5, q.y);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let shield_color = vec3<f32>(
        uniforms.custom[0].x,
        uniforms.custom[0].y,
        uniforms.custom[0].z
    );
    let hex_scale = 5.0 + uniforms.custom[0].w * 20.0;

    let uv = in.uv * hex_scale;
    let col = floor(uv.x / 1.732);
    let row = floor(uv.y / 1.5);
    let offset_x = select(0.0, 0.866, (i32(row) & 1) == 1);
    let hex_uv = vec2<f32>(fract(uv.x / 1.732 + offset_x / 1.732) - 0.5, fract(uv.y / 1.5) - 0.5);
    let hex_edge = 1.0 - smoothstep(0.35, 0.45, hex_dist(hex_uv));

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.0);

    let wave = 0.5 + 0.5 * sin(in.world_position.y * 3.0 - uniforms.time * 4.0);
    let pulse = 0.5 + 0.5 * sin(uniforms.time * 3.0);

    let glow = (hex_edge * 0.6 + fresnel * 0.8) * (0.5 + 0.5 * wave);
    let alpha = glow * (0.3 + 0.7 * pulse);

    let color = shield_color * glow * 2.0 + vec3<f32>(1.0) * pow(fresnel, 4.0) * 0.5;
    return vec4<f32>(color, alpha);
}
"
);

const XRAY: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let xray_color = vec3<f32>(
        uniforms.custom[0].x,
        uniforms.custom[0].y,
        uniforms.custom[0].z
    );
    let base_opacity = uniforms.custom[0].w;

    let edge = 1.0 - abs(dot(normal, view_dir));
    let rim = pow(edge, 1.5);

    let scan = 0.5 + 0.5 * sin(in.world_position.y * 20.0 + uniforms.time * 2.0);
    let detail = 0.9 + 0.1 * scan;

    let alpha = (rim * 0.8 + base_opacity * 0.4) * detail;
    let color = xray_color * (rim + 0.2) * detail;

    return vec4<f32>(color, alpha);
}
"
);

const LAVA: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
fn lava_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn lava_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(lava_hash(i), lava_hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(lava_hash(i + vec2<f32>(0.0, 1.0)), lava_hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

fn lava_fbm(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var oct = 0; oct < 5; oct++) {
        v += a * lava_noise(q);
        q *= 2.0;
        a *= 0.5;
    }
    return v;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let heat = uniforms.custom[0].x;
    let flow_speed = uniforms.custom[0].y * 2.0;
    let crack_width = uniforms.custom[0].z;

    let uv = in.uv * 4.0;
    let flow = lava_fbm(uv + vec2<f32>(uniforms.time * flow_speed * 0.2, uniforms.time * flow_speed * 0.1));
    let cracks = lava_fbm(uv * 3.0 + vec2<f32>(flow * 0.5));

    let is_crack = smoothstep(crack_width, crack_width + 0.1, cracks);

    let hot = vec3<f32>(1.0, 0.8, 0.0) * (1.0 + heat);
    let cool_color = vec3<f32>(0.15, 0.02, 0.0);
    let glow = vec3<f32>(1.0, 0.3, 0.0) * flow * heat * 2.0;

    let diffuse = max(dot(normal, light_dir), 0.0) * 0.3 + 0.1;
    let color = mix(hot + glow, cool_color * diffuse, is_crack);

    return vec4<f32>(color, 1.0);
}
"
);

const ICE_CRYSTAL: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
fn ice_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn ice_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(ice_hash(i), ice_hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(ice_hash(i + vec2<f32>(0.0, 1.0)), ice_hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
    let frost = uniforms.custom[0].x;
    let sparkle = uniforms.custom[0].y;
    let tint = uniforms.custom[0].z;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);

    let uv = in.uv * 12.0;
    let crack_noise = ice_noise(uv) * 0.5 + ice_noise(uv * 3.0) * 0.3 + ice_noise(uv * 7.0) * 0.2;
    let cracks = smoothstep(0.4, 0.6, crack_noise) * frost;

    let sparkle_noise = ice_noise(in.uv * 50.0 + uniforms.time * 0.5);
    let sparkles = pow(sparkle_noise, 20.0) * sparkle * 5.0;

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 64.0);

    let ice_blue = vec3<f32>(0.6, 0.8, 1.0);
    let deep_blue = vec3<f32>(0.1, 0.2, 0.5);
    let base = mix(deep_blue, ice_blue, tint);

    let color = base * (diffuse * 0.5 + 0.3) + vec3<f32>(1.0) * spec * 0.8 + vec3<f32>(1.0) * fresnel * 0.4 + ice_blue * cracks * 0.3 + vec3<f32>(1.0) * sparkles;

    return vec4<f32>(color, 1.0);
}
"
);

const MATCAP: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    geometry_vertex!(),
    r"
@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);

    let warm_amount = uniforms.custom[0].x;
    let cool_amount = uniforms.custom[0].y;
    let metallic = uniforms.custom[0].z;

    let view_normal = normalize(
        (uniforms.view * vec4<f32>(normal, 0.0)).xyz
    );

    let matcap_uv = view_normal.xy * 0.5 + 0.5;

    let warm = vec3<f32>(0.9, 0.6, 0.3);
    let cool_color = vec3<f32>(0.2, 0.4, 0.8);
    let highlight = vec3<f32>(1.0, 0.95, 0.9);
    let shadow = vec3<f32>(0.08, 0.06, 0.1);

    let up_factor = matcap_uv.y;
    let base = mix(shadow, mix(cool_color * cool_amount, warm * warm_amount, up_factor), up_factor);

    let center_dist = length(matcap_uv - 0.5) * 2.0;
    let spec_ring = 1.0 - smoothstep(0.0, 0.6, center_dist);
    let spec = pow(spec_ring, 3.0 + metallic * 8.0);

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);

    let color = base + highlight * spec * (0.5 + metallic * 0.5) + vec3<f32>(0.3, 0.4, 0.6) * fresnel * 0.3;

    return vec4<f32>(color, 1.0);
}
"
);

const GLITCH: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) glitch_factor: f32,
};

fn glitch_hash(p: f32) -> f32 {
    return fract(sin(p * 127.1) * 43758.5453);
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let intensity = uniforms.custom[0].x * 0.6;
    let speed = uniforms.custom[0].y * 15.0 + 5.0;
    let slice_size = uniforms.custom[0].z * 0.2 + 0.05;

    let time_slot = floor(uniforms.time * speed);
    let y_band = floor(pos.y / slice_size);
    let band_hash = glitch_hash(y_band + time_slot * 0.37);

    var glitch = 0.0;
    if band_hash > 0.55 {
        let offset = (glitch_hash(time_slot + y_band * 13.7) - 0.5) * intensity;
        pos.x += offset;
        glitch = abs(offset) * 3.0;
    }
    if band_hash > 0.8 {
        pos.z += (glitch_hash(time_slot * 1.3 + y_band * 7.1) - 0.5) * intensity * 0.5;
        glitch += 0.5;
    }

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.glitch_factor = clamp(glitch, 0.0, 1.0);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.6 + 0.3;

    let base = vec3<f32>(0.85, 0.88, 0.92) * diffuse;

    var color = base;
    let split = in.glitch_factor;
    color.r += split * 0.9;
    color.b -= split * 0.4;
    color.g -= split * 0.2;

    let scan = 0.85 + 0.15 * sin(in.world_position.y * 120.0 + uniforms.time * 25.0);
    color *= scan;

    let noise_val = glitch_hash(floor(in.uv.x * 80.0) + floor(in.uv.y * 80.0) * 37.0 + floor(uniforms.time * 15.0));
    if noise_val > 0.96 {
        color = vec3<f32>(1.0);
    }

    return vec4<f32>(color, 1.0);
}
"
);

const SHOCKWAVE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) wave_intensity: f32,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let amplitude = uniforms.custom[0].x * 0.4;
    let width = uniforms.custom[0].y * 0.5 + 0.1;
    let speed = uniforms.custom[0].z * 3.0 + 1.0;

    let wave_radius = fract(uniforms.time * speed * 0.2) * 3.0;
    let dist_from_center = length(pos.xz);
    let wave_dist = abs(dist_from_center - wave_radius);
    let wave = exp(-wave_dist * wave_dist / (width * width)) * amplitude;

    pos += input.normal * wave;

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.wave_intensity = wave / max(amplitude, 0.001);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 0.5));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 48.0);

    let base_color = vec3<f32>(0.4, 0.45, 0.5);
    let wave_color = vec3<f32>(0.2, 0.7, 1.0);
    let hot_color = vec3<f32>(1.0, 0.9, 0.6);

    let wave = clamp(in.wave_intensity, 0.0, 1.0);
    var color = base_color * (diffuse * 0.6 + 0.25);
    color = mix(color, wave_color * 2.0, wave * 0.7);
    color += hot_color * pow(wave, 3.0) * 2.0;
    color += vec3<f32>(1.0) * spec * 0.5;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    color += wave_color * fresnel * 0.2;

    return vec4<f32>(color, 1.0);
}
"
);

const MELT: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) melt_factor: f32,
};

fn melt_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn melt_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(melt_hash(i), melt_hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(melt_hash(i + vec2<f32>(0.0, 1.0)), melt_hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let amount = uniforms.custom[0].x;
    let speed = uniforms.custom[0].y * 2.0 + 0.5;
    let drip_strength = uniforms.custom[0].z;

    let melt_progress = (sin(uniforms.time * speed * 0.5) * 0.5 + 0.5) * amount;
    let height_factor = smoothstep(-1.0, 1.0, pos.y);

    let noise_val = melt_noise(pos.xz * 3.0 + uniforms.time * 0.2);
    let drip = noise_val * drip_strength * height_factor;

    pos.y -= melt_progress * height_factor * 0.8 + drip * melt_progress;

    let spread = melt_progress * height_factor * 0.3;
    pos.x *= 1.0 + spread;
    pos.z *= 1.0 + spread;

    let drip_wave = sin(pos.x * 8.0 + uniforms.time * speed) * 0.03 * melt_progress;
    pos.y += drip_wave * height_factor;

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.melt_factor = clamp(melt_progress * height_factor, 0.0, 1.0);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 0.5));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 24.0);

    let solid_color = vec3<f32>(0.7, 0.72, 0.75);
    let melted_color = vec3<f32>(1.0, 0.4, 0.1);
    let glow_color = vec3<f32>(1.0, 0.7, 0.2);

    let melt = in.melt_factor;
    var color = mix(solid_color, melted_color, melt);
    color *= diffuse * 0.6 + 0.3;
    color += glow_color * melt * melt * 1.5;
    color += vec3<f32>(1.0) * spec * (1.0 - melt * 0.7);

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.5);
    color += melted_color * fresnel * melt * 0.5;

    return vec4<f32>(color, 1.0);
}
"
);

const TWIST: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let twist_amount = uniforms.custom[0].x * 6.28318;
    let speed = uniforms.custom[0].y * 2.0;

    let angle = pos.y * twist_amount + uniforms.time * speed;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let twisted_x = pos.x * cos_a - pos.z * sin_a;
    let twisted_z = pos.x * sin_a + pos.z * cos_a;
    pos.x = twisted_x;
    pos.z = twisted_z;

    var normal = input.normal;
    let n_twisted_x = normal.x * cos_a - normal.z * sin_a;
    let n_twisted_z = normal.x * sin_a + normal.z * cos_a;
    normal.x = n_twisted_x;
    normal.z = n_twisted_z;

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 0.5));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 48.0);

    let height = in.world_position.y * 0.5 + 0.5;
    let warm = vec3<f32>(0.9, 0.4, 0.15);
    let cool_color = vec3<f32>(0.15, 0.35, 0.8);
    let base_color = mix(cool_color, warm, clamp(height, 0.0, 1.0));

    var color = base_color * (diffuse * 0.6 + 0.3);
    color += vec3<f32>(1.0) * spec * 0.6;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    color += vec3<f32>(0.5, 0.6, 0.9) * fresnel * 0.3;

    return vec4<f32>(color, 1.0);
}
"
);

const INFLATE: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) inflate_amount: f32,
};

fn inflate_hash(p: vec3<f32>) -> f32 {
    var q = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    q += dot(q, q.yzx + 19.19);
    return fract((q.x + q.y) * q.z);
}

fn inflate_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let n000 = inflate_hash(i);
    let n100 = inflate_hash(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = inflate_hash(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = inflate_hash(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = inflate_hash(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = inflate_hash(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = inflate_hash(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = inflate_hash(i + vec3<f32>(1.0, 1.0, 1.0));
    let x0 = mix(n000, n100, u.x);
    let x1 = mix(n010, n110, u.x);
    let x2 = mix(n001, n101, u.x);
    let x3 = mix(n011, n111, u.x);
    return mix(mix(x0, x1, u.y), mix(x2, x3, u.y), u.z);
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let base_amount = uniforms.custom[0].x * 0.5;
    let speed = uniforms.custom[0].y * 4.0 + 1.0;
    let noise_amount = uniforms.custom[0].z;

    let breath = sin(uniforms.time * speed) * 0.5 + 0.5;
    let noise_val = inflate_noise(pos * 3.0 + uniforms.time * 0.3) * noise_amount;
    let displacement = (breath * base_amount + noise_val * base_amount * 0.5);

    pos += input.normal * displacement;

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.inflate_amount = displacement / max(base_amount, 0.001);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 0.5));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 32.0);

    let inflate = clamp(in.inflate_amount, 0.0, 1.0);
    let resting_color = vec3<f32>(0.6, 0.65, 0.7);
    let inflated_color = vec3<f32>(1.0, 0.5, 0.3);
    let sss_color = vec3<f32>(1.0, 0.3, 0.1);

    let base_color = mix(resting_color, inflated_color, inflate);
    var color = base_color * (diffuse * 0.5 + 0.35);
    color += vec3<f32>(1.0) * spec * 0.4;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.0);
    let sss = pow(max(dot(-normal, light_dir), 0.0), 2.0) * inflate;
    color += sss_color * sss * 0.4;
    color += vec3<f32>(0.5, 0.6, 0.8) * fresnel * 0.25;

    return vec4<f32>(color, 1.0);
}
"
);

const SPIKES: &str = concat!(
    uniform_preamble!(),
    texture_preamble!(),
    r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) spike_amount: f32,
};

fn spike_hash(p: vec3<f32>) -> f32 {
    var q = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    q += dot(q, q.yzx + 19.19);
    return fract((q.x + q.y) * q.z);
}

fn spike_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let n000 = spike_hash(i);
    let n100 = spike_hash(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = spike_hash(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = spike_hash(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = spike_hash(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = spike_hash(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = spike_hash(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = spike_hash(i + vec3<f32>(1.0, 1.0, 1.0));
    let x0 = mix(n000, n100, u.x);
    let x1 = mix(n010, n110, u.x);
    let x2 = mix(n001, n101, u.x);
    let x3 = mix(n011, n111, u.x);
    return mix(mix(x0, x1, u.y), mix(x2, x3, u.y), u.z);
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = input.position;

    let height = uniforms.custom[0].x * 0.8;
    let sharpness = uniforms.custom[0].y * 8.0 + 2.0;
    let speed = uniforms.custom[0].z * 2.0;

    let noise_val = spike_noise(pos * sharpness + vec3<f32>(uniforms.time * speed, 0.0, 0.0));
    let spike = pow(noise_val, 3.0) * height;

    pos += input.normal * spike;

    let world_pos = uniforms.model * vec4<f32>(pos, 1.0);
    out.clip_position = uniforms.projection * uniforms.view * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.spike_amount = spike / max(height, 0.001);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(uniforms.camera_position - in.world_position);
    let light_dir = normalize(vec3<f32>(1.0, 1.5, 0.5));

    let diffuse = max(dot(normal, light_dir), 0.0);
    let half_vec = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), 64.0);

    let spike = clamp(in.spike_amount, 0.0, 1.0);
    let base_color = vec3<f32>(0.15, 0.12, 0.2);
    let tip_color = vec3<f32>(0.8, 0.2, 0.05);
    let glow_color = vec3<f32>(1.0, 0.5, 0.1);

    let surface_color = mix(base_color, tip_color, spike);
    var color = surface_color * (diffuse * 0.5 + 0.2);
    color += glow_color * pow(spike, 2.0) * 1.5;
    color += vec3<f32>(1.0) * spec * 0.3;

    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    color += vec3<f32>(0.5, 0.1, 0.0) * fresnel * 0.4;

    return vec4<f32>(color, 1.0);
}
"
);

pub fn default_fullscreen_source() -> &'static str {
    PLASMA
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_source(name: &str, label: &str, source: &str, failures: &mut Vec<String>) {
        match naga::front::wgsl::parse_str(source) {
            Ok(module) => {
                let mut validator = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                );
                if let Err(error) = validator.validate(&module) {
                    failures.push(format!("{name} ({label}): validation error: {error}"));
                }
            }
            Err(error) => {
                failures.push(format!("{name} ({label}): parse error: {error}"));
            }
        }
    }

    #[test]
    fn all_presets_compile() {
        let mut failures = Vec::new();
        for preset in PRESETS {
            let common = preset.common_source.unwrap_or("");
            let image_source = if common.is_empty() {
                preset.source.to_string()
            } else {
                format!("{common}\n{}", preset.source)
            };
            validate_source(preset.name, "Image", &image_source, &mut failures);

            for (label, buffer_source) in [
                ("Buffer A", preset.buffer_a_source),
                ("Buffer B", preset.buffer_b_source),
                ("Buffer C", preset.buffer_c_source),
                ("Buffer D", preset.buffer_d_source),
            ] {
                if let Some(source) = buffer_source {
                    let full = if common.is_empty() {
                        source.to_string()
                    } else {
                        format!("{common}\n{source}")
                    };
                    validate_source(preset.name, label, &full, &mut failures);
                }
            }
        }
        if !failures.is_empty() {
            panic!(
                "{} shader(s) failed:\n{}",
                failures.len(),
                failures.join("\n\n")
            );
        }
    }
}
