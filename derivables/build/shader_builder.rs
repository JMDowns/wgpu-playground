use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};

use super::vertex_builder::{DATA_TOTAL_BITS, VAR_SIZE_LIST};
use fundamentals::consts::{NUMBER_OF_CHUNKS_AROUND_PLAYER, CHUNK_DIMENSION};

pub fn build_shader_file() {
    let shader_path = Path::new("../hello-wgpu/src/shader.wgsl");
    let mut shader_file = BufWriter::new(File::create(&shader_path).unwrap());

    writeln!(
        &mut shader_file,
         "{}",
         build_shader_string()
    ).unwrap();
}

fn build_shader_string() -> String {
    [
        "struct CameraUniform {",
"    view_proj: mat4x4<f32>,",
"};",
"@group(0) @binding(0)",
"var<uniform> camera: CameraUniform;",
"",
"struct ChunkPositions {",
format!("    chunk_positions: array<i32,{}>", NUMBER_OF_CHUNKS_AROUND_PLAYER * 3).as_str(),
"};",
"@group(2) @binding(0)",
"var<storage> chunkPositions: ChunkPositions;",
"struct VertexInput {",
build_vertex_data().as_str(),
"};",
"",
"@group(3) @binding(0)",
format!("var<storage, read_write> visibility_array: array<u32, {}>;", NUMBER_OF_CHUNKS_AROUND_PLAYER).as_str(),
"struct VertexOutput {",
"    @builtin(position) clip_position:vec4<f32>,",
"    @location(0) tex_index: u32,",
"    @location(1) tex_coords: vec2<f32>,",
"};",
"",
"@vertex",
"fn vs_main(",
"    model: VertexInput,",
") -> VertexOutput {",
"    var out: VertexOutput;",
build_vs_main_statements().as_str(),
"    return out;",
"}",
"",
"@group(1) @binding(0)",
"var diffuse_texture_array: binding_array<texture_2d<f32>>;",
"@group(1) @binding(1)",
"var sampler_diffuse: sampler;",
"",
"@fragment",
"fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {",
"   var tex_color = textureSample(diffuse_texture_array[in.tex_index], sampler_diffuse, in.tex_coords);
    return tex_color;",
"}",
    ].join("\n")
}

fn build_vertex_data() -> String {
    let mut data_vec = Vec::new();
    for i in 0..(DATA_TOTAL_BITS as f32 / 32.0).ceil() as u32 {
        data_vec.push(format!("    @location({i}) data{i}: u32,"));
    }
    data_vec.join("\n")
}

fn build_vs_main_statements() -> String {
    let (data_unpack_vec, chunk_index_statement) = build_data_unpack_vec();
    [
        chunk_index_statement,
        format!("    visibility_array[chunk_index]=0u;"),
        format!("    out.clip_position = camera.view_proj * vec4<f32>(f32({}) + f32(chunkPositions.chunk_positions[3u*chunk_index]*{CHUNK_DIMENSION}), f32({}) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*{CHUNK_DIMENSION}), f32({}) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*{CHUNK_DIMENSION}), 1.0);", data_unpack_vec[0], data_unpack_vec[1], data_unpack_vec[2]),
        format!("    out.tex_index = {};", data_unpack_vec[3]),
        format!("    out.tex_coords = vec2<f32>(f32({}), f32({}));", data_unpack_vec[4], data_unpack_vec[5])
    ].join("\n")
}

pub fn build_data_unpack_vec() -> (Vec<String>, String) {
    let mut data_unpack_vec = Vec::new();
    let mut chunk_index_statement = String::new();
    let mut data_bits_used = 0;

    for (name, size) in VAR_SIZE_LIST.iter() {
        if *name == "chunk_index" {
            if *size == 0 {
                chunk_index_statement = String::from("    let chunk_index = 0u;");
            } else if data_bits_used % 32 + size <= 32 {
                if data_bits_used % 32 == 0 {
                    chunk_index_statement = format!("    let chunk_index = (model.data{} & {}u);", data_bits_used / 32, get_mask(*size, data_bits_used % 32));
                } else {
                    chunk_index_statement = format!("    let chunk_index = (model.data{} & {}u) >> {}u;", data_bits_used / 32, get_mask(*size, data_bits_used % 32), data_bits_used % 32);
                }
                
            } else {
                let data_chunk_1_size = 32 - (data_bits_used % 32);
                let first_or = format!("((model.data{} & {}u) >> {}u)", data_bits_used / 32, get_mask(data_chunk_1_size, data_bits_used % 32), data_bits_used % 32);
                let second_or = format!("((model.data{} & {}u) << {}u)", data_bits_used / 32 + 1, get_mask(size - data_chunk_1_size, 0), data_chunk_1_size);
                chunk_index_statement = format!("    let chunk_index = ({} | {});", first_or, second_or);
            }

        } else {
            if data_bits_used % 32 + size <= 32 {
                if data_bits_used % 32 == 0 {
                    data_unpack_vec.push(format!("(model.data{} & {}u)", data_bits_used / 32, get_mask(*size, 0)));
                } else {
                    data_unpack_vec.push(format!("(model.data{} & {}u) >> {}u", data_bits_used / 32, get_mask(*size, data_bits_used % 32), data_bits_used % 32));
                }
            } else {
                let data_chunk_1_size = 32 - (data_bits_used % 32);
                let first_or = format!("((model.data{} & {}u) >> {}u)", data_bits_used / 32, get_mask(data_chunk_1_size, data_bits_used % 32), data_bits_used % 32);
                let second_or = format!("((model.data{} & {}u) << {}u)", data_bits_used / 32 + 1, get_mask(size - data_chunk_1_size, 0), data_chunk_1_size);
                data_unpack_vec.push(format!("{} | {}", first_or, second_or));
            }
        }

        data_bits_used += size;
    }

    (data_unpack_vec, chunk_index_statement)
}

fn get_mask(number_of_ones: u32, number_of_zeroes: u32) -> u32 {
    let mut digit_place = 0;
    let mut mask = 0;
    for _ in 0..number_of_zeroes {
        digit_place += 1;
    }
    for _ in 0..number_of_ones {
        mask += 2_u32.pow(digit_place);
        digit_place += 1;
    }

    mask
}