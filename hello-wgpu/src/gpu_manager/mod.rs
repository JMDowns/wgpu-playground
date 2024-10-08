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
pub mod chunk_index_state;

use camera_state::CameraState;
use cgmath::{Deg, Point3, Rad, SquareMatrix, Vector3};
use compute_state::ComputeState;
use derivables::subvoxel_vertex::generate_cube_at_center;
use flag_state::FlagState;
use log::error;
use web_time::Instant;
use render_state::RenderState;
use surface_state::SurfaceState;
use texture_state::TextureState;
use gpu_data::vertex_gpu_data::VertexGPUData;

use fundamentals::{world_position::WorldPosition, enums::block_side::BlockSide, logi, consts::{self, NUMBER_OF_CHUNKS_AROUND_PLAYER}};
use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{camera::CameraController, tasks::Task, texture, voxels::chunk::{self, Chunk}};

use self::{chunk_index_state::ChunkIndexState, subvoxels::{grid_aligned_subvoxel_object_specification::GridAlignedSubvoxelObjectSpecification, subvoxel_object_specification::SubvoxelObjectSpecification, subvoxel_state::SubvoxelState}};

use crate::gpu_manager::subvoxels::model_manager::SubvoxelModel;
use crate::gpu_manager::subvoxels::grid_aligned_subvoxel_object::ROTATION;

pub struct GPUManager<'a> {
    pub device: Arc<RwLock<wgpu::Device>>,
    pub queue: Arc<RwLock<wgpu::Queue>>,
   // pub compute_state: ComputeState,
    pub render_state: RenderState,
    pub surface_state: SurfaceState<'a>,
    pub texture_state: TextureState,
    pub camera_state: CameraState,
    pub flag_state: FlagState,
    pub subvoxel_state: SubvoxelState,
   // pub vertex_gpu_data: Arc<RwLock<VertexGPUData>>,
    pub chunk_index_state: Arc<RwLock<ChunkIndexState>>,
    pub is_surface_configured: bool
}

