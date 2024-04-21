use cgmath::{Vector3, Point3};

use bytemuck::{Zeroable, Pod};
use fundamentals::world_position::WorldPosition;

use super::subvoxel_object::SUBVOXEL_PALETTE;

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
    pub fn new(id: u32, size: Vector3<u32>, maximal_chunk: WorldPosition, maximal_block_in_chunk: WorldPosition, maximal_subvoxel_in_chunk: WorldPosition, model_name: &str, rotation: ROTATION) -> Self {
        GridAlignedSubvoxelObject {
            id,
            size,
            maximal_chunk,
            maximal_block_in_chunk,
            maximal_subvoxel_in_chunk,
            rotation,
            model_name: String::from(model_name)
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