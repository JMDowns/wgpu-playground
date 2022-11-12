use fundamentals::consts::*;
use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::string::String;

pub const DATA_TOTAL_BITS: u32 = (BITS_PER_POSITION * 3 + BITS_PER_TEX_COORD_X + BITS_PER_TEX_COORD_Y + BITS_PER_AMBIENT_OCCLUSION + BITS_PER_CHUNK_INDEX) as u32;
pub const VAR_SIZE_LIST: [(&str, u32);7] = [
        ("pos.x", BITS_PER_POSITION),
        ("pos.y", BITS_PER_POSITION),
        ("pos.z", BITS_PER_POSITION),
        ("texture_index", BITS_PER_TEX_COORD_X+BITS_PER_TEX_COORD_Y),
        ("u", BITS_PER_POSITION),
        ("v", BITS_PER_POSITION),
        ("chunk_index", BITS_PER_CHUNK_INDEX)
    ];

pub fn return_size_of_vertex_in_bytes() -> usize{
   ((DATA_TOTAL_BITS as f32 / 32.0).ceil() * 4.0) as usize
}

pub fn build_vertex_file() {
    let vertex_path = Path::new("src/vertex.rs");
    let mut vertex_file = BufWriter::new(File::create(&vertex_path).unwrap());

    writeln!(
        &mut vertex_file,
         "{}",
         build_vertex_string()
    ).unwrap();
}

fn build_vertex_string() -> String {
    [
        "use fundamentals::{consts::*, world_position::WorldPosition};",
        "#[repr(C)]",
        "#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]",
        build_vertex_struct().as_str(),
        "impl Vertex {",
        build_vertex_new().as_str(),
        build_vertex_desc().as_str(),
        "}"
    ].join("\n")
}

fn build_vertex_struct() -> String {
    let mut vertex_data = Vec::new();
    let mut data_num = 0;
    while DATA_TOTAL_BITS > data_num * 32 {
        vertex_data.push(format!("data{}: u32,", data_num));
        data_num += 1;
    }
    [
        "pub struct Vertex {",
        vertex_data.join("\n").as_str(),
        "}"
    ].join("\n")
}

fn build_vertex_new() -> String {
    let mut chunk_index_str = ", chunk_index: u32";
    if BITS_PER_CHUNK_INDEX == 0 {
        chunk_index_str = ", _chunk_index: u32";
    }
    [
        &format!("        pub fn new(pos: WorldPosition, texture_index: usize, u: u8, v: u8{chunk_index_str}) -> Self {{"),
        "             ",
        "if pos.x > CHUNK_DIMENSION || pos.y > CHUNK_DIMENSION || pos.z > CHUNK_DIMENSION {
            println!(\"Vertex at {} is outside chunk boundaries\", pos);
        }",
        build_new_bitops().as_str(),
        build_vertex_declaration().as_str(),
        "        }"
    ].join("\n")
}

fn build_new_bitops() -> String {
    let mut data_bits_modified = 0;
    let mut ops_vec = vec![String::from("            let mut data0 = 0;")];
    for (var, size) in VAR_SIZE_LIST.into_iter() {
        if var == "chunk_index" && size == 0 {
            continue;
        }
        if data_bits_modified % 32 + size <= 32 {
            if data_bits_modified % 32 == 0 && data_bits_modified != 0 {
                ops_vec.push(format!("            let mut data{} = 0;", data_bits_modified / 32))
            }
            let mut shift_string = format!(" << {}", data_bits_modified % 32);
            if data_bits_modified % 32 == 0 {
                shift_string = String::new();
            }
            ops_vec.push(format!("            data{} = data{} | ({var} as u32){};", data_bits_modified/32, data_bits_modified/32, shift_string));
        } else {
            let data_chunk_1_size = 32 - (data_bits_modified % 32);
            ops_vec.push(format!("            data{} = data{} | (({var} as u32) & {} ) << {};", data_bits_modified/32, data_bits_modified/32, get_first_chunk_binary_mask(data_chunk_1_size), data_bits_modified % 32));
            ops_vec.push(format!("            let mut data{} = 0;", data_bits_modified / 32 + 1));
            ops_vec.push(format!("            data{} = data{} | (({var} as u32) & {} ) >> {};", data_bits_modified/32+1, data_bits_modified/32+1, get_second_chunk_binary_mask(data_chunk_1_size, size), data_chunk_1_size));
        }

        data_bits_modified += size;
    }
    ops_vec.join("\n")
}

fn get_first_chunk_binary_mask(chunk_size: u32) -> String {
    let mut bit_string = String::from("0b");
    for _ in 0..chunk_size {
        bit_string.push('1');
    }
    bit_string
}

fn get_second_chunk_binary_mask(chunk_size: u32, total_size: u32) -> String {
    let mut bit_string = String::from("0b");
    let positive_chunk = total_size - chunk_size;
    for _ in 0..positive_chunk {
        bit_string.push('1');
    }
    for _ in positive_chunk..total_size {
        bit_string.push('0');
    }
    bit_string
}

fn build_vertex_declaration() -> String {
    let mut data_vec = Vec::new();
    for i in 0..(DATA_TOTAL_BITS as f32 / 32.0).ceil() as u32 {
        data_vec.push(format!("data{}", i))
    }

    format!("            Vertex{{ {} }}", data_vec.join(", "))
}

fn build_vertex_desc() -> String {
    let mut vertex_attributes = Vec::new();
    let mut data_num = 1;
    while DATA_TOTAL_BITS > (data_num-1) * 32 {
        vertex_attributes.push([
            "                    wgpu::VertexAttribute {",
            format!("                        offset: {},", get_offset_string(data_num)).as_str(),
            format!("                        shader_location: {},", data_num-1).as_str(),
            "                        format: wgpu::VertexFormat::Uint32,",
            "                    },"
        ].join("\n"));
        data_num += 1;
    }
    [
        "        pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {",
        "            wgpu::VertexBufferLayout {",
        "                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,",
        "                step_mode: wgpu::VertexStepMode::Vertex,",
        "                attributes: &[",
        vertex_attributes.join("\n").as_str(),
        "                ]",
        "            }",
        "        }",
    ].join("\n")
}

fn get_offset_string(data_num: u32) -> String {
    if data_num - 1 == 0 {
        return String::from("0");
    }

    format!("std::mem::size_of::<[u32;{}]>() as wgpu::BufferAddress", data_num-1)
}