impl<'a> GPUManager<'a> {
    pub fn new(surface: Surface<'a>, size: PhysicalSize<u32>, device: Device, queue: Queue, config: SurfaceConfiguration, is_surface_configured: bool) -> Self {
        let screen_color = wgpu::Color {
            r: 0.0,
            g: 0.5,
            b: 0.5,
            a: 0.0,
        };

        let camera_state = CameraState::new(&device, &config);

        let texture_state = TextureState::new(&device, &queue, &config);        

        let chunk_index_state = Arc::new(RwLock::new(ChunkIndexState::new(camera_state.camera.position, &device)));

        //let vertex_gpu_data = Arc::new(RwLock::new(VertexGPUData::new(&device, chunk_index_state.clone())));

        //let compute_state = ComputeState::new(camera_state.camera.position, &device, &camera_state.camera_buffer, &vertex_gpu_data.read().unwrap().indirect_pool_buffers, &vertex_gpu_data.read().unwrap().visibility_buffer);

        let queue_rwlock = Arc::new(RwLock::new(queue));

        let mut subvoxel_state = SubvoxelState::new(&device, queue_rwlock.clone(), chunk_index_state.clone());

        let model = SubvoxelModel { model_name: String::from("Not Grid"), subvoxel_size: Vector3 { x: 2, y: 2, z: 2 }, subvoxel_vec: vec![1, 0, 1, 1, 0, 1, 0, 0] };
        subvoxel_state.register_model(model);

        let grid_model = SubvoxelModel { model_name: String::from("Grid"), subvoxel_size: Vector3 { x: 2, y: 2, z: 2 }, subvoxel_vec: vec![1, 0, 1, 0, 0, 1, 0, 0] };
        subvoxel_state.register_model(grid_model);

        let mut empty_model = SubvoxelModel { model_name: String::from("Empty"), subvoxel_size: Vector3 { x: 10, y: 10, z: 10 }, subvoxel_vec: vec![0; 1000] };
        empty_model.subvoxel_vec[0] = 1;
        subvoxel_state.register_model(empty_model);

        subvoxel_state.add_grid_aligned_subvoxel_object(GridAlignedSubvoxelObjectSpecification { 
            size: Vector3 { x: 1, y: 1, z: 1 }, 
            maximal_chunk: WorldPosition::new(0, 0, 0), 
            maximal_block_in_chunk: WorldPosition::new(0,2,10), 
            maximal_subvoxel_in_chunk: WorldPosition::new(9,3,0), 
            rotation: ROTATION::RIGHT, 
            model_name: String::from("Grid") 
        });

        subvoxel_state.add_grid_aligned_subvoxel_object(GridAlignedSubvoxelObjectSpecification { 
            size: Vector3 { x: 1, y: 1, z: 1 }, 
            maximal_chunk: WorldPosition::new(0, 0, 0), 
            maximal_block_in_chunk: WorldPosition::new(2,4,10), 
            maximal_subvoxel_in_chunk: WorldPosition::new(11,3,0), 
            rotation: ROTATION::RIGHT, 
            model_name: String::from("Empty") 
        });

        for i in 0..10 {
            for j in 0..10 {
                subvoxel_state.add_subvoxel_object(SubvoxelObjectSpecification {
                    size: Vector3 { x: 2.0, y: 2.0, z: 2.0 },
                    center: Point3 { x: i as f32 * 4., y: 0.0, z: j as f32 * 4.},
                    initial_rotation: Vector3 {x: Deg(i as f32), y: Deg(j as f32), z: Deg(0.)},
                    model_name: String::from("Not Grid")
                });
            }
        }

        let render_state = RenderState::new(
            &device, 
            &config, 
            &camera_state.camera_bind_group_layout, 
          //  &texture_state.diffuse_bind_group_layout, 
            &chunk_index_state.read().unwrap().chunk_index_bind_group_layout,
         //   &vertex_gpu_data.read().unwrap().visibility_bind_group_layout,
            &subvoxel_state.subvoxel_bind_group_layout,
            &subvoxel_state.sv_grid_aligned_bind_group_layout
        );

        GPUManager {
            device: Arc::new(RwLock::new(device)),
            queue: queue_rwlock.clone(),
           // compute_state,
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
           // vertex_gpu_data,
            chunk_index_state,
            is_surface_configured
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_state.size = new_size;
            self.surface_state.config.width = new_size.width;
            self.surface_state.config.height = new_size.height;
            self.surface_state.surface.configure(&self.device.read().unwrap(), &self.surface_state.config);
            self.is_surface_configured = true;
            self.texture_state.depth_texture = Some(texture::Texture::create_depth_texture(&self.device.read().unwrap(), &self.surface_state.config, "depth_texture"));
            self.camera_state.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn process_input(&mut self, flags: &crate::state::flag_state::FlagState) {
        self.flag_state.should_calculate_frustum = flags.has_moved || self.flag_state.should_calculate_frustum;
        self.flag_state.render_wireframe = flags.should_render_wireframe;
    }

    pub fn update_camera_and_reset_conroller(&mut self, controller: &mut CameraController, dt: web_time::Duration) {
        self.camera_state.camera.get_controller_updates_and_reset_controller(controller, dt);
        self.camera_state.camera_uniform.update_view_proj_and_pos(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.read().unwrap().write_buffer(&self.camera_state.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let now = Instant::now();

        if (!self.is_surface_configured) {
            error!("Surface not configured");
            return Ok(());
        }

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

        // {
        //     let mut vertex_gpu_data = self.vertex_gpu_data.write().unwrap();
        //     for (mesh_position, side, side_offset, bucket_position) in vertex_gpu_data.return_frustum_bucket_data_to_update_and_empty_counts() {
        //         self.compute_state.update_frustum_bucket_data(mesh_position, side, side_offset, bucket_position, &queue);
        //     }
        //     for (mesh_position, side, side_offset) in vertex_gpu_data.return_frustum_bucket_data_to_clear_and_empty_counts() {
        //         self.compute_state.clear_frustum_bucket_data(mesh_position, side, side_offset, &queue);
        //     }
        // }

        // let vertex_gpu_data = self.vertex_gpu_data.read().unwrap();
        let chunk_index_state = self.chunk_index_state.read().unwrap();

        // if self.flag_state.should_calculate_frustum {
        //     let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        //         label: Some("Compute Pass"),
        //         timestamp_writes: None,
        //     });

        //     compute_pass.set_pipeline(&self.compute_state.compute_pipeline);
        //     compute_pass.set_bind_group(0, &self.compute_state.compute_bind_group, &[]);
        //     for i in 0..(&self.compute_state.compute_indirect_bind_groups).len() {
        //         compute_pass.set_bind_group((i as u32)+1, &self.compute_state.compute_indirect_bind_groups[i], &[]);
        //     }

        //     compute_pass.dispatch_workgroups((fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as f32 / 256.0).ceil() as u32, 1, 1);

        //     self.flag_state.should_calculate_frustum = false;
        // }

        {
            let depth_stencil_attachment = match &self.texture_state.depth_texture {
                Some(texture) => Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                None => None
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.surface_state.screen_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None
            });

            // if self.flag_state.render_wireframe {
            //     render_pass.set_pipeline(&self.render_state.render_pipeline_wireframe);
            // } else {
            //     render_pass.set_pipeline(&self.render_state.render_pipeline_regular);
            // }

            //Grid-Aligned Vertices

            // render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.texture_state.diffuse_bind_group, &[]);
            // render_pass.set_bind_group(2, &chunk_index_state.chunk_index_bind_group, &[]);
            // render_pass.set_bind_group(3, &vertex_gpu_data.visibility_bind_group, &[]);

            // for i in 0..vertex_gpu_data.vertex_pool_buffers.len() {
            //     render_pass.set_vertex_buffer(0, vertex_gpu_data.vertex_pool_buffers[i].slice(..));
            //     render_pass.set_index_buffer(vertex_gpu_data.index_pool_buffers[i].slice(..), wgpu::IndexFormat::Uint32);
            //     render_pass.multi_draw_indexed_indirect(&vertex_gpu_data.indirect_pool_buffers[i], 0, vertex_gpu_data.number_of_buckets_per_buffer as u32);
            // }

            // //Occlusion

            // render_pass.set_pipeline(&self.render_state.occlusion_cube_render_pipeline);

            // render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            // render_pass.set_bind_group(1, &chunk_index_state.chunk_index_bind_group, &[]);
            // render_pass.set_bind_group(2, &vertex_gpu_data.visibility_bind_group, &[]);

            // render_pass.set_vertex_buffer(0, vertex_gpu_data.occlusion_cube_vertex_buffer.slice(..));
            // render_pass.set_index_buffer(vertex_gpu_data.occlusion_cube_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // render_pass.draw_indexed(0..36*consts::NUMBER_OF_CHUNKS_AROUND_PLAYER, 0, 0..1);

            //Subvoxel

            render_pass.set_pipeline(&self.render_state.subvoxel_render_pipeline);

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.subvoxel_state.subvoxel_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.subvoxel_state.sv_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.subvoxel_state.sv_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..derivables::subvoxel_vertex::INDICES_CUBE_LEN*self.subvoxel_state.subvoxel_objects.len() as u32, 0, 0..1);

            //Grid-Aligned Subvoxel
            render_pass.set_pipeline(&self.render_state.grid_aligned_subvoxel_render_pipeline);

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.subvoxel_state.sv_grid_aligned_bind_group, &[]);
            render_pass.set_bind_group(2, &chunk_index_state.chunk_index_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.subvoxel_state.sv_grid_aligned_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.subvoxel_state.sv_grid_aligned_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..derivables::grid_aligned_subvoxel_vertex::INDICES_CUBE_LEN * self.subvoxel_state.grid_aligned_subvoxel_objects.len() as u32, 0, 0..1);

        }

        // let current_chunk = self.camera_state.camera.get_chunk_coordinates();

        // match chunk_index_state.pos_to_gpu_index.get(&current_chunk) {
        //     Some(index) => {
        //         queue.write_buffer(&vertex_gpu_data.visibility_buffer, ((*index) as u64) * std::mem::size_of::<i32>() as u64, bytemuck::cast_slice(&[1]));
        //     }
        //     None => {}
        // }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        let after = Instant::now();
        let time = (after-now).as_millis();
        logi!("Time to render was {}", time);
        Ok(())
    }


    
    // pub fn create_generate_chunk_mesh_task(&self, chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>) -> Task {
    //     Task::GenerateChunkMesh { 
    //         chunk_position, 
    //         chunk, 
    //         vertex_gpu_data: self.vertex_gpu_data.clone(),
    //         queue: self.queue.clone(),
    //         chunk_index_state: self.chunk_index_state.clone()
    //     }
    // }

    // pub fn create_generate_chunk_side_mesh_task(&self, chunk_position: WorldPosition, chunk: Arc<RwLock<Chunk>>, side: BlockSide) -> Task {
    //     Task::GenerateChunkSideMeshes { 
    //         chunk_position, 
    //         chunk, 
    //         vertex_gpu_data: self.vertex_gpu_data.clone(),
    //         queue: self.queue.clone(),
    //         sides: vec![side],
    //         chunk_index_state: self.chunk_index_state.clone()
    //     }
    // }

    // pub fn process_generate_chunk_mesh_task_result(&mut self) {
    //     self.flag_state.should_calculate_frustum = true;
    //     while self.vertex_gpu_data.read().unwrap().should_allocate_new_buffer() {
    //         self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
    //     }
    // }

    // pub fn process_update_chunk_side_mesh_result(&mut self) {
    //     while self.vertex_gpu_data.read().unwrap().should_allocate_new_buffer() {
    //         self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
    //     }
    // }

    // pub fn allocate_new_buffer(&mut self) { 
    //     self.vertex_gpu_data.write().unwrap().allocate_new_buffer(self.device.clone());
    // }

    pub fn rotate_subvoxel_object(&mut self, id: usize) {
        self.subvoxel_state.rotate(id, Vector3{ x: Deg(1.0), y: Deg(0.0), z: Deg(0.0) });
        self.subvoxel_state.rotate(id+1, Vector3{ x: Deg(-1.0), y: Deg(0.0), z: Deg(0.0) });
    }
}

