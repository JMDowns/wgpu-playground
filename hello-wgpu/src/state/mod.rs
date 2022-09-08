mod buffer_state;
mod camera_state;
mod surface_state;
mod texture_state;

use camera_state::CameraState;
use cgmath::Point3;
use futures_intrusive::buffer;
use surface_state::SurfaceState;
use texture_state::TextureState;
use crate::tasks::Task;
use crate::tasks::TaskResult;
use crate::texture;
use crate::thread_task_manager::ThreadTaskManager;
use crate::camera;
use crate::voxels::world::World;
use wgpu::util::DeviceExt;
use fundamentals::world_position::WorldPosition;
use std::sync::Arc;
use std::sync::RwLock;
use derivables::vertex::Vertex;
use crate::gpu_data::vertex_gpu_data::VertexGPUData;

use winit::{
    event::*,
    window::Window,
};

const SQRT_2_DIV_2: f32 = 0.7071;
const NEG_SQRT_2_DIV_2: f32 = -0.7071;

pub struct ThreadData {
    pub world: Arc<RwLock<World>>,
    pub vertex_gpu_data: Arc<RwLock<VertexGPUData>>,
    pub device: Arc<RwLock<wgpu::Device>>,
}

pub struct InputState {
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub mouse_delta_x: f64,
    pub mouse_delta_y: f64
}

pub struct State {
    pub surface_state: SurfaceState,
    pub queue: wgpu::Queue,
    pub render_pipeline_regular: wgpu::RenderPipeline,
    pub render_pipeline_wireframe: wgpu::RenderPipeline,
    pub camera_state: CameraState,
    pub texture_state: TextureState,
    pub mouse_pressed: bool,
    pub render_wireframe: bool,
    pub thread_data: ThreadData,
    pub thread_task_manager: ThreadTaskManager,
    pub calculate_frustum: bool,
    pub chunk_positions_to_load: Vec<WorldPosition>,
    pub input_state: InputState,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_input_buffer: wgpu::Buffer,
    pub compute_output_buffer: wgpu::Buffer,
    pub compute_staging_vec: Vec<u32>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface.get_supported_formats(&adapter).first().unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let screen_color = wgpu::Color {
            r: 0.0,
            g: 0.5,
            b: 0.5,
            a: 0.0,
        };

        let diffuse_bytes = include_bytes!("../atlas.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "atlas.png").unwrap();

        let texture_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { 
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let camera_position = Point3::new(0.0, 0.0, 0.0);
        let camera = camera::Camera::new(camera_position, cgmath::Deg(0.0), cgmath::Deg(0.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group")
        });

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let vertex_gpu_data = Arc::new(RwLock::new(VertexGPUData::new(&camera, &device)));

        let chunks_around_player = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera_position));

