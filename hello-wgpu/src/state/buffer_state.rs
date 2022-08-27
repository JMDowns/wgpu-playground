pub struct BufferState {
    pub vertex_buffers: [wgpu::Buffer; 6],
    pub index_buffers: [wgpu::Buffer; 6],
    pub index_buffers_lengths: [u32; 6],
    pub chunk_index_bind_group: wgpu::BindGroup,
    pub chunk_index_buffer: wgpu::Buffer,
}