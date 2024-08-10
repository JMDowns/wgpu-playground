use cgmath::{Matrix3, Deg, Vector3, Point3, EuclideanSpace};
use crate::gpu_manager::subvoxels::grid_aligned_subvoxel_object::ROTATION;
use fundamentals::world_position::WorldPosition;

pub struct GridAlignedSubvoxelObjectSpecification {
    pub size: Vector3<u32>,
    pub maximal_chunk: WorldPosition,
    pub maximal_block_in_chunk: WorldPosition,
    pub maximal_subvoxel_in_chunk: WorldPosition,
    pub rotation: ROTATION,
    pub model_name: String
}