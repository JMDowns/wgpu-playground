use std::collections::HashMap;
use num_traits::Zero;
use wgpu::{Device, util::DeviceExt, Queue, BufferUsages};
use cgmath::{EuclideanSpace, Matrix3, Point3, Rad, Vector3, Deg, Vector4};
use derivables::subvoxel_vertex::{generate_cube_at_center, generate_indices_for_index, SubvoxelVertex};
use bytemuck::{Zeroable, Pod};
use std::sync::{Arc, RwLock};

use super::ambient_occlusion_state::AmbientOcclusionState;

use super::subvoxel_object::{SubvoxelObject, SUBVOXEL_PALETTE};
use super::subvoxel_object_specification::SubvoxelObjectSpecification;
use super::subvoxel_gpu_data::SubvoxelGpuData;

pub struct SubvoxelState {
    pub ambient_occlusion_state: AmbientOcclusionState,
    pub subvoxel_objects: Vec<SubvoxelObject>,
    pub sv_data_buffer: wgpu::Buffer,
    pub sv_voxel_buffer: wgpu::Buffer,
    pub sv_palette_buffer: wgpu::Buffer,
    pub sv_vertex_buffer: wgpu::Buffer,
    pub sv_index_buffer: wgpu::Buffer,
    pub subvoxel_bind_group_layout: wgpu::BindGroupLayout,
    pub subvoxel_bind_group: wgpu::BindGroup,
    pub queue: Arc<RwLock<Queue>>,
    pub sv_id_to_vec_offset: HashMap<u32, u32>,
}

const MAX_SUBVOXEL_OBJECTS: u64 = 32;
const MAX_SUBVOXELS: u64 = 4096;
const MAX_COLORS: u64 = 32;

impl SubvoxelState {
    pub fn new(device: &Device, queue: Arc<RwLock<Queue>>) -> Self {
        let subvoxel_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None
                    }
                ],
                label: Some("subvoxel_bind_group_layout"),
            });

        let sv_data_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Data Buffer"),
                size: (std::mem::size_of::<SubvoxelGpuData>()) as u64  * MAX_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_voxel_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Voxel Buffer"),
                size: (std::mem::size_of::<SUBVOXEL_PALETTE>()) as u64  * MAX_SUBVOXELS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_palette_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Palette Buffer"),
                size: (std::mem::size_of::<f32>() * 4) as u64 * MAX_COLORS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let colors: Vec<f32> = vec![
            1.0, 1.0, 1.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
        ];

        queue.read().unwrap().write_buffer(&sv_palette_buffer, 0, bytemuck::cast_slice(&colors));

        let sv_vertex_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Cube Vertex Buffer"),
                size: (std::mem::size_of::<SubvoxelVertex>() * 24) as u64 * MAX_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_index_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Cube Index Buffer"),
                size: (std::mem::size_of::<u32>() * 36) as u64 * MAX_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::INDEX | BufferUsages::COPY_DST,
            }
        );

        let ambient_occlusion_state = AmbientOcclusionState::new(device, queue.clone());

        let subvoxel_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Subvoxel Bind Group"),
            layout: &subvoxel_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sv_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sv_voxel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sv_palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: ambient_occlusion_state.ao_buffer.as_entire_binding()
                }
            ]
        });

        Self {
            ambient_occlusion_state,
            subvoxel_objects: Vec::new(),
            sv_id_to_vec_offset: HashMap::new(),
            sv_data_buffer,
            sv_voxel_buffer,
            sv_palette_buffer,
            sv_vertex_buffer,
            sv_index_buffer,
            subvoxel_bind_group_layout,
            subvoxel_bind_group,
            queue
        }
    }

    pub fn add_subvoxel_object(&mut self, spec: SubvoxelObjectSpecification) -> usize {
        let id = self.subvoxel_objects.len();
        let object = SubvoxelObject::new(id as u32, spec);
        self.ambient_occlusion_state.set_ambient_occlusion(&object, self.queue.clone());
        let offset = 0;
        self.sv_id_to_vec_offset.insert(id as u32, offset);
        self.queue.read().unwrap().write_buffer(&self.sv_index_buffer, id as u64 * std::mem::size_of::<u32>() as u64 * 36, bytemuck::cast_slice(&generate_indices_for_index(id as u32)));
        self.queue.read().unwrap().write_buffer(&self.sv_voxel_buffer, offset as u64, bytemuck::cast_slice(&object.subvoxel_vec));
        self.subvoxel_objects.push(object);
        self.apply_changes_to_sv_data(id);
        return id;
    }

    pub fn rotate(&mut self, subvoxel_id: usize, rotation: Vector3<Deg<f32>>) {
        self.subvoxel_objects.get_mut(subvoxel_id).unwrap().rotate(rotation);
        self.apply_changes_to_sv_data(subvoxel_id);
    }

    pub fn get_subvoxel_object(&self, id: usize) -> &SubvoxelObject {
        self.subvoxel_objects.get(id).unwrap()
    }

    fn apply_changes_to_sv_data(&self, id: usize) {
        let sv_object = self.get_subvoxel_object(id);
        let ao_offset = self.ambient_occlusion_state.subvoxel_id_to_ao_offset.get(&(id as u32)).unwrap();
        let ao_length_in_u32s = (sv_object.subvoxel_vec.len() * 20).div_ceil(32);
        let voxel_offset = self.sv_id_to_vec_offset.get(&(id as u32)).unwrap();
        self.queue.read().unwrap().write_buffer(&self.sv_vertex_buffer, id as u64 * std::mem::size_of::<SubvoxelVertex>() as u64 * 24, bytemuck::cast_slice(&sv_object.subvoxel_vertices));
        self.queue.read().unwrap().write_buffer(&self.sv_data_buffer, id as u64 * std::mem::size_of::<SubvoxelGpuData>() as u64, bytemuck::cast_slice(&[sv_object.to_gpu_data(*ao_offset, ao_length_in_u32s as u32)]));
    }
}