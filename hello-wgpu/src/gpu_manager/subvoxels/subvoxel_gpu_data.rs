use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct SubvoxelGpuData {
    pub rotation_matrix: [[f32; 3]; 3],
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub subvoxel_size_x: u32,
    pub subvoxel_size_y: u32,
    pub subvoxel_size_z: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub center_z: f32,
    pub ao_offset: u32,
    pub ao_length: u32
}