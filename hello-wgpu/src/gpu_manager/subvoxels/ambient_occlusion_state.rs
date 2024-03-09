use std::{collections::HashMap, hash::Hash, sync::{Arc, RwLock}};

use wgpu::{BufferUsages, Queue};

use super::subvoxel_object::SubvoxelObject;

pub struct AmbientOcclusionState {
    pub ao_buffer: wgpu::Buffer,
    pub subvoxel_id_to_ao_offset: HashMap<u32, u32>
}

//Each Bucket is 20 bytes long
pub struct AmbientOcclusionBucket {
    pub start: usize
}

impl AmbientOcclusionState {
    pub fn new(device: &wgpu::Device, queue: Arc<RwLock<Queue>>) -> Self {
        let ao_size = 20;
        let ao_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Ambient Occlusion Buffer"),
                size: ao_size,
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
        }
    }

    pub fn set_ambient_occlusion(&mut self, voxel_object: &SubvoxelObject, queue: Arc<RwLock<Queue>>) {
        let ao_calculator = AmbientOcclusionCalculator {
            voxel_object
        };

        let vec = ao_calculator.generate_bits();

        let mut ao_offset: u32 = 0;
        if (self.subvoxel_id_to_ao_offset.contains_key(&voxel_object.id)) {
            ao_offset = *self.subvoxel_id_to_ao_offset.get(&voxel_object.id).unwrap();
        }
        else {
            ao_offset = 0;
            self.subvoxel_id_to_ao_offset.insert(voxel_object.id, ao_offset);
        }

        queue.read().unwrap().write_buffer(&self.ao_buffer, ao_offset as u64 * 20, bytemuck::cast_slice(&vec));
    }
}

struct AmbientOcclusionCalculator<'a> {
    pub voxel_object: &'a SubvoxelObject
}

impl AmbientOcclusionCalculator<'_> {
    fn solid_at_bit(&self, x: i32, y: i32, z: i32) -> &str {
        if (x < 0 || y < 0 || z < 0 || x as u32 >= self.voxel_object.subvoxel_size.x || y as u32 >= self.voxel_object.subvoxel_size.y || z as u32 >= self.voxel_object.subvoxel_size.z) {
            return "0";
        }

        match self.voxel_object.subvoxel_vec[x as usize + y as usize * self.voxel_object.subvoxel_size.x as usize + z as usize * self.voxel_object.subvoxel_size.x as usize * self.voxel_object.subvoxel_size.y as usize] != 0 {
            true => "1",
            false => "0"
        }
    }

    fn generate_bits(&self) -> Vec<u32> {
        let mut uint_vec = Vec::new();
        let mut bitstring_total = String::new();
        let empty_string = &String::from("0").repeat(20);
        for k in 0..self.voxel_object.subvoxel_size.x as i32 {
            for j in 0..self.voxel_object.subvoxel_size.y as i32 {
                for i in 0..self.voxel_object.subvoxel_size.z as i32 {
                    if (self.solid_at_bit(i, j, k) == "0") {
                        bitstring_total.push_str(empty_string);
                        continue
                    }
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j+1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j-1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i-1, j, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i, j+1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i, j-1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i, j-1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i, j+1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j+1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k+1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j-1, k-1));
                    bitstring_total.push_str(self.solid_at_bit(i+1, j, k-1));
                }
            }
        }
        let num_uints = ((self.voxel_object.subvoxel_vec.len() * 20) as f32 / 32.0).ceil() as usize;
        for i in 0..num_uints {
            let bitstr = &bitstring_total[(i * 32)..((i+1)*32)];
            let uint = u32::from_str_radix(bitstr, 2).unwrap();
            uint_vec.push(uint);
        }
        return uint_vec;
    }
}
