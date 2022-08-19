use crate::tasks::Task;
use crate::tasks::TaskResult;
use crate::tasks::tasks_processors::generate_chunk_mesh_processor::GenerateChunkMeshProcessor;
use crate::tasks::tasks_processors::generate_chunk_processor::GenerateChunkProcessor;
use crate::texture;
use crate::voxels::chunk;
use crossbeam::sync::WaitGroup;
use fundamentals::consts;
use fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER;
use crate::camera;
use crate::voxels::world::World;
use wgpu::util::DeviceExt;
use fundamentals::world_position::WorldPosition;
use crate::voxels::mesh::Mesh;
use std::collections::VecDeque;
use derivables::vertex::Vertex;
use crate::gpu_data::vertex_gpu_data::VertexGPUData;

use winit::{
    event::*,
    window::Window,
};

const SQRT_2_DIV_2: f32 = 0.7071;
const NEG_SQRT_2_DIV_2: f32 = -0.7071;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub screen_color: wgpu::Color,
    pub render_pipeline_regular: wgpu::RenderPipeline,
    pub render_pipeline_wireframe: wgpu::RenderPipeline,
    pub mesh: Mesh,
    pub depth_texture: texture::Texture,
    pub camera: camera::Camera,
    pub camera_uniform: camera::CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    pub mouse_pressed: bool,
    pub render_wireframe: bool,
    pub world: World,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: texture::Texture,
    pub task_queue: VecDeque<Task>,
    pub vertex_buffers: [wgpu::Buffer; 6],
    pub index_buffers: [wgpu::Buffer; 6],
    pub index_buffers_lengths: [u32; 6],
    pub vertex_gpu_data: VertexGPUData,
    pub chunk_positions_around_player: [WorldPosition; NUMBER_OF_CHUNKS_AROUND_PLAYER as usize],
    pub chunk_index_buffer: wgpu::Buffer,
    pub chunk_index_bind_group: wgpu::BindGroup,
    pub chunk_pos_to_index: std::collections::HashMap<WorldPosition, u32>
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

        let diffuse_bytes = include_bytes!("atlas.png");
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

        let screen_color = wgpu::Color {
            r: 0.0,
            g: 0.5,
            b: 0.5,
            a: 0.0,
        };

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

        println!("Player position is {}", WorldPosition::from(camera.position));

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
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
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

        let world = World::new();

        let mut task_queue = VecDeque::new();
        for pos in chunk_positions_around_player {
            task_queue.push_back(Task::GenerateChunk { chunk_position: pos });
        }

        let mesh = Mesh::new();

        let vertex_gpu_data = mesh.get_gpu_data();

        let vertex_buffers = vertex_gpu_data.generate_vertex_buffers(&device);

        let index_buffers = vertex_gpu_data.generate_index_buffers(&device);

        let index_buffers_lengths = vertex_gpu_data.generate_index_buffer_lengths();

        let mut chunk_pos_to_index = std::collections::HashMap::new();

        for i in 0..chunk_positions_around_player.len() {
            chunk_pos_to_index.insert(chunk_positions_around_player[i], i as u32);
        }

        Self {
            surface,
            device,
            queue,
            config,
            size,
            screen_color,
            render_pipeline_regular,
            render_pipeline_wireframe,
            mesh,
            depth_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            projection,
            camera_controller,
            mouse_pressed: false,
            render_wireframe: false,
            world,
            diffuse_bind_group,
            diffuse_texture,
            task_queue,
            vertex_buffers,
            index_buffers,
            index_buffers_lengths,
            vertex_gpu_data,
            chunk_positions_around_player,
            chunk_index_buffer,
            chunk_index_bind_group,
            chunk_pos_to_index,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.projection.resize(new_size.width, new_size.height);
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
                self.camera_controller.process_keyboard(*key, *state)
            }
                ,
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
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
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn process_tasks(&mut self) {
        let mut task_schedules = self.schedule_tasks();

        let wg = WaitGroup::new();

        let _ = crossbeam::scope(|scope| {
            for (ref mut tasks, ref mut task_results) in task_schedules.iter_mut() {
                let wg = wg.clone();
                scope.spawn(|_| {
                    for task in tasks {
                        match task {
                            Task::GenerateChunk { chunk_position } => { 
                                task_results.push(GenerateChunkProcessor::process_task(&chunk_position))
                            },
                            Task::GenerateChunkMesh { chunk_position } => {
                                task_results.push(GenerateChunkMeshProcessor::process_task(&chunk_position, &self.world, &self.chunk_pos_to_index))
                            }
                        }
                    }

                    drop(wg);
                });
            }
        });

        let mut chunks_generated = 0;
        let mut meshes_generated = 0;

        for (_ ,task_results) in task_schedules.drain(0..) {
            for task_result in task_results {
                match task_result {
                    TaskResult::Requeue { task } => self.task_queue.push_front(task),
                    TaskResult::GenerateChunk { chunk } => {
                        println!("Generated chunk {}!", chunks_generated);
                        chunks_generated += 1;
                        self.task_queue.push_front(Task::GenerateChunkMesh { chunk_position: chunk.position });
                        self.world.add_chunk(chunk);
                    },
                    TaskResult::GenerateChunkMesh { mesh } => {
                        println!("Generated mesh {}!", meshes_generated);
                        meshes_generated += 1;
                        self.vertex_gpu_data.add_gpu_data_drain(&mut mesh.get_gpu_data());
                    }
                }
            }
        }

        if meshes_generated > 0 {
            self.vertex_buffers = self.vertex_gpu_data.generate_vertex_buffers(&self.device);
            self.index_buffers = self.vertex_gpu_data.generate_index_buffers(&self.device);
            self.index_buffers_lengths = self.vertex_gpu_data.generate_index_buffer_lengths();
        }
    }

    fn schedule_tasks(&mut self) -> Vec<(Vec<Task>, Vec<TaskResult>)> {
        let mut tasks_scheduled = 0;
        let mut empty_task_list = false;
        let mut task_schedules = Vec::new();
        let number_tasks_per_thread = 15;
        for _ in 0..std::cmp::max(1, consts::NUM_THREADS-1) {
            task_schedules.push((Vec::new(), Vec::new()));
        }
        while tasks_scheduled < std::cmp::max(1, consts::NUM_THREADS-1) * number_tasks_per_thread && !empty_task_list {
            match self.task_queue.pop_front() {
                Some(task) => task_schedules[tasks_scheduled % (std::cmp::max(1, consts::NUM_THREADS-1))].0.push(task),
                None => empty_task_list = true
            }
            
            tasks_scheduled = tasks_scheduled + 1;
        }

        task_schedules
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
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
                        load: wgpu::LoadOp::Clear(self.screen_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
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

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &self.chunk_index_bind_group, &[]);

            let ycos = self.camera.yaw.0.cos();
            let ysin = self.camera.yaw.0.sin();
            let psin = self.camera.pitch.0.sin();

            //front
            if ycos > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[0].slice(..));
                render_pass.set_index_buffer(self.index_buffers[0].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[0], 0, 0..1);
            }
            //back
            if ycos < SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[1].slice(..));
                render_pass.set_index_buffer(self.index_buffers[1].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[1], 0, 0..1);
            }
            //left
            if ysin > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[2].slice(..));
                render_pass.set_index_buffer(self.index_buffers[2].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[2], 0, 0..1);
            }
            //right
            if ysin < SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[3].slice(..));
                render_pass.set_index_buffer(self.index_buffers[3].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[3], 0, 0..1);
            }
            //top
            if psin < SQRT_2_DIV_2  {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[4].slice(..));
                render_pass.set_index_buffer(self.index_buffers[4].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[4], 0, 0..1);
            }
            //bottom
            if psin > NEG_SQRT_2_DIV_2 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[5].slice(..));
                render_pass.set_index_buffer(self.index_buffers[5].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.index_buffers_lengths[5], 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

