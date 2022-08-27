mod buffer_state;
mod camera_state;
mod surface_state;
mod texture_state;

use buffer_state::BufferState;
use camera_state::CameraState;
use surface_state::SurfaceState;
use texture_state::TextureState;
use crate::tasks::Task;
use crate::tasks::TaskResult;
use crate::texture;
use crate::thread_task_manager::ThreadTaskManager;
use fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER;
use crate::camera;
use crate::voxels::world::World;
use wgpu::util::DeviceExt;
use fundamentals::world_position::WorldPosition;
use crate::voxels::mesh::Mesh;
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
    pub vertex_gpu_data: Arc<RwLock<VertexGPUData>>
}

pub struct State {
    pub surface_state: SurfaceState,
    pub queue: wgpu::Queue,
    pub render_pipeline_regular: wgpu::RenderPipeline,
    pub render_pipeline_wireframe: wgpu::RenderPipeline,
    pub camera_state: CameraState,
    pub texture_state: TextureState,
    pub buffer_state: BufferState,
    pub mouse_pressed: bool,
    pub render_wireframe: bool,
    pub chunk_positions_around_player: [WorldPosition; NUMBER_OF_CHUNKS_AROUND_PLAYER as usize],
    pub chunk_pos_to_index: std::collections::HashMap<WorldPosition, u32>,
    pub thread_data: ThreadData,
    pub thread_task_manager: ThreadTaskManager
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

        let camera = camera::Camera::new((0.0,0.0,0.0), cgmath::Deg(0.0), cgmath::Deg(0.0));
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

        let chunk_positions_around_player = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera.position));

        let chunk_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&chunk_positions_around_player),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let chunk_index_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                }
            ],
            label: Some("chunk_offset_bind_group_layout")
        });

        let chunk_index_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &chunk_index_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: chunk_index_buffer.as_entire_binding()
                }
            ],
            label: Some("chunk_index_bind_group")
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
                    &chunk_index_bind_group_layout
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
        for pos in chunk_positions_around_player {
            thread_task_manager.push_task(Task::GenerateChunk { chunk_position: pos, world: world.clone() });
        }

        let vertex_gpu_data = Arc::new(RwLock::new(Mesh::new().get_gpu_data()));

        let vertex_buffers = vertex_gpu_data.read().unwrap().generate_vertex_buffers(&device);

        let index_buffers = vertex_gpu_data.read().unwrap().generate_index_buffers(&device);

        let index_buffers_lengths = vertex_gpu_data.read().unwrap().generate_index_buffer_lengths();

        let mut chunk_pos_to_index = std::collections::HashMap::new();

        for i in 0..chunk_positions_around_player.len() {
            chunk_pos_to_index.insert(chunk_positions_around_player[i], i as u32);
        }

        Self {
            surface_state: SurfaceState {
                surface,
                device,
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
            buffer_state: BufferState {
                vertex_buffers,
                index_buffers,
                index_buffers_lengths,
                chunk_index_buffer,
                chunk_index_bind_group,
            },
            queue,
            render_pipeline_regular,
            render_pipeline_wireframe,
            mouse_pressed: false,
            render_wireframe: false,
            chunk_positions_around_player,
            chunk_pos_to_index,
            thread_data: ThreadData { world, vertex_gpu_data },
            thread_task_manager
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_state.size = new_size;
            self.surface_state.config.width = new_size.width;
            self.surface_state.config.height = new_size.height;
            self.surface_state.surface.configure(&self.surface_state.device, &self.surface_state.config);
            self.texture_state.depth_texture = texture::Texture::create_depth_texture(&self.surface_state.device, &self.surface_state.config, "depth_texture");
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
                if *key == VirtualKeyCode::LControl && *state == ElementState::Pressed {
                    self.render_wireframe = !self.render_wireframe;
                }
                self.camera_state.camera_controller.process_keyboard(*key, *state)
            }
                ,
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_state.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
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
                    self.thread_task_manager.push_task(Task::GenerateChunkMesh { chunk_position, world: self.thread_data.world.clone(), chunk_index: *self.chunk_pos_to_index.get(&chunk_position).unwrap(), vertex_gpu_data: self.thread_data.vertex_gpu_data.clone() });
                    
                },
                TaskResult::GenerateChunkMesh { } => {
                    println!("Generated mesh {}!", meshes_generated);
                    meshes_generated += 1;
                }
            }
        }

        if meshes_generated > 0 {
            self.buffer_state.vertex_buffers = self.thread_data.vertex_gpu_data.read().unwrap().generate_vertex_buffers(&self.surface_state.device);
            self.buffer_state.index_buffers = self.thread_data.vertex_gpu_data.read().unwrap().generate_index_buffers(&self.surface_state.device);
            self.buffer_state.index_buffers_lengths = self.thread_data.vertex_gpu_data.read().unwrap().generate_index_buffer_lengths();
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .surface_state.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
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
            render_pass.set_bind_group(2, &self.buffer_state.chunk_index_bind_group, &[]);

            let ycos = self.camera_state.camera.yaw.0.cos();
            let ysin = self.camera_state.camera.yaw.0.sin();
            let psin = self.camera_state.camera.pitch.0.sin();

            //front
            if ycos > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[0].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[0].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[0], 0, 0..1);
            }
            //back
            if ycos < SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[1].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[1].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[1], 0, 0..1);
            }
            //left
            if ysin > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[2].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[2].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[2], 0, 0..1);
            }
            //right
            if ysin < SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[3].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[3].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[3], 0, 0..1);
            }
            //top
            if psin < SQRT_2_DIV_2  {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[4].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[4].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[4], 0, 0..1);
            }
            //bottom
            if psin > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.buffer_state.vertex_buffers[5].slice(..));
                render_pass.set_index_buffer(self.buffer_state.index_buffers[5].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.buffer_state.index_buffers_lengths[5], 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
