use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;
use rand::Rng;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DemoSceneState::default())
}

fn decode_audio_file(path: &std::path::Path) -> Result<(Vec<f32>, u32, Vec<u8>), String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("No default track")?;
    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|e| e.to_string())?;

    let mut all_samples: Vec<f32> = Vec::new();

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        let Ok(decoded) = decoder.decode(&packet) else {
            continue;
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;

        let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        let samples = sample_buf.samples();
        let channels = spec.channels.count();

        for chunk in samples.chunks(channels) {
            let mono: f32 = chunk.iter().sum::<f32>() / channels as f32;
            all_samples.push(mono);
        }
    }

    let audio_bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok((all_samples, sample_rate, audio_bytes))
}

const DEMOSCENE_SHADER: &str = include_str!("../shaders/demoscene.wgsl");

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DemosceneUniforms {
    time: f32,
    chromatic_aberration: f32,
    wave_distortion: f32,
    color_shift: f32,
    kaleidoscope_segments: f32,
    crt_scanlines: f32,
    vignette: f32,
    plasma_intensity: f32,
    glitch_intensity: f32,
    mirror_mode: f32,
    invert: f32,
    hue_rotation: f32,
    raymarch_mode: f32,
    raymarch_blend: f32,
    film_grain: f32,
    sharpen: f32,
    pixelate: f32,
    color_posterize: f32,
    radial_blur: f32,
    tunnel_speed: f32,
    fractal_iterations: f32,
    glow_intensity: f32,
    screen_shake: f32,
    zoom_pulse: f32,
    speed_lines: f32,
    color_grade_mode: f32,
    vhs_distortion: f32,
    lens_flare: f32,
    edge_glow: f32,
    saturation: f32,
    warp_speed: f32,
    pulse_rings: f32,
    heat_distortion: f32,
    digital_rain: f32,
    strobe: f32,
    color_cycle_speed: f32,
    feedback_amount: f32,
    ascii_mode: f32,
}

impl Default for DemosceneUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            chromatic_aberration: 0.6,
            wave_distortion: 0.4,
            color_shift: 0.5,
            kaleidoscope_segments: 0.0,
            crt_scanlines: 0.0,
            vignette: 0.4,
            plasma_intensity: 0.3,
            glitch_intensity: 0.0,
            mirror_mode: 0.0,
            invert: 0.0,
            hue_rotation: 0.0,
            raymarch_mode: 0.0,
            raymarch_blend: 0.5,
            film_grain: 0.0,
            sharpen: 0.0,
            pixelate: 0.0,
            color_posterize: 0.0,
            radial_blur: 0.0,
            tunnel_speed: 1.0,
            fractal_iterations: 4.0,
            glow_intensity: 0.0,
            screen_shake: 0.0,
            zoom_pulse: 0.0,
            speed_lines: 0.0,
            color_grade_mode: 0.0,
            vhs_distortion: 0.0,
            lens_flare: 0.0,
            edge_glow: 0.0,
            saturation: 1.0,
            warp_speed: 0.0,
            pulse_rings: 0.0,
            heat_distortion: 0.0,
            digital_rain: 0.0,
            strobe: 0.0,
            color_cycle_speed: 1.0,
            feedback_amount: 0.0,
            ascii_mode: 0.0,
        }
    }
}

const VISUALIZER_WAVEFORM_SIZE: usize = 512;
const VISUALIZER_SPECTRUM_SIZE: usize = 128;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct AudioVisualizerData {
    waveform_intensity: f32,
    spectrum_intensity: f32,
    beat_pulse: f32,
    bass_level: f32,
    mids_level: f32,
    highs_level: f32,
    onset_flash: f32,
    bpm: f32,
    beat_phase: f32,
    drop_intensity: f32,
    spectral_centroid: f32,
    energy: f32,
    time: f32,
    visualizer_mode: f32,
    visualizer_opacity: f32,
    kick_decay: f32,
    waveform: [f32; VISUALIZER_WAVEFORM_SIZE],
    spectrum: [f32; VISUALIZER_SPECTRUM_SIZE],
}

impl Default for AudioVisualizerData {
    fn default() -> Self {
        Self {
            waveform_intensity: 0.0,
            spectrum_intensity: 0.0,
            beat_pulse: 0.0,
            bass_level: 0.0,
            mids_level: 0.0,
            highs_level: 0.0,
            onset_flash: 0.0,
            bpm: 120.0,
            beat_phase: 0.0,
            drop_intensity: 0.0,
            spectral_centroid: 0.0,
            energy: 0.0,
            time: 0.0,
            visualizer_mode: 0.0,
            visualizer_opacity: 0.0,
            kick_decay: 0.0,
            waveform: [0.0; VISUALIZER_WAVEFORM_SIZE],
            spectrum: [0.0; VISUALIZER_SPECTRUM_SIZE],
        }
    }
}

struct SharedState {
    uniforms: DemosceneUniforms,
    audio_data: AudioVisualizerData,
    enabled: bool,
    animate_hue: bool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            uniforms: DemosceneUniforms::default(),
            audio_data: AudioVisualizerData::default(),
            enabled: true,
            animate_hue: false,
        }
    }
}

type SharedStateHandle = Arc<RwLock<SharedState>>;

struct DemoscenePass {
    pipeline: wgpu::RenderPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    audio_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    cached_blit_bind_group: Option<wgpu::BindGroup>,
    shared_state: SharedStateHandle,
}

impl DemoscenePass {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        blit_pipeline: wgpu::RenderPipeline,
        shared_state: SharedStateHandle,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Demoscene Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(DEMOSCENE_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Demoscene Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Demoscene Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Demoscene Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Demoscene Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Demoscene Uniform Buffer"),
            size: std::mem::size_of::<DemosceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let audio_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Audio Visualizer Buffer"),
            size: std::mem::size_of::<AudioVisualizerData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Blit Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            pipeline,
            blit_pipeline,
            bind_group_layout,
            blit_bind_group_layout,
            sampler,
            uniform_buffer,
            audio_buffer,
            cached_bind_group: None,
            cached_blit_bind_group: None,
            shared_state,
        }
    }
}

impl PassNode<World> for DemoscenePass {
    fn name(&self) -> &str {
        "demoscene_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
        self.cached_blit_bind_group = None;
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, world: &World) {
        let time = world.resources.window.timing.uptime_milliseconds as f32 * 0.001;
        if let Ok(mut state) = self.shared_state.write() {
            state.uniforms.time = time;
            if state.animate_hue {
                state.uniforms.hue_rotation = (time * 0.1) % 1.0;
            }
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&state.uniforms));
            queue.write_buffer(&self.audio_buffer, 0, bytemuck::bytes_of(&state.audio_data));
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        let input_view = context.get_texture_view("input")?;

        if self.cached_bind_group.is_none() {
            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Demoscene Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.audio_buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        if self.cached_blit_bind_group.is_none() {
            self.cached_blit_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Demoscene Blit Bind Group"),
                    layout: &self.blit_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                },
            ));
        }

        let enabled = self.shared_state.read().map(|s| s.enabled).unwrap_or(true);
        let (pipeline, bind_group) = if enabled {
            (&self.pipeline, self.cached_bind_group.as_ref().unwrap())
        } else {
            (
                &self.blit_pipeline,
                self.cached_blit_bind_group.as_ref().unwrap(),
            )
        };

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Demoscene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load_op,
                        store: color_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DemoPhase {
    TorusField,
    CubeVortex,
    SphereGrid,
    PlasmaRings,
    Finale,
}

impl DemoPhase {
    fn name(&self) -> &str {
        match self {
            DemoPhase::TorusField => "Torus Field",
            DemoPhase::CubeVortex => "Cube Vortex",
            DemoPhase::SphereGrid => "Sphere Grid",
            DemoPhase::PlasmaRings => "Plasma Rings",
            DemoPhase::Finale => "Finale",
        }
    }

    fn all() -> &'static [DemoPhase] {
        &[
            DemoPhase::TorusField,
            DemoPhase::CubeVortex,
            DemoPhase::SphereGrid,
            DemoPhase::PlasmaRings,
            DemoPhase::Finale,
        ]
    }
}

struct AnimatedObject {
    entity: Entity,
    base_position: Vec3,
    phase_offset: f32,
    color_offset: f32,
    scale: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    spin_axis: Vec3,
    spin_speed: f32,
}

struct MovingLight {
    entity: Entity,
    sphere_entity: Entity,
    base_color: Vec3,
    orbit_radius: f32,
    orbit_speed: f32,
    height_offset: f32,
    phase_offset: f32,
}

#[derive(Clone, Copy)]
struct CameraKeyframe {
    position: Vec3,
    time: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum CameraMode {
    Cinematic,
    Orbit,
    Manual,
}

struct DemoSceneState {
    current_phase: DemoPhase,
    phase_time: f32,
    phase_duration: f32,
    auto_transition: bool,
    global_time: f32,
    objects: Vec<AnimatedObject>,
    lights: Vec<MovingLight>,
    particle_emitters: Vec<Entity>,
    title_text: Option<Entity>,
    camera_entity: Option<Entity>,
    color_cycle_speed: f32,
    rotation_speed: f32,
    pulse_intensity: f32,
    bloom_intensity: f32,
    camera_mode: CameraMode,
    camera_keyframes: Vec<CameraKeyframe>,
    camera_orbit_speed: f32,
    camera_orbit_radius: f32,
    camera_orbit_height: f32,
    object_count: usize,
    light_count: usize,
    material_counter: usize,
    shared_state: SharedStateHandle,
    chrome_spheres: Vec<Entity>,
    audio_file_path: Option<PathBuf>,
    audio_analyzer: AudioAnalyzer,
    audio_entity: Option<Entity>,
    audio_playing: bool,
    audio_start_time: f32,
    audio_status: String,
    kaleidoscope_blend: f32,
    bass_sensitivity: f32,
    mids_sensitivity: f32,
    highs_sensitivity: f32,
    firework_shells: Vec<FireworkShell>,
    last_firework_time: f32,
    firework_cooldown: f32,
    visualizer_opacity: f32,
    visualizer_mode: u32,
}

struct FireworkShell {
    entity: Entity,
    position: Vec3,
    velocity: Vec3,
    fuse_time: f32,
    color: Vec3,
    particle_count: u32,
}

impl Default for DemoSceneState {
    fn default() -> Self {
        Self {
            current_phase: DemoPhase::TorusField,
            phase_time: 0.0,
            phase_duration: 15.0,
            auto_transition: true,
            global_time: 0.0,
            objects: Vec::new(),
            lights: Vec::new(),
            particle_emitters: Vec::new(),
            title_text: None,
            camera_entity: None,
            color_cycle_speed: 1.0,
            rotation_speed: 1.0,
            pulse_intensity: 1.0,
            bloom_intensity: 0.8,
            camera_mode: CameraMode::Cinematic,
            camera_keyframes: Vec::new(),
            camera_orbit_speed: 0.15,
            camera_orbit_radius: 40.0,
            camera_orbit_height: 15.0,
            object_count: 64,
            light_count: 12,
            material_counter: 0,
            shared_state: Arc::new(RwLock::new(SharedState::default())),
            chrome_spheres: Vec::new(),
            audio_file_path: None,
            audio_analyzer: AudioAnalyzer::new(),
            audio_entity: None,
            audio_playing: false,
            audio_start_time: 0.0,
            audio_status: String::new(),
            kaleidoscope_blend: 0.0,
            bass_sensitivity: 1.0,
            mids_sensitivity: 1.0,
            highs_sensitivity: 1.0,
            firework_shells: Vec::new(),
            last_firework_time: 0.0,
            firework_cooldown: 0.3,
            visualizer_opacity: 0.8,
            visualizer_mode: 1,
        }
    }
}

impl DemoSceneState {
    fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
        let hue = hue % 360.0;
        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;

        let (r, g, b) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Vec3::new(r + m, g + m, b + m)
    }

