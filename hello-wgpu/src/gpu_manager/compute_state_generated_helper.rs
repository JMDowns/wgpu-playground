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
        },
wgpu::BindGroupLayoutEntry {
            binding: 8,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 9,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 10,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 11,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 12,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 13,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 14,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 15,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 16,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 17,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 18,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        },
wgpu::BindGroupLayoutEntry {
            binding: 19,
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
        },
wgpu::BindGroupEntry {
            binding: 8,
            resource: indirect_buffers[8].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 9,
            resource: indirect_buffers[9].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 10,
            resource: indirect_buffers[10].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 11,
            resource: indirect_buffers[11].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 12,
            resource: indirect_buffers[12].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 13,
            resource: indirect_buffers[13].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 14,
            resource: indirect_buffers[14].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 15,
            resource: indirect_buffers[15].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 16,
            resource: indirect_buffers[16].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 17,
            resource: indirect_buffers[17].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 18,
            resource: indirect_buffers[18].as_entire_binding(),
        },
wgpu::BindGroupEntry {
            binding: 19,
            resource: indirect_buffers[19].as_entire_binding(),
        }
            ]
        });
    
        (indirect_bind_group, indirect_bind_group_layout)
    }
