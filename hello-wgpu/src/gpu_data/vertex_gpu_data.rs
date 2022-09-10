use std::collections::HashMap;

use fundamentals::world_position::WorldPosition;
use wgpu::{Device, util::DeviceExt};

use crate::{voxels::mesh::Mesh, camera::Camera};

use super::vec_vertex_index_length_triple::VecVertexIndexLengthsTriple;

pub struct VertexGPUData {
    pub data_front: VecVertexIndexLengthsTriple,
    pub data_back: VecVertexIndexLengthsTriple,
    pub data_left: VecVertexIndexLengthsTriple,
    pub data_right: VecVertexIndexLengthsTriple,
    pub data_top: VecVertexIndexLengthsTriple,
    pub data_bottom: VecVertexIndexLengthsTriple,
    pub chunk_index_array: [WorldPosition; fundamentals::consts::NUMBER_OF_CHUNKS_AROUND_PLAYER as usize],
    pub chunk_index_buffer: wgpu::Buffer,
    pub chunk_index_bind_group_layout: wgpu::BindGroupLayout,
    pub chunk_index_bind_group: wgpu::BindGroup,
    pub pos_to_gpu_index: HashMap<WorldPosition, usize>,
    pub pos_to_loaded_index: HashMap<WorldPosition, usize>,
    pub loaded_chunks: Vec<WorldPosition>
}

impl VertexGPUData {
    pub fn new(camera: &Camera, device: &Device) -> Self {
        let chunk_index_array = fundamentals::consts::get_positions_around_player(WorldPosition::from(camera.position));
        let mut pos_to_gpu_index = HashMap::new();
        for (i, chunk_position) in chunk_index_array.iter().enumerate() {
            pos_to_gpu_index.insert(*chunk_position, i);
        }
        let chunk_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(&chunk_index_array),
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
        Self {
            data_front: VecVertexIndexLengthsTriple::new(),
            data_back: VecVertexIndexLengthsTriple::new(),
            data_left: VecVertexIndexLengthsTriple::new(),
            data_right: VecVertexIndexLengthsTriple::new(),
            data_top: VecVertexIndexLengthsTriple::new(),
            data_bottom: VecVertexIndexLengthsTriple::new(),
            pos_to_gpu_index,
            pos_to_loaded_index: HashMap::new(),
            chunk_index_array,
            chunk_index_buffer,
            chunk_index_bind_group_layout,
            chunk_index_bind_group,
            loaded_chunks: Vec::new()
        }
    }

    pub fn add_mesh_data_drain(&mut self, mesh: Mesh, device: &Device, mesh_position: &WorldPosition) {
        let mesh_argument_arr = [
            (&mut self.data_front, &mesh.front, "front Mesh"),
            (&mut self.data_back, &mesh.back, "back Mesh"),
            (&mut self.data_left, &mesh.left, "left Mesh"),
            (&mut self.data_right, &mesh.right, "right Mesh"),
            (&mut self.data_top, &mesh.top, "top Mesh"),
            (&mut self.data_bottom, &mesh.bottom, "bottom Mesh"),
            ];
        for i in 0..6 {
            mesh_argument_arr[i].0.vertex_buffers.push(Mesh::generate_vertex_buffer(&mesh_argument_arr[i].1.0, device, mesh_argument_arr[i].2));
            mesh_argument_arr[i].0.index_buffers.push(Mesh::generate_index_buffer(&mesh_argument_arr[i].1.1, device, mesh_argument_arr[i].2));
            mesh_argument_arr[i].0.index_lengths.push(mesh_argument_arr[i].1.2);
        }
        self.pos_to_loaded_index.insert(*mesh_position, self.loaded_chunks.len());
        self.loaded_chunks.push(*mesh_position);
    }

    pub fn get_buffers_at_position(&self, pos: &WorldPosition) -> Option<[(&wgpu::Buffer, &wgpu::Buffer, u32); 6]> {
        match self.pos_to_loaded_index.get(pos) {
            Some(index) => self.get_buffers_at_index_i(*index),
            None => None
        }
        
    }

    pub fn get_buffers_at_index_i(&self, i: usize) -> Option<[(&wgpu::Buffer, &wgpu::Buffer, u32); 6]> {
        if i >= self.pos_to_loaded_index.len() {
            None
        } else {
            Some([
                (&self.data_front.vertex_buffers[i],
                &self.data_front.index_buffers[i],
                self.data_front.index_lengths[i]),
                (&self.data_back.vertex_buffers[i],
                &self.data_back.index_buffers[i],
                self.data_back.index_lengths[i]),
                (&self.data_left.vertex_buffers[i],
                &self.data_left.index_buffers[i],
                self.data_left.index_lengths[i]),
                (&self.data_right.vertex_buffers[i],
                &self.data_right.index_buffers[i],
                self.data_right.index_lengths[i]),
                (&self.data_top.vertex_buffers[i],
                &self.data_top.index_buffers[i],
                self.data_top.index_lengths[i]),
                (&self.data_bottom.vertex_buffers[i],
                &self.data_bottom.index_buffers[i],
                self.data_bottom.index_lengths[i]),
            ])
        }
       
    }
}