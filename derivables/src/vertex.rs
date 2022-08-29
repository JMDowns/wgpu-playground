use fundamentals::{consts::*, world_position::WorldPosition, texture_coords::TextureCoordinates};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
data0: u32,
data1: u32,
}
impl Vertex {
        pub fn new(pos: WorldPosition, tc: TextureCoordinates, ambient_occlusion: u8, chunk_index: u32) -> Self {
             
if pos.x > CHUNK_DIMENSION || pos.y > CHUNK_DIMENSION || pos.z > CHUNK_DIMENSION {
            println!("Vertex at {} is outside chunk boundaries", pos);
        }
            let mut data0 = 0;
            data0 = data0 | (pos.x as u32);
            data0 = data0 | (pos.y as u32) << 5;
            data0 = data0 | (pos.z as u32) << 10;
            data0 = data0 | (tc.tx as u32) << 15;
            data0 = data0 | (tc.ty as u32) << 20;
            data0 = data0 | (ambient_occlusion as u32) << 25;
            data0 = data0 | ((chunk_index as u32) & 0b11111 ) << 27;
            let mut data1 = 0;
            data1 = data1 | ((chunk_index as u32) & 0b11100000 ) >> 5;
            Vertex{ data0, data1 }
        }
        pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Uint32,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[u32;1]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Uint32,
                    },
                ]
            }
        }
}