        let compute_input_buffer = device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
            label: Some("Compute Input Buffer"),
            contents: bytemuck::cast_slice(&chunks_around_player),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE
        });

        let compute_output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Output Buffer"),
            size: std::mem::size_of::<u32>() as u64 * fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../frustum_compute.wgsl").into()),
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
            ],
            label: Some("Compute Bind Group Layout")
        });

        let compute_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: compute_output_buffer.as_entire_binding(),
                }
            ]
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[
                &compute_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main"
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                    &vertex_gpu_data.read().unwrap().chunk_index_bind_group_layout
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_regular =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
                ),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let render_pipeline_wireframe =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Wireframe Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Line,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
                ),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let world = Arc::new(RwLock::new(World::new()));

        let mut thread_task_manager = ThreadTaskManager::new();
        for pos in vertex_gpu_data.read().unwrap().chunk_index_array.iter().rev() {
            thread_task_manager.push_task(Task::GenerateChunk { chunk_position: *pos, world: world.clone() });
        }

        Self {
            surface_state: SurfaceState {
                surface,
                config,
                size,
                screen_color,
            },
            texture_state: TextureState {
                diffuse_bind_group,
                diffuse_texture,
                depth_texture,
            },
            camera_state: CameraState {
                camera,
                camera_uniform,
                camera_buffer,
                camera_bind_group,
                projection,
                camera_controller,
            },
            queue,
            render_pipeline_regular,
            render_pipeline_wireframe,
            mouse_pressed: false,
            render_wireframe: false,
            thread_data: ThreadData { world, vertex_gpu_data, device: Arc::new(RwLock::new(device)) },
            thread_task_manager,
            calculate_frustum: true,
            chunk_positions_to_load: Vec::new(),
            input_state: InputState {
                is_up_pressed: false,
                is_down_pressed: false,
                is_left_pressed: false,
                is_right_pressed: false,
                is_forward_pressed: false,
                is_backward_pressed: false,
                mouse_delta_x: 0.0,
                mouse_delta_y: 0.0
            },
            compute_bind_group,
            compute_input_buffer,
            compute_output_buffer,
            compute_staging_vec: vec![0; fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize],
            compute_pipeline
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_state.size = new_size;
            self.surface_state.config.width = new_size.width;
            self.surface_state.config.height = new_size.height;
            self.surface_state.surface.configure(&self.thread_data.device.read().unwrap(), &self.surface_state.config);
            self.texture_state.depth_texture = texture::Texture::create_depth_texture(&self.thread_data.device.read().unwrap(), &self.surface_state.config, "depth_texture");
            self.camera_state.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => 
            {
                let mut input_movement_character = false;
                if key == &fundamentals::consts::FORWARD_KEY {
                    self.input_state.is_forward_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::BACKWARD_KEY {
                    self.input_state.is_backward_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::LEFT_KEY {
                    self.input_state.is_left_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::RIGHT_KEY {
                    self.input_state.is_right_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::UP_KEY {
                    self.input_state.is_up_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                if key == &fundamentals::consts::DOWN_KEY {
                    self.input_state.is_down_pressed = state == &ElementState::Pressed;
                    input_movement_character = true;
                }
                input_movement_character
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_state.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                if *state == ElementState::Pressed {
                    self.mouse_pressed = true;
                } else {
                    self.mouse_pressed = false;
                    self.input_state.mouse_delta_x = 0.0;
                    self.input_state.mouse_delta_y = 0.0;
                }
                true
            }
            _ => false,
        }
    }

    pub fn process_input(&mut self) {
        let sensitivity = 1.0;
        self.calculate_frustum = self.camera_state.camera_controller.process_keyboard(&self.input_state) || self.calculate_frustum;
        self.calculate_frustum = self.camera_state.camera_controller.process_mouse(&mut self.input_state, sensitivity) || self.calculate_frustum;
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.camera_state.camera_controller.update_camera(&mut self.camera_state.camera, dt);
        self.camera_state.camera_uniform.update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.write_buffer(&self.camera_state.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
    }

    pub fn process_tasks(&mut self) {
        let mut task_results = self.thread_task_manager.process_tasks();

        let mut chunks_generated = 0;
        let mut meshes_generated = 0;

        for task_result in task_results.drain(..) {
            match task_result {
                TaskResult::Requeue { task } => self.thread_task_manager.push_task(task),
                TaskResult::GenerateChunk { chunk_position } => {
                    println!("Generated chunk {}!", chunks_generated);
                    chunks_generated += 1;
                    self.thread_task_manager.push_task(Task::GenerateChunkMesh { chunk_position, world: self.thread_data.world.clone(), vertex_gpu_data: self.thread_data.vertex_gpu_data.clone(), device: self.thread_data.device.clone() });
                    
                },
                TaskResult::GenerateChunkMesh { } => {
                    println!("Generated mesh {}!", meshes_generated);
                    self.calculate_frustum = true;
                    meshes_generated += 1;
                }
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .thread_data.device.read().unwrap()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let vertex_gpu_data = self.thread_data.vertex_gpu_data.read().unwrap();

        let compute_staging_buffer = self.thread_data.device.read().unwrap().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Staging Buffer"),
            size: std::mem::size_of::<u32>() as u64 * fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            compute_pass.dispatch_workgroups((fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as f32 / 256.0).ceil() as u32, 1, 1);
        }


        encoder.copy_buffer_to_buffer(&self.compute_output_buffer, 0, &compute_staging_buffer, 0, std::mem::size_of::<u32>() as u64 * fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as u64);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.surface_state.screen_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.texture_state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if self.render_wireframe {
                render_pass.set_pipeline(&self.render_pipeline_wireframe);
            } else {
                render_pass.set_pipeline(&self.render_pipeline_regular);
            }

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_state.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &vertex_gpu_data.chunk_index_bind_group, &[]);

            let ycos = self.camera_state.camera.yaw.0.cos();
            let ysin = self.camera_state.camera.yaw.0.sin();
            let psin = self.camera_state.camera.pitch.0.sin();

            //Front, Back, Left, Right, Top, Bottom
            let lhs_comp_arr = [ycos, ycos, ysin, ysin, psin, psin];
            let is_comp_lt = [false, true, false, true, true, false];
            let angles = [NEG_SQRT_2_DIV_2, SQRT_2_DIV_2, NEG_SQRT_2_DIV_2, SQRT_2_DIV_2, SQRT_2_DIV_2, NEG_SQRT_2_DIV_2];

            for chunk_pos_index in 0..fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize {
                if self.compute_staging_vec[chunk_pos_index] == 0 {
                    continue;
                }
                let gpu_data_result = vertex_gpu_data.get_buffers_at_position(&vertex_gpu_data.chunk_index_array[chunk_pos_index]);
                match gpu_data_result {
                    Some(gpu_data) => {
                        for i in 0..6 {
                            if is_comp_lt[i] {
                                if lhs_comp_arr[i] < angles[i] {
                                    render_pass.set_vertex_buffer(0, gpu_data[i].0.slice(..));
                                    render_pass.set_index_buffer(gpu_data[i].1.slice(..), wgpu::IndexFormat::Uint32);
                                    render_pass.draw_indexed(0..gpu_data[i].2, 0, 0..1);
                                }
                            } else {
                                if lhs_comp_arr[i] > angles[i] {
                                    render_pass.set_vertex_buffer(0, gpu_data[i].0.slice(..));
                                    render_pass.set_index_buffer(gpu_data[i].1.slice(..), wgpu::IndexFormat::Uint32);
                                    render_pass.draw_indexed(0..gpu_data[i].2, 0, 0..1);
                                }
                            }
                        }
                    },
                    None => {}
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        if self.calculate_frustum {
            let compute_output_buffer_slice = compute_staging_buffer.slice(..);
            let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
            compute_output_buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            // Poll the device in a blocking manner so that our future resolves.
            // In an actual application, `device.poll(...)` should
            // be called in an event loop or on another thread.  
            self.thread_data.device.read().unwrap().poll(wgpu::Maintain::Wait);
            if let Some(Ok(())) = pollster::block_on(receiver.receive()) {
                let data = compute_output_buffer_slice.get_mapped_range();
                let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
                self.compute_staging_vec = result;

                drop(data);
                compute_staging_buffer.unmap();
            }

            self.calculate_frustum = false;
        }
        

        Ok(())
    }
}

