use std::{collections::HashMap, hash::Hash, num, sync::{Arc, RwLock}};

use wgpu::{BufferUsages, Queue};

use super::{model_manager::{SubvoxelModel, SUBVOXEL_PALETTE}, subvoxel_object::SubvoxelObject, subvoxel_state::MAX_SUBVOXELS};

use cgmath::Vector3;

pub struct AmbientOcclusionState {
    pub ao_buffer: wgpu::Buffer,
    pub subvoxel_id_to_ao_offset: HashMap<u32, u32>,
    pub model_name_to_ao_offset: HashMap<String, u32>,
    pub available_ao_space: Vec<AOSpace>,
    pub queue: Arc<RwLock<wgpu::Queue>>
}

pub struct AOSpace {
    pub offset_in_u32s: u32,
    pub length_in_u32s: u32
}

impl AmbientOcclusionState {
    pub fn new(device: &wgpu::Device, queue: Arc<RwLock<Queue>>) -> Self {

        let num_u32s_in_ao_buffer = (MAX_SUBVOXELS * 20).div_ceil(32);
        let ao_buffer_length = std::mem::size_of::<u32>() as u64 * num_u32s_in_ao_buffer;
        let available_ao_space = vec![
            AOSpace {
                length_in_u32s: num_u32s_in_ao_buffer as u32,
                offset_in_u32s: 0
            }
        ];

        let ao_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Ambient Occlusion Buffer"),
                size: ao_buffer_length,
                usage: BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );

        let ao_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                }
            ],
            label: Some("ambient_occlusion_bind_group_layout"),
        });

        let ao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ao_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ao_buffer.as_entire_binding(),
                }
            ],
            label: Some("ambient_occlusion_bind_group")
        });

        AmbientOcclusionState {
            ao_buffer,
            subvoxel_id_to_ao_offset: HashMap::new(),
            model_name_to_ao_offset: HashMap::new(),
            available_ao_space,
            queue: queue.clone()
        }
    }

    pub fn set_model_ambient_occlusion(&mut self, voxel_model: &SubvoxelModel) {
        let vec =  self.generate_bits(&voxel_model.subvoxel_size, &voxel_model.subvoxel_vec);

        let mut ao_offset: u32 = 0;
        if (self.model_name_to_ao_offset.contains_key(&voxel_model.model_name)) {
            ao_offset = *self.model_name_to_ao_offset.get(&voxel_model.model_name).unwrap();
        }
        else {
            ao_offset = self.fill_available_space_and_get_offset(voxel_model.subvoxel_vec.len() as u32);
            self.model_name_to_ao_offset.insert(voxel_model.model_name.clone(), ao_offset);
        }

        self.queue.read().unwrap().write_buffer(&self.ao_buffer, ao_offset as u64 * std::mem::size_of::<u32>() as u64, bytemuck::cast_slice(&vec));
    }

    fn fill_available_space_and_get_offset(&mut self, sv_vec_length: u32) -> u32 {
        for i in 0..self.available_ao_space.len() {
            let space = self.available_ao_space.get(i).unwrap();
            let space_length = space.length_in_u32s;
            let space_offset = space.offset_in_u32s;
            let total_length_in_u32s = (sv_vec_length * 20).div_ceil(32);
            if space_length >= total_length_in_u32s {
                self.available_ao_space.remove(i);
                if space_length != total_length_in_u32s {
                    self.available_ao_space.push( AOSpace {
                        length_in_u32s: space_length - total_length_in_u32s,
                        offset_in_u32s: space_offset + total_length_in_u32s
                    });
                }
                return space_offset;
            }
        }

        panic!("Unable to find ao space, figure something out");
    }

    fn solid_at_bit(&self, x: i32, y: i32, z: i32, subvoxel_size: &Vector3<u32>, subvoxel_vec: &Vec<SUBVOXEL_PALETTE>) -> &str {
        if (x < 0 || y < 0 || z < 0 || x as u32 >= subvoxel_size.x || y as u32 >= subvoxel_size.y || z as u32 >= subvoxel_size.z) {
            return "0";
        }

        match subvoxel_vec[x as usize + y as usize * subvoxel_size.x as usize + z as usize * subvoxel_size.x as usize * subvoxel_size.y as usize] != 0 {
            true => "1",
            false => "0"
        }
    }

    fn generate_bits(&self, subvoxel_size: &Vector3<u32>, subvoxel_vec: &Vec<SUBVOXEL_PALETTE>) -> Vec<u32> {
        let mut uint_vec = Vec::new();
        let mut bitstring_total = String::new();
        let empty_string = &String::from("0").repeat(20);
        for k in 0..subvoxel_size.x as i32 {
            for j in 0..subvoxel_size.y as i32 {
                for i in 0..subvoxel_size.z as i32 {
                    if (self.solid_at_bit(i, j, k, subvoxel_size, subvoxel_vec) == "0") {
                        bitstring_total.push_str(empty_string);
                        continue
                    }
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i, j+1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i, j-1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i, j-1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i, j+1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k+1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k-1, subvoxel_size, subvoxel_vec));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j, k-1, subvoxel_size, subvoxel_vec));
                }
            }
        }
        let num_uints = ((subvoxel_vec.len() * 20) as f32 / 32.0).ceil() as usize;
        for i in 0..num_uints {
            let bitstr = &bitstring_total[(i * 32)..((i+1)*32)];
            let uint = u32::from_str_radix(bitstr, 2).unwrap();
            uint_vec.push(uint);
        }
        return uint_vec;
    }
}