    fn randomize_uniforms(&mut self) {
        let mut rng = rand::rng();

        if let Ok(mut state) = self.shared_state.write() {
            state.uniforms.chromatic_aberration = rng.random_range(0.0..0.8);
            state.uniforms.wave_distortion = rng.random_range(0.0..0.6);
            state.uniforms.color_shift = rng.random_range(0.0..1.0);
            state.uniforms.kaleidoscope_segments = if rng.random::<f32>() > 0.7 {
                [0.0, 4.0, 6.0, 8.0][rng.random_range(0..4)]
            } else {
                0.0
            };
            state.uniforms.crt_scanlines = if rng.random::<f32>() > 0.8 {
                rng.random_range(0.0..0.15)
            } else {
                0.0
            };
            state.uniforms.vignette = rng.random_range(0.1..0.6);
            state.uniforms.plasma_intensity = rng.random_range(0.0..0.5);
            state.uniforms.glitch_intensity = if rng.random::<f32>() > 0.7 {
                rng.random_range(0.0..0.3)
            } else {
                0.0
            };
            state.uniforms.mirror_mode = if rng.random::<f32>() > 0.85 { 1.0 } else { 0.0 };
            state.uniforms.invert = if rng.random::<f32>() > 0.9 {
                rng.random_range(0.0..0.2)
            } else {
                0.0
            };
            state.uniforms.hue_rotation = rng.random_range(0.0..360.0);
            state.uniforms.raymarch_mode = if rng.random::<f32>() > 0.8 {
                rng.random_range(0.0_f32..3.0).floor()
            } else {
                0.0
            };
            state.uniforms.raymarch_blend = rng.random_range(0.0..0.5);
            state.uniforms.film_grain = rng.random_range(0.0..0.03);
            state.uniforms.sharpen = rng.random_range(0.0..0.2);
            state.uniforms.pixelate = if rng.random::<f32>() > 0.9 {
                rng.random_range(1.0_f32..4.0).floor()
            } else {
                0.0
            };
            state.uniforms.radial_blur = rng.random_range(0.0..0.05);
            state.uniforms.tunnel_speed = rng.random_range(0.3..2.0);
            state.uniforms.fractal_iterations = rng.random_range(2.0_f32..6.0).floor();
            state.uniforms.glow_intensity = rng.random_range(0.0..0.3);
            state.uniforms.screen_shake = rng.random_range(0.0..0.1);
            state.uniforms.zoom_pulse = rng.random_range(0.0..0.1);
            state.uniforms.speed_lines = rng.random_range(0.0..0.3);
            state.uniforms.color_grade_mode = rng.random_range(0.0_f32..7.0).floor();
            state.uniforms.vhs_distortion = if rng.random::<f32>() > 0.8 {
                rng.random_range(0.0..0.1)
            } else {
                0.0
            };
            state.uniforms.lens_flare = rng.random_range(0.0..0.5);
            state.uniforms.edge_glow = rng.random_range(0.0..0.3);
            state.uniforms.saturation = rng.random_range(0.3..1.5);
            state.uniforms.warp_speed = if rng.random::<f32>() > 0.8 {
                rng.random_range(0.0..0.6)
            } else {
                0.0
            };
            state.uniforms.pulse_rings = if rng.random::<f32>() > 0.8 {
                rng.random_range(0.0..0.5)
            } else {
                0.0
            };
            state.uniforms.heat_distortion = if rng.random::<f32>() > 0.85 {
                rng.random_range(0.0..0.5)
            } else {
                0.0
            };
            state.uniforms.digital_rain = if rng.random::<f32>() > 0.9 {
                rng.random_range(0.0..0.4)
            } else {
                0.0
            };
            state.uniforms.strobe = 0.0;
        }

        self.bloom_intensity = rng.random_range(0.2..0.8);
        self.color_cycle_speed = rng.random_range(0.5..2.0);
        self.rotation_speed = rng.random_range(0.3..2.0);
        self.pulse_intensity = rng.random_range(0.5..2.0);
    }

