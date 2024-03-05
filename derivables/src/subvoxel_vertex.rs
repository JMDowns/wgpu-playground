use fundamentals::{enums::block_side::BlockSide, consts::LEFT_KEY};
use cgmath;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SubvoxelVertex {
    position: [f32; 3],
    side: i32
}

impl SubvoxelVertex {
    pub fn rotate(&self, rotation_matrix: &cgmath::Matrix3<f32>) -> SubvoxelVertex {
        SubvoxelVertex {
            position: (rotation_matrix * cgmath::Vector3::<f32>::from(self.position)).into(),
            side: self.side
        }
    }

    pub fn translate(&self, translation: cgmath::Vector3<f32>) -> SubvoxelVertex {
        SubvoxelVertex {
            position: (cgmath::Point3::<f32>::from(self.position) + translation).into(),
            side: self.side
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SubvoxelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Sint32,
                }
            ]
        }
    }
}

pub fn generate_cube(x: f32,y: f32, z: f32, scale: cgmath::Vector3<f32>) -> [SubvoxelVertex; 24] {
    [
        SubvoxelVertex { position: [x, y, z], side: BlockSide::LEFT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y, z], side: BlockSide::LEFT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z], side: BlockSide::LEFT as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z], side: BlockSide::LEFT as i32},

        SubvoxelVertex { position: [x, y, z-(scale.z as f32)], side: BlockSide::RIGHT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y, z-(scale.z as f32)], side: BlockSide::RIGHT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::RIGHT as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::RIGHT as i32},

        SubvoxelVertex { position: [x-(scale.x as f32), y, z], side: BlockSide::FRONT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z], side: BlockSide::FRONT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y, z-(scale.z as f32)], side: BlockSide::FRONT as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::FRONT as i32},

        SubvoxelVertex { position: [x, y, z], side: BlockSide::BACK as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z], side: BlockSide::BACK as i32},
        SubvoxelVertex { position: [x, y, z-(scale.z as f32)], side: BlockSide::BACK as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::BACK as i32},

        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z], side: BlockSide::BOTTOM as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z], side: BlockSide::BOTTOM as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::BOTTOM as i32},
        SubvoxelVertex { position: [x, y-(scale.y as f32), z-(scale.z as f32)], side: BlockSide::BOTTOM as i32},

        SubvoxelVertex { position: [x, y, z], side: BlockSide::TOP as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y, z], side: BlockSide::TOP as i32},
        SubvoxelVertex { position: [x, y, z-(scale.z as f32)], side: BlockSide::TOP as i32},
        SubvoxelVertex { position: [x-(scale.x as f32), y, z-(scale.z as f32)], side: BlockSide::TOP as i32}
    ]
}

pub fn generate_cube_at_center(center: cgmath::Point3<f32>, scale: cgmath::Vector3<f32>) -> [SubvoxelVertex; 24] {
    let radius = scale / 2.;
    generate_cube(center.x + radius.x, center.y + radius.y, center.z + radius.z, scale)
}

pub const INDICES_CUBE: &[u32] = &[
     // Front Face
     0,1,2,
     0,2,3,
     // Back Face
     5,4,7,
     5,7,6,
     // Right Face
     8,10,11,
     8,11,9,
     // Left Face
     14,12,13,
     14,13,15,
     // Bottom Face
     16,18,19,
     16,19,17,
     // Top Face
     23,21,20,
     23,20,22,
];

pub const INDICES_CUBE_LEN: u32 = INDICES_CUBE.len() as u32;