use std::collections::HashMap;
use derivables::grid_aligned_subvoxel_vertex::{generate_ga_subvoxel_cube_indices, generate_ga_subvoxel_cube_vertices, GridAlignedSubvoxelVertex};
use fundamentals::consts::{MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS, MAX_SUBVOXEL_COLORS, MAX_SUBVOXEL_OBJECTS};
use fundamentals::world_position::WorldPosition;
use num_traits::Zero;
use wgpu::{Device, util::DeviceExt, Queue, BufferUsages};
use cgmath::{EuclideanSpace, Matrix3, Point3, Rad, Vector3, Deg, Vector4};
use derivables::subvoxel_vertex::{generate_cube_at_center, generate_indices_for_index, SubvoxelVertex};
use bytemuck::{Zeroable, Pod};
use std::sync::{Arc, RwLock};

use crate::gpu_manager::chunk_index_state::ChunkIndexState;
use crate::gpu_manager::gpu_data::vertex_gpu_data::BucketPosition;
use crate::voxels::chunk::Chunk;

use super::ambient_occlusion_state::AmbientOcclusionState;

use super::grid_aligned_subvoxel_object::{GridAlignedSubvoxelGpuData, GridAlignedSubvoxelObject, ROTATION};
use super::grid_aligned_subvoxel_object_specification::GridAlignedSubvoxelObjectSpecification;
use super::model_manager::{self, SubvoxelModelManager, SubvoxelModel};
use super::subvoxel_object::{SubvoxelObject};
use super::subvoxel_object_specification::SubvoxelObjectSpecification;
use super::subvoxel_gpu_data::SubvoxelGpuData;

pub struct SubvoxelState {
    pub subvoxel_objects: Vec<SubvoxelObject>,
    pub grid_aligned_subvoxel_objects: Vec<GridAlignedSubvoxelObject>,
    pub sv_data_buffer: wgpu::Buffer,
    pub sv_palette_buffer: wgpu::Buffer,
    pub sv_vertex_buffer: wgpu::Buffer,
    pub sv_index_buffer: wgpu::Buffer,
    pub sv_grid_aligned_object_buffer: wgpu::Buffer,
    pub sv_grid_aligned_vertex_buffer: wgpu::Buffer,
    pub sv_grid_aligned_index_buffer: wgpu::Buffer,
    pub sv_grid_aligned_bind_group_layout: wgpu::BindGroupLayout,
    pub sv_grid_aligned_bind_group: wgpu::BindGroup,
    pub subvoxel_bind_group_layout: wgpu::BindGroupLayout,
    pub subvoxel_bind_group: wgpu::BindGroup,
    pub queue: Arc<RwLock<Queue>>,
    pub sv_id_to_vec_offset: HashMap<u32, u32>,
    pub chunk_position_to_buckets: HashMap<WorldPosition, Vec<GridAlignedSubvoxelVertexBucket>>,
    pub model_manager: SubvoxelModelManager,
    pub chunk_index_state: Arc<RwLock<ChunkIndexState>>
}

pub struct GridAlignedSubvoxelVertexBucket {
    pub offset: u32,
    pub num_subvoxels_in_bucket: u32,
}

pub struct VoxelBufferSpace {
    pub offset_in_u32s: u32,
    pub length_in_u32s: u32
}

