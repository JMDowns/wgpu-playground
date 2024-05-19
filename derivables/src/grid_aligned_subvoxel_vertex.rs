use fundamentals::{consts::*, world_position::WorldPosition};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridAlignedSubvoxelVertex {
data0: u32,
}

const NUM_UNIQUE_GRID_ALIGNED_SUBVOXELS: u32 = 1024;
const BITS_FOR_SUBVOXEL_ID: u32 = 10;

impl GridAlignedSubvoxelVertex {
        pub fn new(gas_id: u32, vertex_orientation: u32) -> Self {
            let mut data0 = 0;
            data0 = data0 | (gas_id as u32);
            data0 = data0 | (vertex_orientation as u32) << BITS_FOR_SUBVOXEL_ID;
            GridAlignedSubvoxelVertex{ data0 }
        }
        pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GridAlignedSubvoxelVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Uint32,
                    }
                ]
            }
        }
}

pub fn generate_ga_subvoxel_cube_vertices(gas_id: u32) -> [GridAlignedSubvoxelVertex; 8] {
    [
        GridAlignedSubvoxelVertex::new(gas_id, 0),
        GridAlignedSubvoxelVertex::new(gas_id, 1),
        GridAlignedSubvoxelVertex::new(gas_id, 2),
        GridAlignedSubvoxelVertex::new(gas_id, 3),
        GridAlignedSubvoxelVertex::new(gas_id, 4),
        GridAlignedSubvoxelVertex::new(gas_id, 5),
        GridAlignedSubvoxelVertex::new(gas_id, 6),
        GridAlignedSubvoxelVertex::new(gas_id, 7),
    ]
}

pub fn generate_ga_subvoxel_cube_indices(gas_id: u32) -> [u32; 36] {
    [
        0,1,2,
        0,2,3,

        5,4,7,
        5,7,6,

        4,0,3,
        4,3,7,

        1,5,6,
        1,6,2,

        5,1,0,
        5,0,4,
        
        2,6,7,
        2,7,3,
    ].map(|index| index + gas_id * 8)
}

pub const INDICES_CUBE_LEN: u32 = 36;
