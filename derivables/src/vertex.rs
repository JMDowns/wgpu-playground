use fundamentals::{consts::*, world_position::WorldPosition};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
data0: u32,
}
impl Vertex {
        pub fn new(pos: WorldPosition, texture_index: usize, u: u8, v: u8, _chunk_index: u32) -> Self {
             
if pos.x > CHUNK_DIMENSION || pos.y > CHUNK_DIMENSION || pos.z > CHUNK_DIMENSION {
            println!("Vertex at {} is outside chunk boundaries", pos);
        }
            let mut data0 = 0;
            data0 = data0 | (pos.x as u32);
            data0 = data0 | (pos.y as u32) << 5;
            data0 = data0 | (pos.z as u32) << 10;
            data0 = data0 | (texture_index as u32) << 15;
            data0 = data0 | (u as u32) << 17;
            data0 = data0 | (v as u32) << 22;
            Vertex{ data0 }
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
                ]
            }
        }
}
