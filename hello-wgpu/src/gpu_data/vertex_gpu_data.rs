use super::vec_vertex_index_length_triple::VecVertexIndexLengthsTriple;

pub struct VertexGPUData {
    pub data_front: VecVertexIndexLengthsTriple,
    pub data_back: VecVertexIndexLengthsTriple,
    pub data_left: VecVertexIndexLengthsTriple,
    pub data_right: VecVertexIndexLengthsTriple,
    pub data_top: VecVertexIndexLengthsTriple,
    pub data_bottom: VecVertexIndexLengthsTriple,
}

impl VertexGPUData {
    pub fn add_gpu_data_drain(&mut self, other_gpu_data: &mut Self) {
        self.data_front.add_triple_drain(&mut other_gpu_data.data_front);
        self.data_back.add_triple_drain(&mut other_gpu_data.data_back);
        self.data_left.add_triple_drain(&mut other_gpu_data.data_left);
        self.data_right.add_triple_drain(&mut other_gpu_data.data_right);
        self.data_top.add_triple_drain(&mut other_gpu_data.data_top);
        self.data_bottom.add_triple_drain(&mut other_gpu_data.data_bottom);
    }

    pub fn generate_vertex_buffers(&self, device: &wgpu::Device) -> [wgpu::Buffer; 6] {
        [ 
            self.data_front.generate_vertex_buffer(device, "Front mesh"),
            self.data_back.generate_vertex_buffer(device, "Back mesh"),
            self.data_left.generate_vertex_buffer(device, "Left mesh"),
            self.data_right.generate_vertex_buffer(device, "Right mesh"),
            self.data_top.generate_vertex_buffer(device, "Top mesh"),
            self.data_bottom.generate_vertex_buffer(device, "Bottom mesh"),
        ]
    }

    pub fn generate_index_buffers(&self, device: &wgpu::Device) -> [wgpu::Buffer; 6] {
        [ 
            self.data_front.generate_index_buffer(device, "Front mesh"),
            self.data_back.generate_index_buffer(device, "Back mesh"),
            self.data_left.generate_index_buffer(device, "Left mesh"),
            self.data_right.generate_index_buffer(device, "Right mesh"),
            self.data_top.generate_index_buffer(device, "Top mesh"),
            self.data_bottom.generate_index_buffer(device, "Bottom mesh"),
        ]
    }

    pub fn generate_index_buffer_lengths(&self) -> [u32; 6] {
        [
            self.data_front.generate_index_buffer_length(),
            self.data_back.generate_index_buffer_length(),
            self.data_left.generate_index_buffer_length(),
            self.data_right.generate_index_buffer_length(),
            self.data_top.generate_index_buffer_length(),
            self.data_bottom.generate_index_buffer_length(),
        ]
    }
}