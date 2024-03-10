use std::sync::{Arc, RwLock};

mod camera_state;
mod compute_state;
mod compute_state_generated_helper;
mod flag_state;
mod render_state;
mod surface_state;
mod texture_state;
pub mod gpu_data;
mod subvoxels;

use camera_state::CameraState;
use cgmath::{Deg, Point3, Rad, SquareMatrix, Vector3};
use compute_state::ComputeState;
use derivables::subvoxel_vertex::generate_cube_at_center;
use flag_state::FlagState;
use instant::Instant;
use render_state::RenderState;
use surface_state::SurfaceState;
use texture_state::TextureState;
use gpu_data::vertex_gpu_data::VertexGPUData;

use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, logi, consts::{self, NUMBER_OF_CHUNKS_AROUND_PLAYER}};
use winit::window::Window;

use crate::{camera::CameraController, tasks::Task, texture, voxels::chunk::Chunk};

use self::subvoxels::{subvoxel_state::SubvoxelState, subvoxel_object_specification::SubvoxelObjectSpecification};

pub struct GPUManager {
    pub device: Arc<RwLock<wgpu::Device>>,
    pub queue: Arc<RwLock<wgpu::Queue>>,
    pub compute_state: ComputeState,
    pub render_state: RenderState,
    pub surface_state: SurfaceState,
    pub texture_state: TextureState,
    pub camera_state: CameraState,
    pub flag_state: FlagState,
    pub subvoxel_state: SubvoxelState,
    pub vertex_gpu_data: Arc<RwLock<VertexGPUData>>,
}

impl GPUManager {
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
                    features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::VERTEX_WRITABLE_STORAGE,
                    limits: wgpu::Limits::default(),
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

        let camera_state = CameraState::new(&device, &config);

        let texture_state = TextureState::new(&device, &queue, &config);        

        let vertex_gpu_data = Arc::new(RwLock::new(VertexGPUData::new(camera_state.camera.position, &device)));

        let compute_state = ComputeState::new(camera_state.camera.position, &device, &camera_state.camera_buffer, &vertex_gpu_data.read().unwrap().indirect_pool_buffers, &vertex_gpu_data.read().unwrap().visibility_buffer);

        let queue_rwlock = Arc::new(RwLock::new(queue));

        let mut subvoxel_state = SubvoxelState::new(&device, queue_rwlock.clone());
        subvoxel_state.add_subvoxel_object(SubvoxelObjectSpecification {
            size: Vector3 { x: 2.0, y: 2.0, z: 2.0 }, 
            subvoxel_size: Vector3 { x: 2, y: 2, z: 2 }, 
            center: Point3 { x: 0.0, y: 0.0, z: 0.0},
            initial_rotation: Vector3 {x: Deg(0.), y: Deg(0.), z: Deg(0.)},
            subvoxel_vec: vec![0, 2, 1, 0, 1, 0, 0, 1]
        });

        subvoxel_state.add_subvoxel_object(SubvoxelObjectSpecification {
            size: Vector3 { x: 2.0, y: 2.0, z: 2.0 }, 
            subvoxel_size: Vector3 { x: 2, y: 2, z: 2 }, 
            center: Point3 { x: 2.5, y: 2.5, z: 2.5},
            initial_rotation: Vector3 {x: Deg(0.), y: Deg(0.), z: Deg(0.)},
            subvoxel_vec: vec![3, 0, 0, 1, 0, 1, 1, 0]
        });

        subvoxel_state.add_subvoxel_object(SubvoxelObjectSpecification {
            size: Vector3 { x: 2.0, y: 2.0, z: 2.0 }, 
            subvoxel_size: Vector3 { x: 2, y: 2, z: 2 }, 
            center: Point3 { x: 5.0, y: 5.0, z: 5.0},
            initial_rotation: Vector3 {x: Deg(0.), y: Deg(0.), z: Deg(0.)},
            subvoxel_vec: vec![1, 0, 0, 1, 0, 1, 1, 0]
        });

        let render_state = RenderState::new(
            &device, 
            &config, 
            &camera_state.camera_bind_group_layout, 
            &texture_state.diffuse_bind_group_layout, 
            &vertex_gpu_data.read().unwrap().chunk_index_bind_group_layout,
            &vertex_gpu_data.read().unwrap().visibility_bind_group_layout,
            &subvoxel_state.subvoxel_bind_group_layout
        );

