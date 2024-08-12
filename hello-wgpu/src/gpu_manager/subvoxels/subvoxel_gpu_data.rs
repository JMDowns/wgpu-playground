use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct SubvoxelGpuData {
    pub rotation_matrix: [[f32; 3]; 3],
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub center_z: f32,
    pub model_offset: u32
}