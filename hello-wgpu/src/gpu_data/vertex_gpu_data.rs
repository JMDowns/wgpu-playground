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

    pub fn get_buffers_at_index_i(&self, i: usize) -> [(&wgpu::Buffer, &wgpu::Buffer, u32); 6] {
        [
            (&self.data_front.vertex_buffers[i],
            &self.data_front.index_buffers[i],
            self.data_front.index_lengths[i]),
            (&self.data_back.vertex_buffers[i],
            &self.data_back.index_buffers[i],
            self.data_back.index_lengths[i]),
            (&self.data_left.vertex_buffers[i],
            &self.data_left.index_buffers[i],
            self.data_left.index_lengths[i]),
            (&self.data_right.vertex_buffers[i],
            &self.data_right.index_buffers[i],
            self.data_right.index_lengths[i]),
            (&self.data_top.vertex_buffers[i],
            &self.data_top.index_buffers[i],
            self.data_top.index_lengths[i]),
            (&self.data_bottom.vertex_buffers[i],
            &self.data_bottom.index_buffers[i],
            self.data_bottom.index_lengths[i]),
        ]
    }
}