    fn create_material(&mut self, world: &mut World, color: Vec3, emissive: Vec3) -> String {
        let material_name = format!("DemoMaterial_{}", self.material_counter);
        self.material_counter += 1;

        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [color.x, color.y, color.z, 1.0],
                emissive_factor: [emissive.x, emissive.y, emissive.z],
                roughness: 0.3,
                metallic: 0.8,
                ..Default::default()
            },
        );

        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        material_name
    }

    fn spawn_demo_object(
        &mut self,
        world: &mut World,
        mesh_name: &str,
        position: Vec3,
        scale: f32,
        color: Vec3,
        emissive: Vec3,
    ) -> Entity {
        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RENDER_MESH | MATERIAL_REF,
            1,
        )[0];

        world.core.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(scale, scale, scale),
            },
        );
        world
            .core
            .set_render_mesh(entity, RenderMesh::new(mesh_name));

        let material_name = self.create_material(world, color, emissive);
        world
            .core
            .set_material_ref(entity, MaterialRef::new(material_name));

        entity
    }

    fn spawn_light(
        &mut self,
        world: &mut World,
        position: Vec3,
        color: Vec3,
        intensity: f32,
    ) -> (Entity, Entity) {
        let light_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
            1,
        )[0];

        world.core.set_name(
            light_entity,
            Name(format!("DemoLight_{}", self.light_count)),
        );
        world.core.set_local_transform(
            light_entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world
            .core
            .set_local_transform_dirty(light_entity, LocalTransformDirty);
        world
            .core
            .set_global_transform(light_entity, GlobalTransform::default());
        world.core.set_light(
            light_entity,
            Light {
                light_type: LightType::Point,
                color,
                intensity,
                range: 50.0,
                inner_cone_angle: 0.0,
                outer_cone_angle: 0.0,
                cast_shadows: false,
                shadow_bias: 0.007,
            },
        );

        let sphere_entity = self.spawn_demo_object(
            world,
            "Sphere",
            position,
            0.3,
            Vec3::new(0.0, 0.0, 0.0),
            color * 3.0,
        );

        (light_entity, sphere_entity)
    }

    fn clear_scene(&mut self, world: &mut World) {
        for object in self.objects.drain(..) {
            despawn_recursive_immediate(world, object.entity);
        }

        for light in self.lights.drain(..) {
            despawn_recursive_immediate(world, light.entity);
            despawn_recursive_immediate(world, light.sphere_entity);
        }

        for emitter in self.particle_emitters.drain(..) {
            if let Some(particle_emitter) = world.core.get_particle_emitter_mut(emitter) {
                particle_emitter.enabled = false;
            }
        }

        for chrome in self.chrome_spheres.drain(..) {
            despawn_recursive_immediate(world, chrome);
        }
    }

    fn spawn_chrome_sphere(&mut self, world: &mut World, position: Vec3, scale: f32) -> Entity {
        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RENDER_MESH | MATERIAL_REF,
            1,
        )[0];

        world.core.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(scale, scale, scale),
            },
        );
        world
            .core
            .set_render_mesh(entity, RenderMesh::new("Sphere"));

        let material_name = format!("ChromeMaterial_{}", self.material_counter);
        self.material_counter += 1;

        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.9, 0.9, 0.95, 1.0],
                emissive_factor: [0.0, 0.0, 0.0],
                roughness: 0.05,
                metallic: 1.0,
                ..Default::default()
            },
        );

        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        world
            .core
            .set_material_ref(entity, MaterialRef::new(material_name));
        entity
    }

    fn spawn_chrome_spheres_for_phase(&mut self, world: &mut World, phase: DemoPhase) {
        let mut rng = rand::rng();

        let positions: Vec<(Vec3, f32)> = match phase {
            DemoPhase::TorusField => (0..8)
                .map(|index| {
                    let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
                    let radius = 15.0;
                    (
                        Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius),
                        1.5,
                    )
                })
                .collect(),
            DemoPhase::CubeVortex => (0..6)
                .map(|index| {
                    let height = -12.0 + index as f32 * 5.0;
                    let angle = index as f32 * 1.2;
                    let radius = 8.0 + index as f32 * 2.0;
                    (
                        Vec3::new(angle.cos() * radius, height, angle.sin() * radius),
                        1.0 + index as f32 * 0.3,
                    )
                })
                .collect(),
            DemoPhase::SphereGrid => {
                vec![
                    (Vec3::new(0.0, 0.0, 0.0), 3.0),
                    (Vec3::new(-12.0, -12.0, -12.0), 2.0),
                    (Vec3::new(12.0, 12.0, 12.0), 2.0),
                    (Vec3::new(-12.0, 12.0, -12.0), 1.5),
                    (Vec3::new(12.0, -12.0, 12.0), 1.5),
                ]
            }
            DemoPhase::PlasmaRings => (0..12)
                .map(|index| {
                    let angle = (index as f32 / 12.0) * std::f32::consts::TAU;
                    let radius = 8.0;
                    let height = (angle * 2.0).sin() * 5.0;
                    (
                        Vec3::new(angle.cos() * radius, height, angle.sin() * radius),
                        0.8,
                    )
                })
                .collect(),
            DemoPhase::Finale => (0..20)
                .map(|_| {
                    let theta = rng.random::<f32>() * std::f32::consts::TAU;
                    let phi = rng.random::<f32>() * std::f32::consts::PI;
                    let radius = 12.0 + rng.random::<f32>() * 10.0;
                    let position = Vec3::new(
                        radius * phi.sin() * theta.cos(),
                        radius * phi.cos(),
                        radius * phi.sin() * theta.sin(),
                    );
                    (position, 0.5 + rng.random::<f32>() * 1.5)
                })
                .collect(),
        };

        for (position, scale) in positions {
            let entity = self.spawn_chrome_sphere(world, position, scale);
            self.chrome_spheres.push(entity);
        }
    }

    fn setup_torus_field(&mut self, world: &mut World) {
        self.clear_scene(world);

        let mut rng = rand::rng();
        let ring_count = 4;
        let objects_per_ring = self.object_count / ring_count;

        for ring_index in 0..ring_count {
            let ring_radius = 8.0 + ring_index as f32 * 6.0;
            let height = (ring_index as f32 - 1.5) * 4.0;

            for object_index in 0..objects_per_ring {
                let angle = (object_index as f32 / objects_per_ring as f32) * std::f32::consts::TAU;
                let position =
                    Vec3::new(angle.cos() * ring_radius, height, angle.sin() * ring_radius);

                let hue = (ring_index as f32 * 90.0 + object_index as f32 * 10.0) % 360.0;
                let color = Self::hsv_to_rgb(hue, 0.8, 0.9);
                let emissive = color * 0.5;

                let entity = self.spawn_demo_object(world, "Torus", position, 0.8, color, emissive);

                self.objects.push(AnimatedObject {
                    entity,
                    base_position: position,
                    phase_offset: angle + ring_index as f32 * 0.5,
                    color_offset: hue,
                    scale: 0.8,
                    orbit_radius: ring_radius,
                    orbit_speed: 0.3 * (1.0 + ring_index as f32 * 0.2),
                    spin_axis: Vec3::new(
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                    )
                    .normalize(),
                    spin_speed: rng.random_range(0.5..2.0),
                });
            }
        }

        self.setup_lights(world, DemoPhase::TorusField);
        self.spawn_phase_particles(world, DemoPhase::TorusField);
    }

    fn setup_cube_vortex(&mut self, world: &mut World) {
        self.clear_scene(world);

        let mut rng = rand::rng();
        let spiral_turns = 5;
        let objects_per_turn = self.object_count / spiral_turns;

        for turn_index in 0..spiral_turns {
            for object_index in 0..objects_per_turn {
                let progress = (turn_index * objects_per_turn + object_index) as f32
                    / (spiral_turns * objects_per_turn) as f32;
                let angle = progress * std::f32::consts::TAU * spiral_turns as f32;
                let radius = 5.0 + progress * 20.0;
                let height = -15.0 + progress * 30.0;

                let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

                let hue = progress * 360.0;
                let color = Self::hsv_to_rgb(hue, 0.9, 1.0);
                let emissive = color * (0.3 + progress * 0.7);

                let scale = 0.3 + progress * 1.2;
                let entity =
                    self.spawn_demo_object(world, "Cube", position, scale, color, emissive);

                self.objects.push(AnimatedObject {
                    entity,
                    base_position: position,
                    phase_offset: angle,
                    color_offset: hue,
                    scale,
                    orbit_radius: radius,
                    orbit_speed: 0.5 - progress * 0.3,
                    spin_axis: Vec3::new(
                        rng.random_range(-1.0..1.0),
                        1.0,
                        rng.random_range(-1.0..1.0),
                    )
                    .normalize(),
                    spin_speed: 1.0 + progress * 2.0,
                });
            }
        }

        self.setup_lights(world, DemoPhase::CubeVortex);
        self.spawn_phase_particles(world, DemoPhase::CubeVortex);
    }

    fn setup_sphere_grid(&mut self, world: &mut World) {
        self.clear_scene(world);

        let grid_size = (self.object_count as f32).cbrt().ceil() as usize;
        let spacing = 5.0;
        let offset = (grid_size as f32 - 1.0) * spacing / 2.0;

        let mut index = 0;
        for x_index in 0..grid_size {
            for y_index in 0..grid_size {
                for z_index in 0..grid_size {
                    if index >= self.object_count {
                        break;
                    }

                    let position = Vec3::new(
                        x_index as f32 * spacing - offset,
                        y_index as f32 * spacing - offset,
                        z_index as f32 * spacing - offset,
                    );

                    let distance = position.magnitude();
                    let hue = distance * 20.0;
                    let color = Self::hsv_to_rgb(hue, 0.7, 0.9);
                    let emissive = color * 0.4;

                    let entity =
                        self.spawn_demo_object(world, "Sphere", position, 0.6, color, emissive);

                    self.objects.push(AnimatedObject {
                        entity,
                        base_position: position,
                        phase_offset: distance * 0.1,
                        color_offset: hue,
                        scale: 0.6,
                        orbit_radius: 0.0,
                        orbit_speed: 0.0,
                        spin_axis: Vec3::y(),
                        spin_speed: 0.5,
                    });

                    index += 1;
                }
            }
        }

        self.setup_lights(world, DemoPhase::SphereGrid);
        self.spawn_phase_particles(world, DemoPhase::SphereGrid);
    }

    fn setup_plasma_rings(&mut self, world: &mut World) {
        self.clear_scene(world);

        let mut rng = rand::rng();
        let ring_count = 6;
        let objects_per_ring = self.object_count / ring_count;

        for ring_index in 0..ring_count {
            let ring_radius = 12.0;
            let ring_tilt = (ring_index as f32 / ring_count as f32) * std::f32::consts::PI;

            for object_index in 0..objects_per_ring {
                let angle = (object_index as f32 / objects_per_ring as f32) * std::f32::consts::TAU;

                let local_position =
                    Vec3::new(angle.cos() * ring_radius, 0.0, angle.sin() * ring_radius);

                let rotation_axis = Vec3::new(ring_tilt.cos(), 0.0, ring_tilt.sin());
                let rotation_angle = ring_index as f32 * std::f32::consts::PI / ring_count as f32;
                let rotation = nalgebra_glm::quat_angle_axis(rotation_angle, &rotation_axis);
                let position = nalgebra_glm::quat_rotate_vec3(&rotation, &local_position);

                let hue = (ring_index as f32 * 60.0 + angle.to_degrees()) % 360.0;
                let color = Self::hsv_to_rgb(hue, 1.0, 1.0);
                let emissive = color * 1.5;

                let entity = self.spawn_demo_object(world, "Torus", position, 0.5, color, emissive);

                self.objects.push(AnimatedObject {
                    entity,
                    base_position: position,
                    phase_offset: angle + ring_index as f32,
                    color_offset: hue,
                    scale: 0.5,
                    orbit_radius: ring_radius,
                    orbit_speed: 0.8 + ring_index as f32 * 0.1,
                    spin_axis: rotation_axis,
                    spin_speed: rng.random_range(1.0..3.0),
                });
            }
        }

        self.setup_lights(world, DemoPhase::PlasmaRings);
        self.spawn_phase_particles(world, DemoPhase::PlasmaRings);
    }

    fn setup_finale(&mut self, world: &mut World) {
        self.clear_scene(world);

        let mut rng = rand::rng();
        let layer_count = 3;
        let objects_per_layer = self.object_count / layer_count;

        for layer_index in 0..layer_count {
            let mesh_name = match layer_index % 3 {
                0 => "Torus",
                1 => "Cube",
                _ => "Sphere",
            };

            for _object_index in 0..objects_per_layer {
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let phi = rng.random::<f32>() * std::f32::consts::PI;
                let radius = 8.0 + layer_index as f32 * 8.0;

                let position = Vec3::new(
                    radius * phi.sin() * theta.cos(),
                    radius * phi.cos(),
                    radius * phi.sin() * theta.sin(),
                );

                let hue = rng.random::<f32>() * 360.0;
                let color = Self::hsv_to_rgb(hue, 1.0, 1.0);
                let emissive = color * (1.0 + layer_index as f32 * 0.5);

                let scale = rng.random_range(0.4..1.2);
                let entity =
                    self.spawn_demo_object(world, mesh_name, position, scale, color, emissive);

                self.objects.push(AnimatedObject {
                    entity,
                    base_position: position,
                    phase_offset: rng.random::<f32>() * std::f32::consts::TAU,
                    color_offset: hue,
                    scale,
                    orbit_radius: radius,
                    orbit_speed: rng.random_range(0.2..1.0),
                    spin_axis: Vec3::new(
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                        rng.random_range(-1.0..1.0),
                    )
                    .normalize(),
                    spin_speed: rng.random_range(0.5..3.0),
                });
            }
        }

        self.setup_lights(world, DemoPhase::Finale);
        self.spawn_phase_particles(world, DemoPhase::Finale);
    }

    fn setup_lights(&mut self, world: &mut World, phase: DemoPhase) {
        let light_colors = match phase {
            DemoPhase::TorusField => vec![
                Vec3::new(1.0, 0.3, 0.3),
                Vec3::new(0.3, 1.0, 0.3),
                Vec3::new(0.3, 0.3, 1.0),
                Vec3::new(1.0, 1.0, 0.3),
                Vec3::new(1.0, 0.3, 1.0),
                Vec3::new(0.3, 1.0, 1.0),
            ],
            DemoPhase::CubeVortex => vec![
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(1.0, 0.0, 0.5),
                Vec3::new(0.5, 0.0, 1.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(0.0, 1.0, 0.5),
                Vec3::new(0.5, 1.0, 0.0),
            ],
            DemoPhase::SphereGrid => vec![
                Vec3::new(0.8, 0.8, 1.0),
                Vec3::new(1.0, 0.8, 0.8),
                Vec3::new(0.8, 1.0, 0.8),
                Vec3::new(1.0, 1.0, 0.8),
            ],
            DemoPhase::PlasmaRings => vec![
                Vec3::new(1.0, 0.0, 0.5),
                Vec3::new(0.5, 0.0, 1.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(0.0, 1.0, 0.5),
                Vec3::new(0.5, 1.0, 0.0),
                Vec3::new(1.0, 0.5, 0.0),
            ],
            DemoPhase::Finale => vec![
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(1.0, 0.8, 0.3),
                Vec3::new(0.3, 0.8, 1.0),
                Vec3::new(1.0, 0.3, 0.8),
                Vec3::new(0.8, 1.0, 0.3),
                Vec3::new(0.3, 1.0, 0.8),
                Vec3::new(0.8, 0.3, 1.0),
                Vec3::new(1.0, 0.5, 0.5),
            ],
        };

        for (light_index, color) in light_colors.iter().enumerate().take(self.light_count) {
            let angle = (light_index as f32 / light_colors.len() as f32) * std::f32::consts::TAU;
            let radius = 20.0;
            let height = (light_index as f32 - light_colors.len() as f32 / 2.0) * 3.0;

            let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

            let (light_entity, sphere_entity) = self.spawn_light(world, position, *color, 5.0);

            self.lights.push(MovingLight {
                entity: light_entity,
                sphere_entity,
                base_color: *color,
                orbit_radius: radius,
                orbit_speed: 0.3 + light_index as f32 * 0.05,
                height_offset: height,
                phase_offset: angle,
            });
        }
    }

    fn spawn_phase_particles(&mut self, world: &mut World, phase: DemoPhase) {
        let positions = match phase {
            DemoPhase::TorusField => vec![
                Vec3::new(0.0, -10.0, 0.0),
                Vec3::new(15.0, -10.0, 0.0),
                Vec3::new(-15.0, -10.0, 0.0),
            ],
            DemoPhase::CubeVortex => vec![Vec3::new(0.0, -20.0, 0.0), Vec3::new(0.0, 20.0, 0.0)],
            DemoPhase::SphereGrid => vec![
                Vec3::new(-15.0, -15.0, -15.0),
                Vec3::new(15.0, -15.0, 15.0),
                Vec3::new(-15.0, 15.0, 15.0),
                Vec3::new(15.0, 15.0, -15.0),
            ],
            DemoPhase::PlasmaRings => vec![Vec3::new(0.0, 0.0, 0.0)],
            DemoPhase::Finale => vec![
                Vec3::new(0.0, -25.0, 0.0),
                Vec3::new(20.0, 0.0, 0.0),
                Vec3::new(-20.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 20.0),
                Vec3::new(0.0, 0.0, -20.0),
            ],
        };

        for position in positions {
            let entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
            let mut emitter = ParticleEmitter::fire(position);
            emitter.spawn_rate = 50.0;
            emitter.particle_lifetime_min = 2.0;
            emitter.particle_lifetime_max = 4.0;
            emitter.size_start = 0.5;
            emitter.size_end = 0.1;
            emitter.emissive_strength = 8.0;
            emitter.initial_velocity_min = 3.0;
            emitter.initial_velocity_max = 8.0;
            world.core.set_particle_emitter(entity, emitter);
            self.particle_emitters.push(entity);
        }
    }

    fn switch_phase(&mut self, world: &mut World, phase: DemoPhase) {
        self.current_phase = phase;
        self.phase_time = 0.0;
        self.camera_keyframes = Self::get_camera_keyframes_for_phase(phase, self.phase_duration);

        match phase {
            DemoPhase::TorusField => self.setup_torus_field(world),
            DemoPhase::CubeVortex => self.setup_cube_vortex(world),
            DemoPhase::SphereGrid => self.setup_sphere_grid(world),
            DemoPhase::PlasmaRings => self.setup_plasma_rings(world),
            DemoPhase::Finale => self.setup_finale(world),
        }

        self.spawn_chrome_spheres_for_phase(world, phase);

        if let Some(title_entity) = self.title_text {
            if let Some(text) = world.core.get_text(title_entity) {
                let text_index = text.text_index;
                world
                    .resources
                    .text_cache
                    .set_text(text_index, phase.name().to_string());
            }
            if let Some(text) = world.core.get_text_mut(title_entity) {
                text.dirty = true;
            }
        }
    }

    fn randomize_shader_settings(&mut self) {
        let mut rng = rand::rng();
        if let Ok(mut state) = self.shared_state.write() {
            state.uniforms.chromatic_aberration = rng.random::<f32>() * 1.5;
            state.uniforms.wave_distortion = rng.random::<f32>() * 1.5;
            state.uniforms.color_shift = rng.random::<f32>() * 1.5;
            state.uniforms.kaleidoscope_segments = if rng.random::<f32>() > 0.7 {
                (rng.random::<f32>() * 10.0 + 2.0).floor()
            } else {
                0.0
            };
            state.uniforms.crt_scanlines = if rng.random::<f32>() > 0.6 {
                rng.random::<f32>() * 0.8
            } else {
                0.0
            };
            state.uniforms.vignette = rng.random::<f32>() * 1.2;
            state.uniforms.plasma_intensity = rng.random::<f32>() * 0.6;
            state.uniforms.glitch_intensity = if rng.random::<f32>() > 0.7 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.mirror_mode = if rng.random::<f32>() > 0.8 { 1.0 } else { 0.0 };
            state.uniforms.invert = if rng.random::<f32>() > 0.9 { 1.0 } else { 0.0 };
            state.uniforms.hue_rotation = rng.random::<f32>();
            state.uniforms.raymarch_mode = if rng.random::<f32>() > 0.5 {
                (rng.random::<f32>() * 5.0 + 1.0).floor()
            } else {
                0.0
            };
            state.uniforms.raymarch_blend = rng.random::<f32>();
            state.uniforms.film_grain = if rng.random::<f32>() > 0.6 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.sharpen = if rng.random::<f32>() > 0.7 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.pixelate = if rng.random::<f32>() > 0.85 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.color_posterize = if rng.random::<f32>() > 0.8 {
                rng.random::<f32>() * 0.7
            } else {
                0.0
            };
            state.uniforms.radial_blur = if rng.random::<f32>() > 0.8 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.tunnel_speed = 0.5 + rng.random::<f32>() * 2.0;
            state.uniforms.fractal_iterations = (2.0 + rng.random::<f32>() * 4.0).floor();
            state.uniforms.glow_intensity = if rng.random::<f32>() > 0.5 {
                rng.random::<f32>() * 1.0
            } else {
                0.0
            };
            state.uniforms.screen_shake = if rng.random::<f32>() > 0.8 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.zoom_pulse = if rng.random::<f32>() > 0.7 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.speed_lines = if rng.random::<f32>() > 0.7 {
                rng.random::<f32>() * 0.6
            } else {
                0.0
            };
            state.uniforms.color_grade_mode = if rng.random::<f32>() > 0.5 {
                (rng.random::<f32>() * 6.0 + 1.0).floor()
            } else {
                0.0
            };
            state.uniforms.vhs_distortion = if rng.random::<f32>() > 0.8 {
                rng.random::<f32>() * 0.6
            } else {
                0.0
            };
            state.uniforms.lens_flare = if rng.random::<f32>() > 0.6 {
                rng.random::<f32>() * 0.8
            } else {
                0.0
            };
            state.uniforms.edge_glow = if rng.random::<f32>() > 0.7 {
                rng.random::<f32>() * 0.5
            } else {
                0.0
            };
            state.uniforms.saturation = 0.5 + rng.random::<f32>() * 1.0;
        }
    }

    fn next_phase(&self) -> DemoPhase {
        match self.current_phase {
            DemoPhase::TorusField => DemoPhase::CubeVortex,
            DemoPhase::CubeVortex => DemoPhase::SphereGrid,
            DemoPhase::SphereGrid => DemoPhase::PlasmaRings,
            DemoPhase::PlasmaRings => DemoPhase::Finale,
            DemoPhase::Finale => DemoPhase::TorusField,
        }
    }

    fn load_audio_from_path(&mut self, world: &mut World, path: &std::path::Path) {
        self.audio_status = "Loading audio...".to_string();
        self.audio_file_path = Some(path.to_path_buf());

        match decode_audio_file(path) {
            Ok((samples, sample_rate, audio_bytes)) => {
                self.audio_analyzer.load_samples(samples, sample_rate);

                let static_bytes: &'static [u8] = Box::leak(audio_bytes.into_boxed_slice());
                match load_sound_from_bytes(static_bytes) {
                    Ok(sound_data) => {
                        world.resources.audio.load_sound("demo_audio", sound_data);

                        if self.audio_entity.is_none() {
                            let entity = world.spawn_entities(AUDIO_SOURCE, 1)[0];
                            self.audio_entity = Some(entity);
                        }

                        if let Some(entity) = self.audio_entity {
                            world.core.set_audio_source(
                                entity,
                                AudioSource::new("demo_audio")
                                    .with_volume(1.0)
                                    .with_looping(false)
                                    .playing(),
                            );
                        }

                        self.audio_playing = true;
                        self.audio_start_time = self.global_time;
                        self.audio_status = format!(
                            "Playing: {}",
                            path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        );
                    }
                    Err(error) => {
                        self.audio_status = format!("Failed to load audio: {}", error);
                    }
                }
            }
            Err(error) => {
                self.audio_status = format!("Failed to decode audio: {}", error);
            }
        }
    }

    fn update_audio_reactive(&mut self) {
        if !self.audio_playing {
            return;
        }

        let latency_compensation = 0.06;
        let audio_time = (self.global_time - self.audio_start_time - latency_compensation).max(0.0);
        self.audio_analyzer.analyze_at_time(audio_time);

        let _sub_bass = self.audio_analyzer.smoothed_sub_bass * self.bass_sensitivity;
        let bass = self.audio_analyzer.smoothed_bass * self.bass_sensitivity;
        let low_mids = self.audio_analyzer.smoothed_low_mids * self.mids_sensitivity;
        let mids = self.audio_analyzer.smoothed_mids * self.mids_sensitivity;
        let high_mids = self.audio_analyzer.smoothed_high_mids * self.highs_sensitivity;
        let highs = self.audio_analyzer.smoothed_highs * self.highs_sensitivity;

        let kick = self.audio_analyzer.kick_decay;
        let snare = self.audio_analyzer.snare_decay;
        let hat = self.audio_analyzer.hat_decay;
        let onset = self.audio_analyzer.onset_decay;
        let energy = self.audio_analyzer.average_energy;
        let intensity = self.audio_analyzer.intensity;

        let brightness = self.audio_analyzer.smoothed_centroid;
        let noisiness = self.audio_analyzer.smoothed_flatness;
        let transient_ratio = self.audio_analyzer.transient_ratio;
        let low_transient = self.audio_analyzer.low_transient;
        let mid_transient = self.audio_analyzer.mid_transient;
        let high_transient = self.audio_analyzer.high_transient;

        let groove = self.audio_analyzer.groove_sync;
        let beat_phase = self.audio_analyzer.beat_phase;
        let beat_confidence = self.audio_analyzer.beat_confidence;

        let is_building = self.audio_analyzer.is_building;
        let is_dropping = self.audio_analyzer.is_dropping;
        let is_breakdown = self.audio_analyzer.is_breakdown;
        let drop_intensity = self.audio_analyzer.drop_intensity;
        let build_intensity = self.audio_analyzer.build_intensity;
        let breakdown_intensity = self.audio_analyzer.breakdown_intensity;
        let harmonic_change = self.audio_analyzer.harmonic_change;

        let song_progress = self.audio_analyzer.song_progress(audio_time);

        let energy_mult = if energy < 0.05 {
            0.2
        } else if energy < 0.15 {
            0.5 + energy * 2.0
        } else {
            1.0
        };

        let breakdown_dampen = if is_breakdown {
            1.0 - breakdown_intensity * 0.6
        } else {
            1.0
        };

        let drop_bloom_boost = if is_dropping {
            drop_intensity * 0.25
        } else {
            0.0
        };
        let build_bloom_dampen = if is_building {
            build_intensity * 0.1
        } else {
            0.0
        };
        let breakdown_bloom = if is_breakdown {
            -breakdown_intensity * 0.2
        } else {
            0.0
        };
        self.bloom_intensity = (0.2 + energy * 0.25 * energy_mult + kick * 0.15 + drop_bloom_boost
            - build_bloom_dampen
            + breakdown_bloom)
            .clamp(0.1, 0.85);

        if let Ok(mut state) = self.shared_state.write() {
            let intro = (song_progress * 2.0).min(1.0);
            let buildup = ((song_progress - 0.15) * 1.5).clamp(0.0, 1.0);
            let peak_section = ((song_progress - 0.3) * 1.43).clamp(0.0, 1.0);
            let climax = ((song_progress - 0.5) * 2.0).clamp(0.0, 1.0);
            let finale = ((song_progress - 0.75) * 4.0).clamp(0.0, 1.0);

            let beat_pulse = if beat_confidence > 0.3 {
                (beat_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5
            } else {
                0.5
            };

            let bd = breakdown_dampen;
            let em = energy_mult;

            let base_react = 0.15 + intro * 0.35 + buildup * 0.25 + peak_section * 0.25;

            state.uniforms.vignette = if is_breakdown {
                0.7 + breakdown_intensity * 0.2
            } else {
                (0.7 - intro * 0.15 - buildup * 0.15 - intensity * 0.1 * em - drop_intensity * 0.15)
                    .max(0.1)
            };
            state.uniforms.saturation = if is_breakdown {
                0.3 - breakdown_intensity * 0.15
            } else {
                0.3 + intro * 0.2 + buildup * 0.2 + intensity * 0.15 * em + brightness * 0.1
            };

            state.uniforms.glow_intensity = (intro * 0.02
                + bass * 0.1 * base_react
                + kick * 0.12 * base_react
                + groove * 0.06 * buildup)
                * bd
                * em;
            state.uniforms.plasma_intensity =
                (mids * 0.08 * base_react + noisiness * 0.12 * peak_section) * bd * em;
            state.uniforms.wave_distortion =
                (low_mids * 0.05 * base_react + harmonic_change * 0.1 * buildup) * bd * em;

            let transient_chroma = (low_transient * 0.2 + mid_transient * 0.1) * em;
            state.uniforms.chromatic_aberration = (kick * 0.12 * base_react
                + onset * 0.1 * base_react
                + transient_chroma * peak_section
                + drop_intensity * 0.15)
                * bd;
            state.uniforms.screen_shake = (kick * 0.04 * intro
                + kick * 0.08 * peak_section
                + kick * 0.12 * climax * (1.0 + drop_intensity * 0.8))
                * bd;
            state.uniforms.zoom_pulse =
                (kick * 0.03 * base_react + drop_intensity * 0.06 * climax) * bd;
            state.uniforms.radial_blur =
                (kick * 0.02 * base_react + low_transient * 0.03 * peak_section) * bd;

            state.uniforms.speed_lines = (highs * 0.08 * intro
                + highs * 0.15 * peak_section
                + onset * 0.15 * climax
                + hat * 0.1 * buildup
                + drop_intensity * 0.2)
                * bd
                * em;
            state.uniforms.edge_glow = (high_mids * 0.1 * base_react
                + snare * 0.18 * peak_section
                + brightness * 0.12 * buildup)
                * bd
                * em;

            let lull_decay = if energy < 0.1 {
                0.8
            } else if energy < 0.2 {
                0.88
            } else {
                0.94
            };

            if is_breakdown {
                state.uniforms.glitch_intensity =
                    (state.uniforms.glitch_intensity + breakdown_intensity * 0.02).min(0.2);
                state.uniforms.film_grain = 0.03 + breakdown_intensity * 0.03;
                state.uniforms.invert *= 0.85;
            } else if onset > 0.35 || mid_transient > 0.25 {
                let invert_amount = (onset * 0.1 + mid_transient * 0.06) * peak_section * em;
                state.uniforms.invert = invert_amount.min(0.15);
            } else {
                state.uniforms.invert *= lull_decay;
            }

            let glitch_trigger =
                (snare > 0.35 || (noisiness > 0.3 && high_transient > 0.2)) && !is_breakdown;
            if glitch_trigger && buildup > 0.15 {
                let glitch_add =
                    (snare * 0.1 + noisiness * high_transient * 0.12) * peak_section * em;
                state.uniforms.glitch_intensity =
                    (state.uniforms.glitch_intensity + glitch_add).min(0.3);
                if snare > 0.5 && peak_section > 0.25 {
                    state.uniforms.mirror_mode = 1.0;
                }
            } else if !is_breakdown {
                state.uniforms.glitch_intensity *= lull_decay;
                state.uniforms.mirror_mode *= lull_decay;
            }

            let trigger_raymarch = !is_breakdown
                && ((intensity > 1.0 && peak_section > 0.3 && kick > 0.4)
                    || (is_dropping && drop_intensity > 0.3)
                    || (transient_ratio > 1.4 && intensity > 0.9 && climax > 0.2));
            if trigger_raymarch {
                state.uniforms.raymarch_mode = if is_dropping { 2.0 } else { 1.0 };
                state.uniforms.raymarch_blend = (state.uniforms.raymarch_blend + 0.1).min(0.4);
                state.uniforms.tunnel_speed = 0.5 + bass * 1.2 + groove * 0.4;
            } else {
                state.uniforms.raymarch_blend *= lull_decay;
                if state.uniforms.raymarch_blend < 0.02 {
                    state.uniforms.raymarch_mode = 0.0;
                }
            }

            let target_kaleidoscope = if is_breakdown {
                0.0
            } else if (climax > 0.3 && onset > 0.35) || (is_dropping && kick > 0.4) {
                if is_dropping {
                    8.0
                } else if finale > 0.3 {
                    6.0
                } else {
                    4.0
                }
            } else if peak_section > 0.4 && kick > 0.5 && harmonic_change > 0.25 {
                4.0
            } else {
                0.0
            };
            if target_kaleidoscope > 0.0 {
                self.kaleidoscope_blend = (self.kaleidoscope_blend + 0.18).min(1.0);
            } else {
                self.kaleidoscope_blend *= lull_decay;
            }
            state.uniforms.kaleidoscope_segments = if self.kaleidoscope_blend > 0.25 {
                target_kaleidoscope
            } else {
                0.0
            };

            let hue_base = song_progress * 50.0;
            let hue_beat = if beat_confidence > 0.3 {
                beat_pulse * 12.0
            } else {
                0.0
            };
            let hue_harmonic = harmonic_change * 15.0;
            state.uniforms.hue_rotation =
                hue_base + bass * 12.0 * base_react * em + hue_beat + hue_harmonic;
            state.uniforms.color_shift =
                (highs * 0.15 * base_react + brightness * 0.2 * peak_section) * bd * em;

            if is_breakdown {
                state.uniforms.lens_flare *= 0.85;
                state.uniforms.vhs_distortion = breakdown_intensity * 0.06;
            } else if peak_section > 0.3 || is_dropping {
                let flare_amount =
                    (kick * 0.3 * peak_section + onset * 0.15 + drop_intensity * 0.15) * em;
                state.uniforms.lens_flare = flare_amount.min(0.6);
                state.uniforms.film_grain = 0.008 + onset * 0.025 + noisiness * 0.015;
                state.uniforms.vhs_distortion =
                    (snare * 0.06 + high_transient * 0.03) * climax * em;
            } else {
                state.uniforms.lens_flare *= lull_decay;
                state.uniforms.film_grain = noisiness * 0.005 * em;
                state.uniforms.vhs_distortion *= lull_decay;
            }

            let retro_trigger = finale > 0.5 && intensity > 1.0 && !is_breakdown;
            state.uniforms.crt_scanlines = if retro_trigger {
                0.1 + beat_pulse * 0.04
            } else {
                0.0
            };
            state.uniforms.pixelate = if finale > 0.7 && kick > 0.5 && !is_breakdown {
                2.0
            } else {
                0.0
            };

            if is_building {
                state.uniforms.fractal_iterations = 3.0 + build_intensity * 2.5;
            } else if is_breakdown {
                state.uniforms.fractal_iterations = 2.0;
            } else {
                state.uniforms.fractal_iterations = 4.0;
            }

            state.uniforms.warp_speed = if is_dropping && drop_intensity > 0.4 {
                (drop_intensity * 1.0 + bass * 0.4) * bd
            } else if climax > 0.5 && intensity > 1.1 {
                (intensity - 0.8) * 0.5 * bd
            } else {
                state.uniforms.warp_speed * lull_decay
            };

            state.uniforms.pulse_rings = if kick > 0.5 && peak_section > 0.3 {
                (kick * 0.7 + bass * 0.4) * peak_section * bd
            } else {
                state.uniforms.pulse_rings * lull_decay
            };

            state.uniforms.heat_distortion = if is_dropping {
                drop_intensity * 1.0 * bd
            } else if intensity > 1.2 && climax > 0.3 {
                (intensity - 0.9) * 0.4 * bd
            } else {
                state.uniforms.heat_distortion * lull_decay
            };

            state.uniforms.digital_rain = if is_breakdown && breakdown_intensity > 0.4 {
                breakdown_intensity * 0.5
            } else if finale > 0.6 && noisiness > 0.35 {
                noisiness * 0.4 * bd
            } else {
                state.uniforms.digital_rain * lull_decay
            };

            state.uniforms.strobe = if is_dropping && kick > 0.6 && drop_intensity > 0.5 {
                0.7 * bd
            } else if snare > 0.6 && climax > 0.4 {
                0.4 * bd
            } else {
                0.0
            };

            state.audio_data.waveform_intensity = intensity;
            state.audio_data.spectrum_intensity = intensity;
            state.audio_data.beat_pulse = beat_pulse;
            state.audio_data.bass_level = bass;
            state.audio_data.mids_level = mids;
            state.audio_data.highs_level = highs;
            state.audio_data.onset_flash = onset;
            state.audio_data.bpm = self.audio_analyzer.estimated_bpm;
            state.audio_data.beat_phase = beat_phase;
            state.audio_data.drop_intensity = drop_intensity;
            state.audio_data.spectral_centroid = brightness;
            state.audio_data.energy = energy;
            state.audio_data.time = audio_time;
            state.audio_data.visualizer_mode = self.visualizer_mode as f32;
            state.audio_data.visualizer_opacity = self.visualizer_opacity;
            state.audio_data.kick_decay = kick;

            let fft_size = self.audio_analyzer.fft_size();
            let sample_position = (audio_time * self.audio_analyzer.sample_rate() as f32) as usize;
            if sample_position + fft_size <= self.audio_analyzer.samples().len() {
                for waveform_index in 0..VISUALIZER_WAVEFORM_SIZE {
                    let source_index = waveform_index * (fft_size / VISUALIZER_WAVEFORM_SIZE);
                    state.audio_data.waveform[waveform_index] =
                        self.audio_analyzer.samples()[sample_position + source_index];
                }
            }

            let spectrum = self.audio_analyzer.prev_spectrum();
            for spectrum_index in 0..VISUALIZER_SPECTRUM_SIZE {
                let bin_a = spectrum_index * 2;
                let bin_b = bin_a + 1;
                state.audio_data.spectrum[spectrum_index] = if bin_b < spectrum.len() {
                    (spectrum[bin_a] + spectrum[bin_b]) * 0.5
                } else if bin_a < spectrum.len() {
                    spectrum[bin_a]
                } else {
                    0.0
                };
            }
        }
    }

    fn launch_firework(&mut self, world: &mut World, color: Vec3, particle_count: u32) {
        let mut rng = rand::rng();

        let spread = 80.0;
        let x_offset: f32 = rng.random_range(-spread..spread);
        let z_offset: f32 = rng.random_range(-40.0..40.0);

        let launch_pos = Vec3::new(x_offset, -20.0, z_offset - 60.0);
        let target_height: f32 = rng.random_range(40.0..80.0);

        let velocity = Vec3::new(
            rng.random_range(-5.0..5.0),
            rng.random_range(60.0..90.0),
            rng.random_range(-5.0..5.0),
        );

        let fuse_time = target_height / velocity.y;

        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let trail_emitter = ParticleEmitter::firework_shell(launch_pos, velocity);
        world.core.set_particle_emitter(entity, trail_emitter);

        self.firework_shells.push(FireworkShell {
            entity,
            position: launch_pos,
            velocity,
            fuse_time,
            color,
            particle_count,
        });
    }

    fn update_fireworks(&mut self, world: &mut World, delta_time: f32) {
        let mut explosions: Vec<(Vec3, Vec3, u32, Entity)> = Vec::new();

        for shell in self.firework_shells.iter_mut() {
            shell.fuse_time -= delta_time;
            shell.position += shell.velocity * delta_time;
            shell.velocity.y -= 15.0 * delta_time;

            if let Some(emitter) = world.core.get_particle_emitter_mut(shell.entity) {
                emitter.position = shell.position;
            }

            if shell.fuse_time <= 0.0 {
                explosions.push((
                    shell.position,
                    shell.color,
                    shell.particle_count,
                    shell.entity,
                ));
            }
        }

        for (pos, color, particle_count, entity) in explosions {
            let flash_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
            let flash_emitter = ParticleEmitter::flash_burst(pos);
            world.core.set_particle_emitter(flash_entity, flash_emitter);

            let explosion_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
            let emitter = ParticleEmitter::firework_explosion(pos, color, particle_count);
            world.core.set_particle_emitter(explosion_entity, emitter);

            let glitter_entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
            let glitter_emitter = ParticleEmitter::firework_glitter(pos, particle_count / 3);
            world
                .core
                .set_particle_emitter(glitter_entity, glitter_emitter);

            if let Some(emitter) = world.core.get_particle_emitter_mut(entity) {
                emitter.enabled = false;
            }
        }

        self.firework_shells.retain(|shell| shell.fuse_time > 0.0);
    }

    fn trigger_audio_fireworks(&mut self, world: &mut World) {
        if !self.audio_playing {
            return;
        }

        let kick = self.audio_analyzer.kick_decay;
        let drop_intensity = self.audio_analyzer.drop_intensity;
        let is_dropping = self.audio_analyzer.is_dropping;
        let intensity = self.audio_analyzer.intensity;

        let should_launch = (is_dropping && drop_intensity > 0.6 && kick > 0.5)
            || (intensity > 1.5 && kick > 0.7)
            || (self.audio_analyzer.onset_decay > 0.8 && intensity > 1.3);

        if should_launch && self.global_time > self.last_firework_time + self.firework_cooldown {
            let mut rng = rand::rng();
            let hue = rng.random::<f32>() * 360.0;
            let color = Self::hsv_to_rgb(hue, 0.9, 1.0);

            let particle_count = if is_dropping {
                rng.random_range(600..1000)
            } else {
                rng.random_range(300..600)
            };

            self.launch_firework(world, color, particle_count);

            if is_dropping && drop_intensity > 0.8 {
                for _ in 0..rng.random_range(2..5) {
                    let hue2 = rng.random::<f32>() * 360.0;
                    let color2 = Self::hsv_to_rgb(hue2, 0.9, 1.0);
                    self.launch_firework(world, color2, rng.random_range(400..700));
                }
            }

            self.last_firework_time = self.global_time;
        }
    }

    fn stop_audio(&mut self, world: &mut World) {
        if let Some(entity) = self.audio_entity
            && let Some(source) = world.core.get_audio_source_mut(entity)
        {
            source.playing = false;
        }
        self.audio_playing = false;
        self.audio_status = "Stopped".to_string();
    }

    fn draw_waveform_preview(&self, ui: &mut egui::Ui) {
        let audio_time = (self.global_time - self.audio_start_time).max(0.0);
        let sample_rate = self.audio_analyzer.sample_rate() as f32;
        let total_samples = self.audio_analyzer.samples().len();
        if total_samples == 0 {
            return;
        }

        let available_width = ui.available_width();
        let height = 60.0;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, height), egui::Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(10, 10, 20));

        let center_y = rect.center().y;
        let sample_center = (audio_time * sample_rate) as usize;
        let visible_samples = (sample_rate * 0.05) as usize;
        let half_visible = visible_samples / 2;

        let start = sample_center.saturating_sub(half_visible);
        let end = (sample_center + half_visible).min(total_samples);
        if start >= end {
            return;
        }

        let samples_per_pixel = ((end - start) as f32 / available_width).max(1.0);
        let pixel_count = ((end - start) as f32 / samples_per_pixel) as usize;

        let kick = self.audio_analyzer.kick_decay;
        let onset = self.audio_analyzer.onset_decay;
        let base_color = egui::Color32::from_rgb(
            (0.0 + kick * 200.0).min(255.0) as u8,
            (200.0 + onset * 55.0).min(255.0) as u8,
            255,
        );
        let glow_color = egui::Color32::from_rgba_premultiplied(0, 150, 255, 60);

        let mut points = Vec::with_capacity(pixel_count);
        let mut glow_points = Vec::with_capacity(pixel_count);

        for pixel_index in 0..pixel_count {
            let sample_index = start + (pixel_index as f32 * samples_per_pixel) as usize;
            if sample_index >= total_samples {
                break;
            }

            let sample_value = self.audio_analyzer.samples()[sample_index];
            let x = rect.left() + (pixel_index as f32 / pixel_count as f32) * available_width;
            let y = center_y - sample_value * (height * 0.45);
            points.push(egui::pos2(x, y));
            glow_points.push(egui::pos2(x, y));
        }

        if glow_points.len() >= 2 {
            painter.add(egui::Shape::line(
                glow_points,
                egui::Stroke::new(4.0, glow_color),
            ));
        }
        if points.len() >= 2 {
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(1.5, base_color),
            ));
        }

        let playhead_x = rect.left() + available_width * 0.5;
        painter.line_segment(
            [
                egui::pos2(playhead_x, rect.top()),
                egui::pos2(playhead_x, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 100)),
        );
    }

    fn update_objects(&mut self, world: &mut World, _delta_time: f32) {
        let time = self.global_time;

        let (
            audio_speed,
            kick,
            snare,
            hat,
            bass,
            mids,
            highs,
            intensity,
            groove,
            beat_phase,
            drop_intensity,
            is_dropping,
            brightness,
        ) = if self.audio_playing {
            let bpm_mult = (self.audio_analyzer.estimated_bpm / 120.0).clamp(0.6, 1.8);
            (
                bpm_mult,
                self.audio_analyzer.kick_decay,
                self.audio_analyzer.snare_decay,
                self.audio_analyzer.hat_decay,
                self.audio_analyzer.smoothed_bass,
                self.audio_analyzer.smoothed_mids,
                self.audio_analyzer.smoothed_highs,
                self.audio_analyzer.intensity,
                self.audio_analyzer.groove_sync,
                self.audio_analyzer.beat_phase,
                self.audio_analyzer.drop_intensity,
                self.audio_analyzer.is_dropping,
                self.audio_analyzer.smoothed_centroid,
            )
        } else {
            (
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, false, 0.5,
            )
        };

        let beat_pulse_factor = (beat_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        for (object_index, object) in self.objects.iter().enumerate() {
            let object_phase_offset = object_index as f32 * 0.1;

            let base_pulse =
                (time * self.color_cycle_speed * audio_speed + object.phase_offset).sin() * 0.5
                    + 0.5;
            let audio_pulse =
                kick * 0.4 + bass * 0.2 + highs * 0.15 + beat_pulse_factor * groove * 0.3;
            let pulse = base_pulse + audio_pulse;

            let kick_scale_boost = kick * 0.3;
            let highs_shimmer = highs * 0.1 * (time * 8.0 + object_phase_offset).sin().abs();
            let drop_scale = if is_dropping {
                drop_intensity * 0.2
            } else {
                0.0
            };
            let scale_pulse = 1.0
                + pulse * 0.15 * self.pulse_intensity
                + kick_scale_boost
                + drop_scale
                + highs_shimmer;

            let spin_speed_boost = 1.0 + intensity * 0.5 + snare * 0.8 + highs * 0.4;
            let spin_angle =
                time * object.spin_speed * self.rotation_speed * audio_speed * spin_speed_boost;
            let spin_rotation = nalgebra_glm::quat_angle_axis(spin_angle, &object.spin_axis);

            let orbit_speed_boost = 1.0 + groove * 0.3 + intensity * 0.2;
            let orbit_angle =
                time * object.orbit_speed * self.rotation_speed * audio_speed * orbit_speed_boost
                    + object.phase_offset;

            let kick_displacement = kick * 2.0;
            let snare_jitter = Vec3::new(
                (time * 17.3 + object_phase_offset).sin() * snare * 1.5,
                (time * 13.7 + object_phase_offset).cos() * snare * 1.0,
                (time * 11.1 + object_phase_offset).sin() * snare * 1.2,
            );
            let hat_sparkle = Vec3::new(
                (time * 31.0 + object_phase_offset).sin() * hat * 0.5,
                (time * 29.0 + object_phase_offset).cos() * hat * 0.5,
                (time * 23.0 + object_phase_offset).sin() * hat * 0.5,
            );

            let new_position = if object.orbit_radius > 0.0 {
                match self.current_phase {
                    DemoPhase::TorusField => {
                        let base_wave =
                            (time * 0.5 * audio_speed + object.phase_offset).sin() * 2.0;
                        let bass_wave = bass * 3.0 * (time * 2.0 + object.phase_offset).sin();
                        let wave = base_wave + bass_wave;
                        let radius_pulse = object.orbit_radius + kick_displacement;
                        Vec3::new(
                            orbit_angle.cos() * radius_pulse,
                            object.base_position.y + wave,
                            orbit_angle.sin() * radius_pulse,
                        ) + snare_jitter
                            + hat_sparkle
                    }
                    DemoPhase::CubeVortex => {
                        let spiral_progress = (object.base_position.y + 15.0) / 30.0;
                        let current_angle = orbit_angle;
                        let audio_radius_mod = bass * 3.0 * spiral_progress + kick_displacement;
                        let current_radius = object.orbit_radius
                            + (time * 0.3 * audio_speed).sin() * 2.0 * spiral_progress
                            + audio_radius_mod;
                        let height_mod = intensity * 2.0 * spiral_progress;
                        Vec3::new(
                            current_angle.cos() * current_radius,
                            object.base_position.y
                                + (time * 2.0 * audio_speed).sin() * spiral_progress
                                + height_mod,
                            current_angle.sin() * current_radius,
                        ) + snare_jitter
                    }
                    DemoPhase::SphereGrid => {
                        let wave_freq = audio_speed * 0.8;
                        let wave_amp = 1.5 + bass * 2.0 + kick * 1.5;
                        let wave_x =
                            (time * wave_freq + object.base_position.x * 0.1).sin() * wave_amp;
                        let wave_y = (time * wave_freq * 0.75 + object.base_position.y * 0.1).sin()
                            * wave_amp;
                        let wave_z = (time * wave_freq * 0.875 + object.base_position.z * 0.1)
                            .sin()
                            * wave_amp;
                        object.base_position + Vec3::new(wave_x, wave_y, wave_z) + hat_sparkle
                    }
                    DemoPhase::PlasmaRings => {
                        let rotation =
                            nalgebra_glm::quat_angle_axis(orbit_angle, &object.spin_axis);
                        let base_pos =
                            nalgebra_glm::quat_rotate_vec3(&rotation, &object.base_position);
                        let expansion = 1.0 + kick * 0.2 + mids * 0.1;
                        base_pos * expansion + snare_jitter
                    }
                    DemoPhase::Finale => {
                        let breathing =
                            1.0 + (time * 0.5 * audio_speed + object.phase_offset).sin() * 0.3;
                        let audio_breathing = 1.0 + kick * 0.15 + bass * 0.1 + drop_intensity * 0.2;
                        let radius = object.orbit_radius * breathing * audio_breathing;
                        object.base_position.normalize() * radius + snare_jitter + hat_sparkle
                    }
                }
            } else {
                let wave_speed = audio_speed * 0.8;
                let wave_amp = self.pulse_intensity * (1.0 + bass * 0.5 + kick * 0.3);
                let wave_x =
                    (time * wave_speed + object.base_position.x * 0.1).sin() * 1.5 * wave_amp;
                let wave_y = (time * wave_speed * 0.75 + object.base_position.y * 0.1).sin()
                    * 1.5
                    * wave_amp;
                let wave_z = (time * wave_speed * 0.875 + object.base_position.z * 0.1).sin()
                    * 1.5
                    * wave_amp;
                object.base_position + Vec3::new(wave_x, wave_y, wave_z) + snare_jitter
            };

            if let Some(transform) = world.core.get_local_transform_mut(object.entity) {
                transform.translation = new_position;
                transform.rotation = spin_rotation;
                transform.scale = Vec3::new(object.scale, object.scale, object.scale) * scale_pulse;
            }
            world.mark_local_transform_dirty(object.entity);

            let hue_speed = self.color_cycle_speed * audio_speed * (1.0 + intensity * 0.5);
            let hue_kick_shift = kick * 30.0;
            let hue_brightness_shift = brightness * 20.0;
            let new_hue = (object.color_offset
                + time * 30.0 * hue_speed
                + hue_kick_shift
                + hue_brightness_shift)
                % 360.0;

            let saturation = 0.85 + intensity * 0.1 + kick * 0.05;
            let value = 0.9 + kick * 0.1;
            let new_color = Self::hsv_to_rgb(new_hue, saturation.min(1.0), value.min(1.0));

            let base_emissive = 0.4 + pulse * self.pulse_intensity * 0.3;
            let audio_emissive = kick * 0.8 + bass * 0.4 + intensity * 0.3 + drop_intensity * 0.5;
            let emissive_strength = base_emissive + audio_emissive;

            if let Some(material_ref) = world.core.get_material_ref(object.entity)
                && let Some(material_index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_ref.name)
                    .copied()
                && let Some(Some(material)) = world
                    .resources
                    .material_registry
                    .registry
                    .entries
                    .get_mut(material_index as usize)
            {
                material.emissive_factor = [
                    new_color.x * emissive_strength,
                    new_color.y * emissive_strength,
                    new_color.z * emissive_strength,
                ];
            }
        }
    }

    fn update_lights(&mut self, world: &mut World, _delta_time: f32) {
        let time = self.global_time;

        let (
            audio_speed,
            kick,
            snare,
            bass,
            mids,
            highs,
            intensity,
            groove,
            beat_phase,
            drop_intensity,
            brightness,
        ) = if self.audio_playing {
            let bpm_mult = (self.audio_analyzer.estimated_bpm / 120.0).clamp(0.6, 1.8);
            (
                bpm_mult,
                self.audio_analyzer.kick_decay,
                self.audio_analyzer.snare_decay,
                self.audio_analyzer.smoothed_bass,
                self.audio_analyzer.smoothed_mids,
                self.audio_analyzer.smoothed_highs,
                self.audio_analyzer.intensity,
                self.audio_analyzer.groove_sync,
                self.audio_analyzer.beat_phase,
                self.audio_analyzer.drop_intensity,
                self.audio_analyzer.smoothed_centroid,
            )
        } else {
            (1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5)
        };

        let beat_pulse = (beat_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        for (light_index, light) in self.lights.iter().enumerate() {
            let light_phase = light_index as f32 * 0.3;

            let orbit_boost = 1.0 + groove * 0.2 + intensity * 0.15;
            let angle = time * light.orbit_speed * self.rotation_speed * audio_speed * orbit_boost
                + light.phase_offset;

            let base_height_wave = (time * 0.3 * audio_speed + light.phase_offset).sin() * 5.0;
            let beat_bob = beat_pulse * groove * 3.0;
            let kick_jump = kick * 4.0;
            let height_wave = base_height_wave + beat_bob + kick_jump;

            let radius_pulse = light.orbit_radius + kick * 3.0 + bass * 2.0;

            let snare_scatter = Vec3::new(
                (time * 19.0 + light_phase).sin() * snare * 2.0,
                (time * 17.0 + light_phase).cos() * snare * 1.5,
                (time * 13.0 + light_phase).sin() * snare * 2.0,
            );

            let new_position = Vec3::new(
                angle.cos() * radius_pulse,
                light.height_offset + height_wave,
                angle.sin() * radius_pulse,
            ) + snare_scatter;

            if let Some(transform) = world.core.get_local_transform_mut(light.entity) {
                transform.translation = new_position;
            }
            world.mark_local_transform_dirty(light.entity);

            if let Some(transform) = world.core.get_local_transform_mut(light.sphere_entity) {
                transform.translation = new_position;
            }
            world.mark_local_transform_dirty(light.sphere_entity);

            let hue_speed_mult = audio_speed * (1.0 + intensity * 0.3 + highs * 0.2);
            let kick_hue_jump = kick * 40.0;
            let mids_hue = mids * 15.0;
            let brightness_hue = brightness * 30.0;
            let hue_shift = (time * 20.0 * self.color_cycle_speed * hue_speed_mult
                + kick_hue_jump
                + mids_hue
                + brightness_hue)
                % 360.0;
            let base_hue = light.base_color.x * 120.0
                + light.base_color.y * 120.0
                + light.base_color.z * 120.0;
            let new_hue = (base_hue + hue_shift) % 360.0;

            let saturation = 0.9 + kick * 0.1 + mids * 0.05;
            let value = 0.95 + kick * 0.05 + highs * 0.03;
            let new_color = Self::hsv_to_rgb(new_hue, saturation.min(1.0), value.min(1.0));

            let base_intensity = 3.0
                + (time * 2.0 * audio_speed + light.phase_offset).sin()
                    * 2.0
                    * self.pulse_intensity;
            let audio_intensity = kick * 4.0
                + bass * 2.0
                + mids * 1.5
                + highs * 1.0
                + intensity * 2.0
                + drop_intensity * 3.0
                + beat_pulse * groove * 1.5;
            let intensity_pulse = base_intensity + audio_intensity;

            if let Some(light_component) = world.core.get_light_mut(light.entity) {
                light_component.color = new_color;
                light_component.intensity = intensity_pulse;
            }

            let emissive_boost = 5.0
                + kick * 3.0
                + mids * 1.0
                + highs * 0.8
                + intensity * 2.0
                + drop_intensity * 2.0;
            if let Some(material_ref) = world.core.get_material_ref(light.sphere_entity)
                && let Some(material_index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_ref.name)
                    .copied()
                && let Some(Some(material)) = world
                    .resources
                    .material_registry
                    .registry
                    .entries
                    .get_mut(material_index as usize)
            {
                material.emissive_factor = [
                    new_color.x * emissive_boost,
                    new_color.y * emissive_boost,
                    new_color.z * emissive_boost,
                ];
            }
        }
    }

    fn ease_in_out_sine(t: f32) -> f32 {
        -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0)
    }

    fn get_camera_keyframes_for_phase(phase: DemoPhase, duration: f32) -> Vec<CameraKeyframe> {
        match phase {
            DemoPhase::TorusField => vec![
                CameraKeyframe {
                    position: Vec3::new(50.0, 20.0, 0.0),
                    time: 0.0,
                },
                CameraKeyframe {
                    position: Vec3::new(30.0, 5.0, 30.0),
                    time: duration * 0.2,
                },
                CameraKeyframe {
                    position: Vec3::new(-10.0, 2.0, 15.0),
                    time: duration * 0.35,
                },
                CameraKeyframe {
                    position: Vec3::new(-40.0, 25.0, -20.0),
                    time: duration * 0.55,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, 40.0, 50.0),
                    time: duration * 0.75,
                },
                CameraKeyframe {
                    position: Vec3::new(50.0, 20.0, 0.0),
                    time: duration,
                },
            ],
            DemoPhase::CubeVortex => vec![
                CameraKeyframe {
                    position: Vec3::new(0.0, -10.0, 60.0),
                    time: 0.0,
                },
                CameraKeyframe {
                    position: Vec3::new(25.0, 0.0, 25.0),
                    time: duration * 0.15,
                },
                CameraKeyframe {
                    position: Vec3::new(15.0, 20.0, 15.0),
                    time: duration * 0.3,
                },
                CameraKeyframe {
                    position: Vec3::new(-5.0, 35.0, 5.0),
                    time: duration * 0.5,
                },
                CameraKeyframe {
                    position: Vec3::new(-30.0, 15.0, -30.0),
                    time: duration * 0.7,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, -5.0, -50.0),
                    time: duration * 0.85,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, -10.0, 60.0),
                    time: duration,
                },
            ],
            DemoPhase::SphereGrid => vec![
                CameraKeyframe {
                    position: Vec3::new(35.0, 35.0, 35.0),
                    time: 0.0,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, 0.0, 40.0),
                    time: duration * 0.2,
                },
                CameraKeyframe {
                    position: Vec3::new(-20.0, -20.0, 20.0),
                    time: duration * 0.4,
                },
                CameraKeyframe {
                    position: Vec3::new(-35.0, 10.0, -35.0),
                    time: duration * 0.6,
                },
                CameraKeyframe {
                    position: Vec3::new(10.0, -30.0, -20.0),
                    time: duration * 0.8,
                },
                CameraKeyframe {
                    position: Vec3::new(35.0, 35.0, 35.0),
                    time: duration,
                },
            ],
            DemoPhase::PlasmaRings => vec![
                CameraKeyframe {
                    position: Vec3::new(0.0, 0.0, 45.0),
                    time: 0.0,
                },
                CameraKeyframe {
                    position: Vec3::new(30.0, 20.0, 30.0),
                    time: duration * 0.15,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, 40.0, 0.1),
                    time: duration * 0.3,
                },
                CameraKeyframe {
                    position: Vec3::new(-30.0, 10.0, 30.0),
                    time: duration * 0.45,
                },
                CameraKeyframe {
                    position: Vec3::new(-20.0, -15.0, -20.0),
                    time: duration * 0.6,
                },
                CameraKeyframe {
                    position: Vec3::new(20.0, -10.0, -30.0),
                    time: duration * 0.75,
                },
                CameraKeyframe {
                    position: Vec3::new(40.0, 5.0, 0.1),
                    time: duration * 0.9,
                },
                CameraKeyframe {
                    position: Vec3::new(0.0, 0.0, 45.0),
                    time: duration,
                },
            ],
            DemoPhase::Finale => vec![
                CameraKeyframe {
                    position: Vec3::new(60.0, 30.0, 0.1),
                    time: 0.0,
                },
                CameraKeyframe {
                    position: Vec3::new(40.0, 10.0, 40.0),
                    time: duration * 0.1,
                },
                CameraKeyframe {
                    position: Vec3::new(0.1, 5.0, 55.0),
                    time: duration * 0.2,
                },
                CameraKeyframe {
                    position: Vec3::new(-35.0, 15.0, 35.0),
                    time: duration * 0.3,
                },
                CameraKeyframe {
                    position: Vec3::new(-50.0, 25.0, 0.1),
                    time: duration * 0.4,
                },
                CameraKeyframe {
                    position: Vec3::new(-30.0, 5.0, -40.0),
                    time: duration * 0.5,
                },
                CameraKeyframe {
                    position: Vec3::new(0.1, -10.0, -55.0),
                    time: duration * 0.6,
                },
                CameraKeyframe {
                    position: Vec3::new(35.0, 0.1, -35.0),
                    time: duration * 0.7,
                },
                CameraKeyframe {
                    position: Vec3::new(50.0, 20.0, 0.1),
                    time: duration * 0.8,
                },
                CameraKeyframe {
                    position: Vec3::new(30.0, 40.0, 30.0),
                    time: duration * 0.9,
                },
                CameraKeyframe {
                    position: Vec3::new(60.0, 30.0, 0.1),
                    time: duration,
                },
            ],
        }
    }

    fn interpolate_keyframes(keyframes: &[CameraKeyframe], time: f32) -> Vec3 {
        if keyframes.is_empty() {
            return Vec3::new(0.0, 10.0, 40.0);
        }

        if keyframes.len() == 1 {
            return keyframes[0].position;
        }

        let mut from_index = 0;
        for (index, keyframe) in keyframes.iter().enumerate() {
            if keyframe.time <= time {
                from_index = index;
            } else {
                break;
            }
        }

        let to_index = (from_index + 1).min(keyframes.len() - 1);

        if from_index == to_index {
            return keyframes[from_index].position;
        }

        let from = &keyframes[from_index];
        let to = &keyframes[to_index];

        let segment_duration = to.time - from.time;
        let segment_time = time - from.time;
        let t = if segment_duration > 0.0 {
            (segment_time / segment_duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let eased_t = Self::ease_in_out_sine(t);

        from.position.lerp(&to.position, eased_t)
    }

    fn update_camera(&mut self, world: &mut World, _delta_time: f32) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let audio_speed_mult = if self.audio_playing {
            let bpm = self.audio_analyzer.estimated_bpm;
            let base_mult = (bpm / 120.0).clamp(0.5, 2.0);
            let energy_mult = 1.0 + self.audio_analyzer.intensity * 0.3;
            base_mult * energy_mult
        } else {
            1.0
        };

        let kick_push = if self.audio_playing {
            self.audio_analyzer.kick_decay * 3.0
        } else {
            0.0
        };

        let drop_zoom = if self.audio_playing && self.audio_analyzer.is_dropping {
            self.audio_analyzer.drop_intensity * 8.0
        } else {
            0.0
        };

        let groove = if self.audio_playing {
            self.audio_analyzer.groove_sync
        } else {
            0.0
        };

        let beat_phase = if self.audio_playing {
            self.audio_analyzer.beat_phase
        } else {
            0.0
        };

        let base_position = match self.camera_mode {
            CameraMode::Manual => return,
            CameraMode::Orbit => {
                let time = self.global_time;
                let audio_angle_boost = groove * 0.3;
                let angle = time * self.camera_orbit_speed * audio_speed_mult + audio_angle_boost;

                let beat_bob = (beat_phase * std::f32::consts::TAU).sin() * 2.0 * groove;
                let height_wave = (time * 0.2 * audio_speed_mult).sin() * 5.0 + beat_bob;

                let dynamic_radius = self.camera_orbit_radius - kick_push - drop_zoom;

                Vec3::new(
                    angle.cos() * dynamic_radius,
                    self.camera_orbit_height + height_wave,
                    angle.sin() * dynamic_radius,
                )
            }
            CameraMode::Cinematic => {
                if self.camera_keyframes.is_empty() {
                    self.camera_keyframes = Self::get_camera_keyframes_for_phase(
                        self.current_phase,
                        self.phase_duration,
                    );
                }
                let base = Self::interpolate_keyframes(&self.camera_keyframes, self.phase_time);

                let beat_sway = if self.audio_playing {
                    let sway_x = (beat_phase * std::f32::consts::TAU).sin() * groove * 1.5;
                    let sway_y = (beat_phase * std::f32::consts::TAU * 2.0).cos() * groove * 0.8;
                    Vec3::new(sway_x, sway_y, 0.0)
                } else {
                    Vec3::zeros()
                };

                let intensity_push = if self.audio_playing {
                    let dir = base.normalize();
                    dir * (-kick_push - drop_zoom)
                } else {
                    Vec3::zeros()
                };

                base + beat_sway + intensity_push
            }
        };

        let snare_look_offset = if self.audio_playing {
            let snare = self.audio_analyzer.snare_decay;
            Vec3::new(
                (self.global_time * 7.3).sin() * snare * 2.0,
                (self.global_time * 5.7).cos() * snare * 1.5,
                (self.global_time * 4.1).sin() * snare * 1.0,
            )
        } else {
            Vec3::zeros()
        };

        let look_at_target = Vec3::zeros() + snare_look_offset;
        let view_matrix = nalgebra_glm::look_at(&base_position, &look_at_target, &Vec3::y());
        let rotation_matrix = view_matrix.fixed_view::<3, 3>(0, 0).transpose();
        let rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.translation = base_position;
            transform.rotation = rotation;
        }
        world.mark_local_transform_dirty(camera_entity);
    }
}

