pub struct ComputeState {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_input_buffer: wgpu::Buffer,
    pub compute_output_buffer: wgpu::Buffer,
    pub compute_staging_vec: Vec<u32>,
}