use super::subvoxel_object::SUBVOXEL_PALETTE;
use cgmath::{Matrix3, Deg, Vector3, Point3, EuclideanSpace};

pub struct SubvoxelObjectSpecification {
    pub size: Vector3<f32>,
    pub subvoxel_size: Vector3<u32>,
    pub center: Point3<f32>,
    pub initial_rotation: Vector3<Deg<f32>>,
    pub subvoxel_vec: Vec<SUBVOXEL_PALETTE>
}