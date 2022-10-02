use std::sync::{Arc, RwLock};

mod camera_state;
mod compute_state;
mod compute_state_generated_helper;
mod flag_state;
mod render_state;
mod surface_state;
mod texture_state;
pub mod gpu_data;

use camera_state::CameraState;
use compute_state::ComputeState;
use flag_state::FlagState;
use render_state::RenderState;
use surface_state::SurfaceState;
use texture_state::TextureState;
use gpu_data::vertex_gpu_data::VertexGPUData;

use fundamentals::world_position::WorldPosition;
use winit::window::Window;

use crate::{texture, camera::CameraController, tasks::Task, voxels::world::World};

const SQRT_2_DIV_2: f32 = 0.7071;
const NEG_SQRT_2_DIV_2: f32 = -0.7071;

pub struct GPUManager {
    pub device: Arc<RwLock<wgpu::Device>>,
    pub queue: Arc<RwLock<wgpu::Queue>>,
    pub compute_state: ComputeState,
    pub render_state: RenderState,
    pub surface_state: SurfaceState,
    pub texture_state: TextureState,
    pub camera_state: CameraState,
    pub flag_state: FlagState,
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
                    features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::MULTI_DRAW_INDIRECT,
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

        let camera_state = CameraState::new(&device, &config);

        let texture_state = TextureState::new(&device, &queue, &config);        

        let vertex_gpu_data = Arc::new(RwLock::new(VertexGPUData::new(camera_state.camera.position, &device)));

        let compute_state = ComputeState::new(camera_state.camera.position, &device, &camera_state.camera_buffer, &vertex_gpu_data.read().unwrap().indirect_pool_buffers);

        let render_state = RenderState::new(
            &device, 
            &config, 
            &camera_state.camera_bind_group_layout, 
            &texture_state.diffuse_bind_group_layout, 
            &vertex_gpu_data.read().unwrap().chunk_index_bind_group_layout
        );

        GPUManager {
            device: Arc::new(RwLock::new(device)),
            queue: Arc::new(RwLock::new(queue)),
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

    pub fn process_input(&mut self, has_moved: bool) {
        self.flag_state.should_calculate_frustum = has_moved || self.flag_state.should_calculate_frustum;
    }

    pub fn update_camera_and_reset_conroller(&mut self, controller: &mut CameraController, dt: instant::Duration) {
        self.camera_state.camera.get_controller_updates_and_reset_controller(controller, dt);
        self.camera_state.camera_uniform.update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.read().unwrap().write_buffer(&self.camera_state.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_state.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            for (mesh_position, side, side_offset, bucket_position, index_count) in vertex_gpu_data.return_frustum_bucket_data_to_update_and_empty_counts() {
                self.compute_state.update_frustum_bucket_data(mesh_position, side, side_offset, bucket_position, index_count, &queue);
            }
        }

        let vertex_gpu_data = self.vertex_gpu_data.read().unwrap();

        if self.flag_state.should_calculate_frustum {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&self.compute_state.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_state.compute_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.compute_state.compute_indirect_bind_group, &[]);

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

            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_state.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &vertex_gpu_data.chunk_index_bind_group, &[]);

            for i in 0..vertex_gpu_data.vertex_pool_buffers.len() - 1 {
                render_pass.set_vertex_buffer(0, vertex_gpu_data.vertex_pool_buffers[i].slice(..));
                render_pass.set_index_buffer(vertex_gpu_data.index_pool_buffers[i].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.multi_draw_indexed_indirect(&vertex_gpu_data.indirect_pool_buffers[i], 0, vertex_gpu_data.number_of_buckets_per_buffer as u32);
            }

            render_pass.set_vertex_buffer(0, vertex_gpu_data.vertex_pool_buffers[vertex_gpu_data.vertex_pool_buffers.len() - 1].slice(..));
            render_pass.set_index_buffer(vertex_gpu_data.index_pool_buffers[vertex_gpu_data.vertex_pool_buffers.len() - 1].slice(..), wgpu::IndexFormat::Uint32);
            render_pass.multi_draw_indexed_indirect(&vertex_gpu_data.indirect_pool_buffers[vertex_gpu_data.vertex_pool_buffers.len() - 1], 0, vertex_gpu_data.number_of_buckets_in_last_buffer as u32);

            // let ycos = self.camera_state.camera.yaw.0.cos();indirect_buffer
            // let ysin = self.camera_state.camera.yaw.0.sin();
            // let psin = self.camera_state.camera.pitch.0.sin();

            // //Front, Back, Left, Right, Top, Bottom
            // let lhs_comp_arr = [ycos, ycos, ysin, ysin, psin, psin];
            // let is_comp_lt = [false, true, false, true, true, false];
            // let angles = [NEG_SQRT_2_DIV_2, SQRT_2_DIV_2, NEG_SQRT_2_DIV_2, SQRT_2_DIV_2, SQRT_2_DIV_2, NEG_SQRT_2_DIV_2];

            // for chunk_pos_index in 0..fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize {
            //     if self.compute_state.compute_staging_vec[chunk_pos_index] == 0 {
            //         continue;
            //     }
            //     let gpu_data_result = vertex_gpu_data.get_buffers_at_position(&vertex_gpu_data.chunk_index_array[chunk_pos_index]);
            //     match gpu_data_result {
            //         Some(gpu_data) => {
            //             for i in 0..6 {
            //                 if is_comp_lt[i] {
            //                     if lhs_comp_arr[i] < angles[i] {
            //                         render_pass.set_vertex_buffer(0, gpu_data[i].0.slice(..));
            //                         render_pass.set_index_buffer(gpu_data[i].1.slice(..), wgpu::IndexFormat::Uint32);
            //                         render_pass.draw_indexed(0..gpu_data[i].2, 0, 0..1);
            //                     }
            //                 } else {
            //                     if lhs_comp_arr[i] > angles[i] {
            //                         render_pass.set_vertex_buffer(0, gpu_data[i].0.slice(..));
            //                         render_pass.set_index_buffer(gpu_data[i].1.slice(..), wgpu::IndexFormat::Uint32);
            //                         render_pass.draw_indexed(0..gpu_data[i].2, 0, 0..1);
            //                     }
            //                 }
            //             }
            //         },
            //         None => {}
            //     }
            // }
        }

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }

    pub fn create_generate_chunk_mesh_task(&self, chunk_position: WorldPosition, world: Arc<RwLock<World>>) -> Task {
        Task::GenerateChunkMesh { 
            chunk_position, 
            world, 
            vertex_gpu_data: self.vertex_gpu_data.clone(),
            queue: self.queue.clone()
        }
    }

    pub fn process_generate_chunk_mesh_task_result(&mut self) {
        self.flag_state.should_calculate_frustum = true;
    }
}