impl SubvoxelState {
    pub fn new(device: &Device, queue: Arc<RwLock<Queue>>, chunk_index_state: Arc<RwLock<ChunkIndexState>>) -> Self {
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

        let sv_grid_aligned_bind_group_layout =
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
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                label: Some("grid_aligned_subvoxel_bind_group_layout"),
            });

        let sv_data_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Data Buffer"),
                size: (std::mem::size_of::<SubvoxelGpuData>()) as u64  * MAX_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_palette_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Subvoxel Palette Buffer"),
                size: (std::mem::size_of::<f32>() * 4) as u64 * MAX_SUBVOXEL_COLORS,
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

        let sv_grid_aligned_vertex_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Grid-Aligned Cube Vertex Buffer"),
                size: (std::mem::size_of::<GridAlignedSubvoxelVertex>() * 24) as u64 * MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_grid_aligned_index_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Grid-Aligned Cube Index Buffer"),
                size: (std::mem::size_of::<u32>() * 36) as u64 * MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sv_grid_aligned_object_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Grid-Aligned Cube Object Buffer"),
                size: std::mem::size_of::<GridAlignedSubvoxelObject>() as u64 * MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let model_manager = SubvoxelModelManager::new(device, queue.clone());

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
                    resource: model_manager.sv_model_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sv_palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: model_manager.ao_state.ao_buffer.as_entire_binding()
                }
            ]
        });

        let sv_grid_aligned_bind_group = device.create_bind_group( &wgpu::BindGroupDescriptor {
            label: Some("Grid-Aligned Subvoxel Bind Group"),
            layout: &sv_grid_aligned_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_manager.sv_model_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_manager.ao_state.ao_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sv_grid_aligned_object_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: sv_palette_buffer.as_entire_binding(),
                },
            ]
        });

        Self {
            subvoxel_objects: Vec::new(),
            grid_aligned_subvoxel_objects: Vec::new(),
            sv_id_to_vec_offset: HashMap::new(),
            sv_data_buffer,
            sv_palette_buffer,
            sv_vertex_buffer,
            sv_index_buffer,
            sv_grid_aligned_object_buffer,
            sv_grid_aligned_vertex_buffer,
            sv_grid_aligned_index_buffer,
            sv_grid_aligned_bind_group_layout,
            sv_grid_aligned_bind_group,
            subvoxel_bind_group_layout,
            subvoxel_bind_group,
            queue,
            chunk_position_to_buckets: HashMap::new(),
            model_manager,
            chunk_index_state
        }
    }

    pub fn add_grid_aligned_subvoxel_object(&mut self, spec: GridAlignedSubvoxelObjectSpecification) {
        let object_id = self.grid_aligned_subvoxel_objects.len() as u32;
        let model_offset = self.model_manager.get_model_offset(&spec.model_name);
        let maximal_chunk_index = *(self.chunk_index_state.read().unwrap().pos_to_gpu_index.get(&spec.maximal_chunk).unwrap());
        
        let object = GridAlignedSubvoxelObject::new(object_id, spec);
        self.queue.read().unwrap().write_buffer(&self.sv_grid_aligned_vertex_buffer, object_id as u64 * std::mem::size_of::<GridAlignedSubvoxelVertex>() as u64 * 8, bytemuck::cast_slice(&generate_ga_subvoxel_cube_vertices(object_id)));
        self.queue.read().unwrap().write_buffer(&self.sv_grid_aligned_index_buffer, object_id as u64 * std::mem::size_of::<u32>() as u64 * 36, bytemuck::cast_slice(&generate_ga_subvoxel_cube_indices(object_id as u32)));
        self.queue.read().unwrap().write_buffer(&self.sv_grid_aligned_object_buffer, object_id as u64 * std::mem::size_of::<GridAlignedSubvoxelGpuData>() as u64, bytemuck::cast_slice(&[object.into_gpu_data(maximal_chunk_index as u32, model_offset)]));
        
        self.grid_aligned_subvoxel_objects.push(object);
    }

    pub fn add_subvoxel_object(&mut self, spec: SubvoxelObjectSpecification) -> usize {
        let id = self.subvoxel_objects.len();
        let object = SubvoxelObject::new(id as u32, spec);
        self.queue.read().unwrap().write_buffer(&self.sv_index_buffer, id as u64 * std::mem::size_of::<u32>() as u64 * 36, bytemuck::cast_slice(&generate_indices_for_index(id as u32)));
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
        let model_offset = self.model_manager.get_model_offset(&sv_object.model_name);
        self.queue.read().unwrap().write_buffer(&self.sv_vertex_buffer, id as u64 * std::mem::size_of::<SubvoxelVertex>() as u64 * 24, bytemuck::cast_slice(&sv_object.subvoxel_vertices));
        self.queue.read().unwrap().write_buffer(&self.sv_data_buffer, id as u64 * std::mem::size_of::<SubvoxelGpuData>() as u64, bytemuck::cast_slice(&[sv_object.to_gpu_data(model_offset)]));
    }

    pub fn register_model(&mut self, model: SubvoxelModel) {
        self.model_manager.add_model(model);
    }
}