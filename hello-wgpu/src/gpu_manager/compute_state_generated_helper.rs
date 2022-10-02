pub fn return_bind_group_and_layout(device: &wgpu::Device, indirect_buffers: &Vec<wgpu::Buffer>) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let indirect_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
            ],
            label: Some("Indirect Bind Group Layout")
        });
    
        let indirect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Indirect Bind Group"),
            layout: &indirect_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
            binding: 0,
            resource: indirect_buffers[0].as_entire_binding(),
        }
            ]
        });
    
        (indirect_bind_group, indirect_bind_group_layout)
    }
