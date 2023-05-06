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
        },
wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
            ]
        }),

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Indirect Bind Group Layout 1"),
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
        },
wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
            ]
        }),

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Indirect Bind Group Layout 2"),
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
        },
wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 3,
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
        },
wgpu::BindGroupEntry {
            binding: 2,
            resource: indirect_buffers[2].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 3,
            resource: indirect_buffers[3].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 4,
            resource: indirect_buffers[4].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 5,
            resource: indirect_buffers[5].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 6,
            resource: indirect_buffers[6].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 7,
            resource: indirect_buffers[7].as_entire_binding(),
        }
            ]
        }),

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Indirect Bind Group 1"),
            layout: &indirect_bind_group_layouts[1],
            entries: &[
                wgpu::BindGroupEntry {
            binding: 0,
            resource: indirect_buffers[8].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 1,
            resource: indirect_buffers[9].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 2,
            resource: indirect_buffers[10].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 3,
            resource: indirect_buffers[11].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 4,
            resource: indirect_buffers[12].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 5,
            resource: indirect_buffers[13].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 6,
            resource: indirect_buffers[14].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 7,
            resource: indirect_buffers[15].as_entire_binding(),
        }
            ]
        }),

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Indirect Bind Group 2"),
            layout: &indirect_bind_group_layouts[2],
            entries: &[
                wgpu::BindGroupEntry {
            binding: 0,
            resource: indirect_buffers[16].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 1,
            resource: indirect_buffers[17].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 2,
            resource: indirect_buffers[18].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 3,
            resource: indirect_buffers[19].as_entire_binding(),
        }
            ]
        })
        ];
    
        (indirect_bind_groups, indirect_bind_group_layouts)
    }
