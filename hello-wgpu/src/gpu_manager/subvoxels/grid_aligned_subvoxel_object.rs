use cgmath::{Vector3, Point3};

use bytemuck::{Zeroable, Pod};
use fundamentals::world_position::WorldPosition;

use super::{grid_aligned_subvoxel_object_specification::GridAlignedSubvoxelObjectSpecification};

#[derive(Clone, Copy)]
pub enum ROTATION {
    FRONT,
    BACK,
    LEFT,
    RIGHT,
    TOP,
    BOTTOM
}

pub struct GridAlignedSubvoxelObject {
    pub id: u32,
    pub size: Vector3<u32>,
    pub maximal_chunk: WorldPosition,
    pub maximal_block_in_chunk: WorldPosition,
    pub maximal_subvoxel_in_chunk: WorldPosition,
    pub rotation: ROTATION,
    pub model_name: String,
}

impl GridAlignedSubvoxelObject {
    pub fn new(id: u32, spec: GridAlignedSubvoxelObjectSpecification) -> GridAlignedSubvoxelObject {
        GridAlignedSubvoxelObject {
            id, 
            size: spec.size,
            maximal_chunk: spec.maximal_chunk,
            maximal_block_in_chunk: spec.maximal_block_in_chunk,
            maximal_subvoxel_in_chunk: spec.maximal_subvoxel_in_chunk,
            model_name: spec.model_name, 
            rotation: spec.rotation
        }
    }

    pub fn into_gpu_data(&self, maximal_chunk_index: u32, model_offset: u32) -> GridAlignedSubvoxelGpuData {
        GridAlignedSubvoxelGpuData {
            size_x: self.size.x,
            size_y: self.size.y,
            size_z: self.size.z,
            maximal_block_x: self.maximal_block_in_chunk.x as u32,
            maximal_block_y: self.maximal_block_in_chunk.y as u32,
            maximal_block_z: self.maximal_block_in_chunk.z as u32,
            maximal_subvoxel_x: self.maximal_subvoxel_in_chunk.x as u32,
            maximal_subvoxel_y: self.maximal_subvoxel_in_chunk.y as u32,
            maximal_subvoxel_z: self.maximal_subvoxel_in_chunk.z as u32,
            model_offset,
            maximal_chunk_index,
            rotation: self.rotation as u32
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct GridAlignedSubvoxelGpuData {
    size_x: u32,
    size_y: u32,
    size_z: u32,
    maximal_chunk_index: u32,
    maximal_block_x: u32,
    maximal_block_y: u32,
    maximal_block_z: u32,
    maximal_subvoxel_x: u32,
    maximal_subvoxel_y: u32,
    maximal_subvoxel_z: u32,
    model_offset: u32,
    rotation: u32,
}