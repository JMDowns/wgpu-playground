use fundamentals::{world_position::WorldPosition, texture_coords::TextureCoordinates};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
data0: u32,
}
impl Vertex {
        pub fn new(pos: WorldPosition, tc: TextureCoordinates, ambient_occlusion: u8) -> Self {
             
            let mut data0 = 0;
            data0 = data0 | (pos.x as u32);
            data0 = data0 | (pos.y as u32) << 5;
            data0 = data0 | (pos.z as u32) << 10;
            data0 = data0 | (tc.tx as u32) << 15;
            data0 = data0 | (tc.ty as u32) << 18;
            data0 = data0 | (ambient_occlusion as u32) << 21;
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
