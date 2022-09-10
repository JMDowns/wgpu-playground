use wgpu::Buffer;

pub struct VecVertexIndexLengthsTriple {
    pub vertex_buffers: Vec<Buffer>,
    pub index_buffers: Vec<Buffer>,
    pub index_lengths: Vec<u32>
}

impl VecVertexIndexLengthsTriple {
    pub fn new() -> Self {
        Self {
            vertex_buffers: Vec::new(),
            index_buffers: Vec::new(),
            index_lengths: Vec::new(),
        }
    }
}