        GPUManager {
            device: Arc::new(RwLock::new(device)),
            queue: queue_rwlock.clone(),
            compute_state,
            texture_state,
            camera_state,
            render_state,
            surface_state: SurfaceState {
                surface,
                config,
                size,
                screen_color,
            },
            flag_state: FlagState {
                should_calculate_frustum: false,
                render_wireframe: false,
            },
            subvoxel_state,
            vertex_gpu_data,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_state.size = new_size;
            self.surface_state.config.width = new_size.width;
            self.surface_state.config.height = new_size.height;
            self.surface_state.surface.configure(&self.device.read().unwrap(), &self.surface_state.config);
            self.texture_state.depth_texture = texture::Texture::create_depth_texture(&self.device.read().unwrap(), &self.surface_state.config, "depth_texture");
            self.camera_state.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn process_input(&mut self, flags: &crate::state::flag_state::FlagState) {
        self.flag_state.should_calculate_frustum = flags.has_moved || self.flag_state.should_calculate_frustum;
        self.flag_state.render_wireframe = flags.should_render_wireframe;
    }

    pub fn update_camera_and_reset_conroller(&mut self, controller: &mut CameraController, dt: instant::Duration) {
        self.camera_state.camera.get_controller_updates_and_reset_controller(controller, dt);
        self.camera_state.camera_uniform.update_view_proj_and_pos(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.read().unwrap().write_buffer(&self.camera_state.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let now = Instant::now();
        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device.read().unwrap()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let queue = self.queue.read().unwrap();

        {
            let mut vertex_gpu_data = self.vertex_gpu_data.write().unwrap();
            for (mesh_position, side, side_offset, bucket_position) in vertex_gpu_data.return_frustum_bucket_data_to_update_and_empty_counts() {
                self.compute_state.update_frustum_bucket_data(mesh_position, side, side_offset, bucket_position, &queue);
            }
            for (mesh_position, side, side_offset) in vertex_gpu_data.return_frustum_bucket_data_to_clear_and_empty_counts() {
                self.compute_state.clear_frustum_bucket_data(mesh_position, side, side_offset, &queue);
            }
        }

        let vertex_gpu_data = self.vertex_gpu_data.read().unwrap();

        if self.flag_state.should_calculate_frustum {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&self.compute_state.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_state.compute_bind_group, &[]);
            for i in 0..(&self.compute_state.compute_indirect_bind_groups).len() {
                compute_pass.set_bind_group((i as u32)+1, &self.compute_state.compute_indirect_bind_groups[i], &[]);
            }

            compute_pass.dispatch_workgroups((fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as f32 / 256.0).ceil() as u32, 1, 1);

            self.flag_state.should_calculate_frustum = false;
        }

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

            if self.flag_state.render_wireframe {
                render_pass.set_pipeline(&self.render_state.render_pipeline_wireframe);
            } else {
                render_pass.set_pipeline(&self.render_state.render_pipeline_regular);
            }

            //Grid-Aligned Vertices

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_state.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &vertex_gpu_data.chunk_index_bind_group, &[]);
            render_pass.set_bind_group(3, &vertex_gpu_data.visibility_bind_group, &[]);

            for i in 0..vertex_gpu_data.vertex_pool_buffers.len() {
                render_pass.set_vertex_buffer(0, vertex_gpu_data.vertex_pool_buffers[i].slice(..));
                render_pass.set_index_buffer(vertex_gpu_data.index_pool_buffers[i].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.multi_draw_indexed_indirect(&vertex_gpu_data.indirect_pool_buffers[i], 0, vertex_gpu_data.number_of_buckets_per_buffer as u32);
            }

            //Occlusion

            render_pass.set_pipeline(&self.render_state.occlusion_cube_render_pipeline);

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &vertex_gpu_data.chunk_index_bind_group, &[]);
            render_pass.set_bind_group(2, &vertex_gpu_data.visibility_bind_group, &[]);

            render_pass.set_vertex_buffer(0, vertex_gpu_data.occlusion_cube_vertex_buffer.slice(..));
            render_pass.set_index_buffer(vertex_gpu_data.occlusion_cube_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..36*consts::NUMBER_OF_CHUNKS_AROUND_PLAYER, 0, 0..1);

            //Subvoxel

            render_pass.set_pipeline(&self.render_state.subvoxel_render_pipeline);

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.subvoxel_state.subvoxel_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.subvoxel_state.sv_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.subvoxel_state.sv_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..derivables::subvoxel_vertex::INDICES_CUBE_LEN*self.subvoxel_state.subvoxel_objects.len() as u32, 0, 0..1);

        }

        let current_chunk = self.camera_state.camera.get_chunk_coordinates();

        match vertex_gpu_data.pos_to_gpu_index.get(&current_chunk) {
            Some(index) => {
                queue.write_buffer(&vertex_gpu_data.visibility_buffer, ((*index) as u64) * std::mem::size_of::<i32>() as u64, bytemuck::cast_slice(&[1]));
            }
            None => {}
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        let after = Instant::now();
        let time = (after-now).as_millis();
        logi!("Time to render was {}", time);
        Ok(())
    }

    pub fn create_generate_chunk_mesh_task(&self, chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>) -> Task {
        Task::GenerateChunkMesh { 
            chunk_position, 
            chunk, 
            vertex_gpu_data: self.vertex_gpu_data.clone(),
            queue: self.queue.clone()
        }
    }

    pub fn create_generate_chunk_side_mesh_task(&self, chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, side: BlockSide) -> Task {
        Task::GenerateChunkSideMeshes { 
            chunk_position, 
            chunk, 
            vertex_gpu_data: self.vertex_gpu_data.clone(),
            queue: self.queue.clone(),
            sides: vec![side]
        }
    }

    pub fn process_generate_chunk_mesh_task_result(&mut self) {
        self.flag_state.should_calculate_frustum = true;
        while self.vertex_gpu_data.read().unwrap().should_allocate_new_buffer() {
            self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
        }
    }

    pub fn process_update_chunk_side_mesh_result(&mut self) {
        while self.vertex_gpu_data.read().unwrap().should_allocate_new_buffer() {
            self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
        }
    }

    pub fn allocate_new_buffer(&mut self) { 
        self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
    }

    pub fn rotate_subvoxel_object(&mut self, id: usize) {
        self.subvoxel_state.rotate(id, Vector3{ x: Deg(1.0), y: Deg(0.0), z: Deg(0.0) });
        self.subvoxel_state.rotate(id+1, Vector3{ x: Deg(-1.0), y: Deg(0.0), z: Deg(0.0) });
    }
}

