use derivables::vertex::Vertex;
use wgpu::util::DeviceExt;

pub struct VecVertexIndexLengthsTriple {
    pub vertex_buffers: Vec<Vec<Vertex>>,
    pub index_buffers: Vec<Vec<u32>>,
    pub index_lengths: Vec<u32>
}

impl VecVertexIndexLengthsTriple {

    pub fn add_triple_drain(&mut self, triple: &mut Self) {
        let index_sum: u32 = self.vertex_buffers.iter().map(|buffer| buffer.len() as u32).sum::<u32>();
        self.vertex_buffers.append(&mut triple.vertex_buffers);
        let mut new_index_buffers = triple.index_buffers.iter().map(|indices| indices.into_iter().map(|index| (*index)+index_sum).collect()).collect();
        self.index_buffers.append(&mut new_index_buffers);
        self.index_lengths.append(&mut triple.index_lengths);
    }

    pub fn generate_vertex_buffer(&self, device: &wgpu::Device, label: &str) -> wgpu::Buffer {
        let flat_vertex_vec = self.vertex_buffers.iter().cloned().flatten().collect::<Vec<Vertex>>();
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&String::from(format!("{} Vertex", label))),
                contents: bytemuck::cast_slice(&flat_vertex_vec),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }

    pub fn generate_index_buffer(&self, device: &wgpu::Device, label: &str) -> wgpu::Buffer {
        let flat_index_vec = self.index_buffers.iter().cloned().flatten().collect::<Vec<u32>>();
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&String::from(format!("{} Index", label))),
                contents: bytemuck::cast_slice(&flat_index_vec),
                usage: wgpu::BufferUsages::INDEX,
            }
        )
    }

    pub fn generate_index_buffer_length(&self) -> u32 {
        self.index_lengths.iter().sum()
    }
}