impl State for DemoSceneState {
    fn title(&self) -> &str {
        "Demo Scene"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.05, 1.0];
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = self.bloom_intensity;

        let camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA,
            1,
        )[0];

        world.core.set_local_transform(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 15.0, 40.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world
            .core
            .set_local_transform_dirty(camera, LocalTransformDirty);
        world
            .core
            .set_global_transform(camera, GlobalTransform::default());
        world.core.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(500.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        self.camera_entity = Some(camera);
        world.resources.active_camera = Some(camera);

        let title = spawn_3d_text_with_properties(
            world,
            self.current_phase.name(),
            Vec3::new(0.0, 25.0, 0.0),
            TextProperties {
                font_size: 80.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.03,
                outline_color: Vec4::new(0.2, 0.5, 1.0, 1.0),
                smoothing: 0.01,
                ..Default::default()
            },
        );
        self.title_text = Some(title);

        spawn_ui_text_with_properties(
            world,
            "DEMOSCENE\nWASD: Move | Mouse: Look | 1-5: Phase | Space: Next | ESC: Exit",
            Vec2::zeros(),
            TextProperties {
                font_size: 18.0,
                color: Vec4::new(1.0, 1.0, 1.0, 0.9),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        self.setup_torus_field(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.camera_mode == CameraMode::Manual {
            fly_camera_system(world);
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.global_time += delta_time;
        self.phase_time += delta_time;

        if self.auto_transition && self.phase_time >= self.phase_duration {
            let next = self.next_phase();
            self.switch_phase(world, next);
        }

        self.update_objects(world, delta_time);
        self.update_lights(world, delta_time);
        self.update_camera(world, delta_time);
        self.update_audio_reactive();
        self.trigger_audio_fireworks(world);
        self.update_fireworks(world, delta_time);

        world.resources.graphics.bloom_intensity = self.bloom_intensity;

        if let Some(title_entity) = self.title_text {
            let title_rotation = nalgebra_glm::quat_angle_axis(self.global_time * 0.1, &Vec3::y());
            if let Some(transform) = world.core.get_local_transform_mut(title_entity) {
                transform.rotation = title_rotation;
                let pulse = (self.global_time * 2.0).sin() * 0.5 + 0.5;
                transform.scale = Vec3::new(1.0, 1.0, 1.0) * (0.9 + pulse * 0.2);
            }
            world.mark_local_transform_dirty(title_entity);
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::Space => {
                let next = self.next_phase();
                self.switch_phase(world, next);
            }
            KeyCode::Digit1 => self.switch_phase(world, DemoPhase::TorusField),
            KeyCode::Digit2 => self.switch_phase(world, DemoPhase::CubeVortex),
            KeyCode::Digit3 => self.switch_phase(world, DemoPhase::SphereGrid),
            KeyCode::Digit4 => self.switch_phase(world, DemoPhase::PlasmaRings),
            KeyCode::Digit5 => self.switch_phase(world, DemoPhase::Finale),
            KeyCode::KeyC => {
                self.camera_mode = match self.camera_mode {
                    CameraMode::Cinematic => CameraMode::Orbit,
                    CameraMode::Orbit => CameraMode::Manual,
                    CameraMode::Manual => CameraMode::Cinematic,
                };
            }
            KeyCode::KeyA
                if world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::ControlLeft) =>
            {
                self.auto_transition = !self.auto_transition;
            }
            _ => {}
        }
    }

    fn on_dropped_file(&mut self, world: &mut World, path: &std::path::Path) {
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        match extension.as_str() {
            "mp3" | "wav" | "ogg" | "flac" | "mp4" | "m4a" | "aac" | "wma" | "webm" => {
                self.load_audio_from_path(world, path);
            }
            _ => {
                self.audio_status = format!("Unsupported format: .{}", extension);
            }
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_texture = graph
            .add_color_texture("postprocess")
            .format(wgpu::TextureFormat::Rgba8Unorm)
            .size(width, height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let postprocess_pass =
            passes::PostProcessPass::new(device, wgpu::TextureFormat::Rgba8Unorm, 1.0);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", postprocess_texture);

        let blit_pipeline = passes::BlitPass::create_pipeline(device, surface_format);
        let demoscene_pass = DemoscenePass::new(
            device,
            surface_format,
            blit_pipeline,
            Arc::clone(&self.shared_state),
        );
        graph
            .pass(Box::new(demoscene_pass))
            .read("input", postprocess_texture)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Demo Controls")
            .default_pos([10.0, 60.0])
            .default_width(320.0)
            .vscroll(true)
            .show(ui_context, |ui| {
                egui::CollapsingHeader::new("Audio Visualizer")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Open Audio File...").clicked()
                                && let Some(path) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Audio",
                                        &[
                                            "mp3", "wav", "ogg", "flac", "mp4", "m4a", "aac",
                                            "wma", "webm",
                                        ],
                                    )
                                    .pick_file()
                            {
                                self.load_audio_from_path(world, &path);
                            }
                            if self.audio_playing && ui.button("Stop").clicked() {
                                self.stop_audio(world);
                            }
                        });

                        ui.small("Or drag & drop an audio file onto the window");

                        if !self.audio_status.is_empty() {
                            ui.colored_label(egui::Color32::YELLOW, &self.audio_status);
                        }

                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut self.visualizer_opacity, 0.0..=1.0)
                                .text("Visualizer opacity"),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Mode:");
                            ui.selectable_value(&mut self.visualizer_mode, 0, "Off");
                            ui.selectable_value(&mut self.visualizer_mode, 1, "All");
                            ui.selectable_value(&mut self.visualizer_mode, 2, "Spectrum");
                            ui.selectable_value(&mut self.visualizer_mode, 3, "Waveform");
                        });

                        ui.separator();
                        if ui.button("Randomize All Effects").clicked() {
                            self.randomize_uniforms();
                        }

                        ui.add(
                            egui::Slider::new(&mut self.bass_sensitivity, 0.0..=3.0)
                                .text("Bass sensitivity"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.mids_sensitivity, 0.0..=3.0)
                                .text("Mids sensitivity"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.highs_sensitivity, 0.0..=3.0)
                                .text("Highs sensitivity"),
                        );

                        if self.audio_playing {
                            let audio_time = (self.global_time - self.audio_start_time).max(0.0);
                            let progress = self.audio_analyzer.song_progress(audio_time);
                            ui.add(
                                egui::ProgressBar::new(progress)
                                    .text(format!("{:.0}%", progress * 100.0)),
                            );

                            ui.separator();
                            self.draw_waveform_preview(ui);
                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Low:");
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_sub_bass.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_bass.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Mid:");
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_low_mids.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_mids.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("High:");
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_high_mids.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                                ui.add(
                                    egui::ProgressBar::new(
                                        self.audio_analyzer.smoothed_highs.min(1.0),
                                    )
                                    .desired_width(40.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Events:");
                                if self.audio_analyzer.kick_decay > 0.3 {
                                    ui.colored_label(egui::Color32::RED, "KICK");
                                }
                                if self.audio_analyzer.snare_decay > 0.3 {
                                    ui.colored_label(egui::Color32::YELLOW, "SNARE");
                                }
                                if self.audio_analyzer.onset_decay > 0.3 {
                                    ui.colored_label(egui::Color32::GREEN, "ONSET");
                                }
                            });
                        }
                    });

                ui.collapsing("Phase", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        ui.strong(self.current_phase.name());
                    });

                    let progress = self.phase_time / self.phase_duration;
                    ui.add(egui::ProgressBar::new(progress).show_percentage());

                    ui.checkbox(&mut self.auto_transition, "Auto-transition");

                    if self.auto_transition {
                        ui.add(
                            egui::Slider::new(&mut self.phase_duration, 5.0..=60.0)
                                .text("Duration (s)"),
                        );
                    }

                    ui.horizontal_wrapped(|ui| {
                        for phase in DemoPhase::all() {
                            if ui
                                .selectable_label(self.current_phase == *phase, phase.name())
                                .clicked()
                            {
                                self.switch_phase(world, *phase);
                            }
                        }
                    });
                });

                ui.collapsing("Scene", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.color_cycle_speed, 0.0..=3.0)
                            .text("Color cycle"),
                    );
                    ui.add(egui::Slider::new(&mut self.rotation_speed, 0.0..=3.0).text("Rotation"));
                    ui.add(egui::Slider::new(&mut self.pulse_intensity, 0.0..=2.0).text("Pulse"));
                    ui.add(egui::Slider::new(&mut self.bloom_intensity, 0.0..=2.0).text("Bloom"));

                    ui.separator();

                    let mut object_count_slider = self.object_count;
                    if ui
                        .add(egui::Slider::new(&mut object_count_slider, 16..=256).text("Objects"))
                        .changed()
                        && object_count_slider != self.object_count
                    {
                        self.object_count = object_count_slider;
                        self.switch_phase(world, self.current_phase);
                    }

                    let mut light_count_slider = self.light_count;
                    if ui
                        .add(egui::Slider::new(&mut light_count_slider, 4..=24).text("Lights"))
                        .changed()
                        && light_count_slider != self.light_count
                    {
                        self.light_count = light_count_slider;
                        self.switch_phase(world, self.current_phase);
                    }
                });

                ui.collapsing("Camera", |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.camera_mode,
                            CameraMode::Cinematic,
                            "Cinematic",
                        );
                        ui.selectable_value(&mut self.camera_mode, CameraMode::Orbit, "Orbit");
                        ui.selectable_value(&mut self.camera_mode, CameraMode::Manual, "Manual");
                    });

                    match self.camera_mode {
                        CameraMode::Cinematic => {
                            let keyframe_count = self.camera_keyframes.len();
                            let current_keyframe = self
                                .camera_keyframes
                                .iter()
                                .rposition(|k| k.time <= self.phase_time)
                                .map(|i| i + 1)
                                .unwrap_or(1);
                            ui.label(format!(
                                "Keyframe: {} / {}",
                                current_keyframe, keyframe_count
                            ));
                        }
                        CameraMode::Orbit => {
                            ui.add(
                                egui::Slider::new(&mut self.camera_orbit_speed, 0.0..=1.0)
                                    .text("Speed"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.camera_orbit_radius, 20.0..=80.0)
                                    .text("Radius"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.camera_orbit_height, 0.0..=40.0)
                                    .text("Height"),
                            );
                        }
                        CameraMode::Manual => {
                            ui.label("WASD + mouse");
                        }
                    }
                });

                ui.collapsing("Shader FX", |ui| {
                    let mut shader_enabled =
                        self.shared_state.read().map(|s| s.enabled).unwrap_or(true);
                    if ui.checkbox(&mut shader_enabled, "Enable").changed()
                        && let Ok(mut state) = self.shared_state.write()
                    {
                        state.enabled = shader_enabled;
                    }

                    if shader_enabled {
                        ui.horizontal(|ui| {
                            if ui.button("Randomize").clicked() {
                                self.randomize_shader_settings();
                            }
                            if ui.button("Reset").clicked()
                                && let Ok(mut state) = self.shared_state.write()
                            {
                                state.uniforms = DemosceneUniforms::default();
                            }
                        });

                        if let Ok(mut state) = self.shared_state.write() {
                            ui.collapsing("Raymarching", |ui| {
                                let modes = [
                                    "Off",
                                    "Tunnel",
                                    "Fractal",
                                    "Mandelbulb",
                                    "Vortex",
                                    "Geometric",
                                ];
                                let current = state.uniforms.raymarch_mode as usize;
                                ui.horizontal_wrapped(|ui| {
                                    for (i, name) in modes.iter().enumerate() {
                                        if ui.selectable_label(current == i, *name).clicked() {
                                            state.uniforms.raymarch_mode = i as f32;
                                        }
                                    }
                                });
                                if state.uniforms.raymarch_mode > 0.5 {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut state.uniforms.raymarch_blend,
                                            0.0..=1.0,
                                        )
                                        .text("Blend"),
                                    );
                                    ui.add(
                                        egui::Slider::new(
                                            &mut state.uniforms.tunnel_speed,
                                            0.0..=5.0,
                                        )
                                        .text("Speed"),
                                    );
                                    ui.add(
                                        egui::Slider::new(
                                            &mut state.uniforms.fractal_iterations,
                                            2.0..=8.0,
                                        )
                                        .text("Iterations"),
                                    );
                                }
                            });

                            ui.collapsing("Distortion", |ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.chromatic_aberration,
                                        0.0..=2.0,
                                    )
                                    .text("Chromatic"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.wave_distortion,
                                        0.0..=2.0,
                                    )
                                    .text("Wave"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.radial_blur, 0.0..=1.0)
                                        .text("Radial blur"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.kaleidoscope_segments,
                                        0.0..=12.0,
                                    )
                                    .text("Kaleidoscope"),
                                );
                                let mut mirror = state.uniforms.mirror_mode > 0.5;
                                if ui.checkbox(&mut mirror, "Mirror").changed() {
                                    state.uniforms.mirror_mode = if mirror { 1.0 } else { 0.0 };
                                }
                            });

                            ui.collapsing("Color", |ui| {
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.color_shift, 0.0..=2.0)
                                        .text("Shift"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.plasma_intensity,
                                        0.0..=1.0,
                                    )
                                    .text("Plasma"),
                                );
                                ui.horizontal(|ui| {
                                    ui.add_enabled(
                                        !state.animate_hue,
                                        egui::Slider::new(
                                            &mut state.uniforms.hue_rotation,
                                            0.0..=1.0,
                                        )
                                        .text("Hue"),
                                    );
                                    ui.checkbox(&mut state.animate_hue, "Auto");
                                });
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.color_posterize,
                                        0.0..=1.0,
                                    )
                                    .text("Posterize"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.saturation, 0.0..=2.0)
                                        .text("Saturation"),
                                );
                                let mut invert = state.uniforms.invert > 0.5;
                                if ui.checkbox(&mut invert, "Invert").changed() {
                                    state.uniforms.invert = if invert { 1.0 } else { 0.0 };
                                }
                            });

                            ui.collapsing("Color Grading", |ui| {
                                let grades = [
                                    "Off",
                                    "Cyberpunk",
                                    "Vaporwave",
                                    "Mono",
                                    "Sepia",
                                    "Matrix",
                                    "Inferno",
                                ];
                                let current = state.uniforms.color_grade_mode as usize;
                                ui.horizontal_wrapped(|ui| {
                                    for (i, name) in grades.iter().enumerate() {
                                        if ui.selectable_label(current == i, *name).clicked() {
                                            state.uniforms.color_grade_mode = i as f32;
                                        }
                                    }
                                });
                            });

                            ui.collapsing("Effects", |ui| {
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.vignette, 0.0..=2.0)
                                        .text("Vignette"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.glow_intensity,
                                        0.0..=2.0,
                                    )
                                    .text("Glow"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.glitch_intensity,
                                        0.0..=1.0,
                                    )
                                    .text("Glitch"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.film_grain, 0.0..=1.0)
                                        .text("Film grain"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.sharpen, 0.0..=1.0)
                                        .text("Sharpen"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.lens_flare, 0.0..=1.0)
                                        .text("Lens flare"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.edge_glow, 0.0..=1.0)
                                        .text("Edge glow"),
                                );
                            });

                            ui.collapsing("Motion", |ui| {
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.screen_shake, 0.0..=1.0)
                                        .text("Shake"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.zoom_pulse, 0.0..=1.0)
                                        .text("Zoom pulse"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.speed_lines, 0.0..=1.0)
                                        .text("Speed lines"),
                                );
                            });

                            ui.collapsing("Retro", |ui| {
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.crt_scanlines, 0.0..=1.0)
                                        .text("CRT"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.uniforms.pixelate, 0.0..=1.0)
                                        .text("Pixelate"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.uniforms.vhs_distortion,
                                        0.0..=1.0,
                                    )
                                    .text("VHS"),
                                );
                            });
                        }
                    }
                });

                ui.collapsing("Stats", |ui| {
                    ui.label(format!("Objects: {}", self.objects.len()));
                    ui.label(format!("Lights: {}", self.lights.len()));
                    ui.label(format!("Chrome spheres: {}", self.chrome_spheres.len()));
                    ui.label(format!(
                        "FPS: {:.0}",
                        world.resources.window.timing.frames_per_second
                    ));
                });
            });

        if ui_context.input(|i| i.key_pressed(egui::Key::R)) {
            self.randomize_shader_settings();
        }
    }
}
