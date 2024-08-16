pub fn return_bind_group_and_layout(device: &wgpu::Device, indirect_buffers: &Vec<wgpu::Buffer>) -> (Vec<wgpu::BindGroup>, Vec<wgpu::BindGroupLayout>) {
        
        let indirect_bind_group_layouts = vec![
            
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Indirect Bind Group Layout 0"),
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
        },
wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
            ]
        })
        ];
    
        
        let indirect_bind_groups = vec![
            
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Indirect Bind Group 0"),
            layout: &indirect_bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
            binding: 0,
            resource: indirect_buffers[0].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 1,
            resource: indirect_buffers[1].as_entire_binding(),
        }
            ]
        })
        ];
    
        (indirect_bind_groups, indirect_bind_group_layouts)
    }
