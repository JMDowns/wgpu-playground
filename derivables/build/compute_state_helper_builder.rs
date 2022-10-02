use std::{path::Path, fs::File};

use std::io::{BufWriter, Write};

use fundamentals::buffer_size_function;

pub fn build_compute_helper_file() {
    let compute_helper_path = Path::new("../hello-wgpu/src/gpu_manager/compute_state_generated_helper.rs");
    let mut compute_helper_file = BufWriter::new(File::create(&compute_helper_path).unwrap());

    writeln!(
        &mut compute_helper_file,
         "{}",
         build_compute_helper_string()
    ).unwrap();
}

fn build_compute_helper_string() -> String {
    let bind_group_layout_entries = build_layout_entries();
    let bind_group_entries = build_bind_group_entries();
    format!("pub fn return_bind_group_and_layout(device: &wgpu::Device, indirect_buffers: &Vec<wgpu::Buffer>) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {{
        let indirect_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {{
            entries: &[
                {bind_group_layout_entries}
            ],
            label: Some(\"Indirect Bind Group Layout\")
        }});
    
        let indirect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {{
            label: Some(\"Indirect Bind Group\"),
            layout: &indirect_bind_group_layout,
            entries: &[
                {bind_group_entries}
            ]
        }});
    
        (indirect_bind_group, indirect_bind_group_layout)
    }}")
}

fn build_layout_entries() -> String {
    let buffer_info = buffer_size_function::return_bucket_buffer_size_and_amount_information(super::vertex_builder::return_size_of_vertex_in_bytes());
    let mut bind_group_layout_vec = Vec::new();
    for i in 0..buffer_info.number_of_buffers {
        let binding = format!("wgpu::BindGroupLayoutEntry {{
            binding: {i},
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {{ 
                ty: wgpu::BufferBindingType::Storage {{ read_only: false }},
                has_dynamic_offset: false,
                min_binding_size: None,
            }},
            count: None
        }}");
        bind_group_layout_vec.push(binding);
    }

    bind_group_layout_vec.join(",\n")
}

fn build_bind_group_entries() -> String {
    let buffer_info = buffer_size_function::return_bucket_buffer_size_and_amount_information(super::vertex_builder::return_size_of_vertex_in_bytes());
    let mut bind_group_entry_vec = Vec::new();
    for i in 0..buffer_info.number_of_buffers {
        let binding = format!("wgpu::BindGroupEntry {{
            binding: {i},
            resource: indirect_buffers[{i}].as_entire_binding(),
        }}");
        bind_group_entry_vec.push(binding);
    }

    bind_group_entry_vec.join(",\n")
}