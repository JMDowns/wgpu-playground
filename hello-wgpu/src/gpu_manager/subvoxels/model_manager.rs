use std::{collections::HashMap, sync::{Arc, RwLock}};
use cgmath::{Vector3, Vector4};
use fundamentals::consts::SUBVOXEL_PALETTE;

use super::ambient_occlusion_state::AmbientOcclusionState;

pub struct SubvoxelModelManager {
    pub queue: Arc<RwLock<wgpu::Queue>>,
    pub sv_model_name_to_buffer_offset_in_u32s: HashMap<String, u32>,
    pub sv_model_buffer: wgpu::Buffer,
    pub subvoxel_models: HashMap<String, SubvoxelModel>,
    pub available_model_space: Vec<ModelBufferSpace>,
    pub ao_state: AmbientOcclusionState
}

pub struct ModelBufferSpace {
    length_in_u32s: u32,
    offset_in_u32s: u32,
}

pub struct SubvoxelModel {
    pub model_name: String,
    pub subvoxel_size: Vector3<u32>,
    pub subvoxel_vec: Vec<SUBVOXEL_PALETTE>,
}

impl SubvoxelModelManager {
    pub fn new(device: &wgpu::Device, queue: Arc<RwLock<wgpu::Queue>>) -> Self {
        let sv_model_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Model Buffer"),
                size: std::mem::size_of::<u32>() as u64 * fundamentals::consts::NUM_SUBVOXEL_U32s,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let ao_state = AmbientOcclusionState::new(device, queue.clone());

        SubvoxelModelManager {
            queue,
            sv_model_name_to_buffer_offset_in_u32s: HashMap::new(),
            sv_model_buffer,
            subvoxel_models: HashMap::new(),
            available_model_space: vec![ModelBufferSpace { length_in_u32s: fundamentals::consts::NUM_SUBVOXEL_U32s as u32, offset_in_u32s: 0}],
            ao_state
        }
    }

    pub fn add_model(&mut self, model: SubvoxelModel) {
        let model_data = bytemuck::cast_slice(&model.subvoxel_vec);
        let model_subvoxel_size_array: [u32; 3] = model.subvoxel_size.into();
        let offset_in_u32s = self.fill_available_space_and_get_offset(model.subvoxel_vec.len() as u32);

        self.ao_state.set_model_ambient_occlusion(&model);
        let ao_offset = self.ao_state.model_name_to_ao_offset.get(&model.model_name).unwrap();

        self.queue.read().unwrap().write_buffer(&self.sv_model_buffer, offset_in_u32s as u64 * std::mem::size_of::<u32>() as u64, bytemuck::cast_slice(&model_subvoxel_size_array));
        self.queue.read().unwrap().write_buffer(&self.sv_model_buffer, offset_in_u32s as u64 * std::mem::size_of::<u32>() as u64  + std::mem::size_of::<Vector3<u32>>() as u64, bytemuck::cast_slice(&[*ao_offset]));
        self.queue.read().unwrap().write_buffer(&self.sv_model_buffer, offset_in_u32s as u64 * std::mem::size_of::<u32>() as u64 + std::mem::size_of::<Vector3<u32>>() as u64 + std::mem::size_of::<u32>() as u64, model_data);

        self.sv_model_name_to_buffer_offset_in_u32s.insert(model.model_name.clone(), offset_in_u32s);
        self.subvoxel_models.insert(model.model_name.clone(), model);
    }

    pub fn get_model(&self, model_name: &str) -> &SubvoxelModel {
        self.subvoxel_models.get(model_name).unwrap()
    }

    pub fn get_model_offset(&self, model_name: &str) -> u32 {
        *self.sv_model_name_to_buffer_offset_in_u32s.get(model_name).unwrap()
    }

    fn fill_available_space_and_get_offset(&mut self, sv_vec_length: u32) -> u32 {
        for i in 0..self.available_model_space.len() {
            let space = self.available_model_space.get(i).unwrap();
            let space_length = space.length_in_u32s;
            let space_offset = space.offset_in_u32s;
            let total_length = (std::mem::size_of::<SUBVOXEL_PALETTE>() as u32 * 8 * sv_vec_length + std::mem::size_of::<Vector4<u32>>() as u32 * 8).div_ceil(32);
            if space_length >= total_length {
                self.available_model_space.remove(i);
                if space_length != total_length {
                    self.available_model_space.push( ModelBufferSpace {
                        length_in_u32s: space_length - total_length,
                        offset_in_u32s: space_offset + total_length
                    });
                }
                return space_offset;
            }
        }

        panic!("Unable to find voxel model space, figure something out");
    }
}