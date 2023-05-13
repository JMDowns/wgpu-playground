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
    let bind_group_layouts_string = build_bind_group_layouts_string();
    let bind_groups_string = build_bind_groups_string();

    format!("pub fn return_bind_group_and_layout(device: &wgpu::Device, indirect_buffers: &Vec<wgpu::Buffer>) -> (Vec<wgpu::BindGroup>, Vec<wgpu::BindGroupLayout>) {{
        {bind_group_layouts_string}
    
        {bind_groups_string}
    
        (indirect_bind_groups, indirect_bind_group_layouts)
    }}")
}

fn build_bind_group_layouts_string() -> String {
    let mut indirect_bind_group_layouts_vec = Vec::new();

    let bind_group_layout_entries = build_layout_entries();

    for i in 0..bind_group_layout_entries.len() {
        let entry = &bind_group_layout_entries[i];
        let layout_string = format!("
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {{
            label: Some(\"Indirect Bind Group Layout {i}\"),
            entries: &[
                {entry}
            ]
        }})");
        indirect_bind_group_layouts_vec.push(layout_string);
    }

    let indirect_bind_group_layout_string = indirect_bind_group_layouts_vec.join(",\n");

    format!("
        let indirect_bind_group_layouts = vec![
            {indirect_bind_group_layout_string}
        ];") 
}

fn build_bind_groups_string() -> String {

    let bind_group_entries = build_bind_group_entries();

    let mut indirect_bind_groups_vec = Vec::new();

    for i in 0..bind_group_entries.len() {
        let entry = &bind_group_entries[i];
        let group_string = format!("
        device.create_bind_group(&wgpu::BindGroupDescriptor {{
            label: Some(\"Indirect Bind Group {i}\"),
            layout: &indirect_bind_group_layouts[{i}],
            entries: &[
                {entry}
            ]
        }})");

        indirect_bind_groups_vec.push(group_string);
    }

    let indirect_bind_groups_string = indirect_bind_groups_vec.join(",\n");

    format!("
        let indirect_bind_groups = vec![
            {indirect_bind_groups_string}
        ];") 
}

fn build_layout_entries() -> Vec<String> {
    let buffer_info = buffer_size_function::return_bucket_buffer_size_and_amount_information(super::vertex_builder::return_size_of_vertex_in_bytes());
    let mut bind_group_layout_vec = Vec::new();
    for i in 0..buffer_info.num_max_buffers {
        let binding_num = i % 8;
        let binding = format!("wgpu::BindGroupLayoutEntry {{
            binding: {binding_num},
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

    let chunks = bind_group_layout_vec.chunks(8);

    let mut bind_group_layouts = Vec::new();

    for chunk in chunks {
        bind_group_layouts.push(chunk.iter().map(|s| s.clone()).collect::<Vec<String>>().join(",\n"))
    }

    bind_group_layouts
}

fn build_bind_group_entries() -> Vec<String> {
    let buffer_info = buffer_size_function::return_bucket_buffer_size_and_amount_information(super::vertex_builder::return_size_of_vertex_in_bytes());
    let mut bind_group_entry_vec = Vec::new();
    for i in 0..buffer_info.num_max_buffers {
        let binding_num = i % 8;
        let binding = format!("wgpu::BindGroupEntry {{
            binding: {binding_num},
            resource: indirect_buffers[{i}].as_entire_binding(),
        }}");
        bind_group_entry_vec.push(binding);
    }

    let chunks = bind_group_entry_vec.chunks(8);

    let mut bind_group_entries = Vec::new();

    for chunk in chunks {
        bind_group_entries.push(chunk.iter().map(|s| s.clone()).collect::<Vec<String>>().join(",\n"))
    }

    bind_group_entries
}