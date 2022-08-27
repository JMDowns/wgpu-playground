use derivables::vertex::Vertex;
use wgpu::util::DeviceExt;
use wgpu::Buffer;

pub struct VecVertexIndexLengthsTriple {
    pub vertex_buffers: Vec<Buffer>,
    pub index_buffers: Vec<Buffer>,
    pub index_lengths: Vec<u32>
}

impl VecVertexIndexLengthsTriple {

    pub fn add_triple_drain(&mut self, triple: &mut Self) {
        self.vertex_buffers.append(&mut triple.vertex_buffers);
        self.index_buffers.append(&mut triple.index_buffers);
        self.index_lengths.append(&mut triple.index_lengths);
    }
}