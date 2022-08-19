use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};

use super::vertex_builder::{DATA_TOTAL_BITS, VAR_SIZE_LIST};
use fundamentals::consts::{TEX_MAX_X, TEX_MAX_Y};

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
"struct VertexInput {",
build_vertex_data().as_str(),
"};",
"",
"struct VertexOutput {",
"    @builtin(position) clip_position:vec4<f32>,",
"    @location(0) tex_coords: vec2<f32>,",
"    @location(1) ambient_occlusion: f32,",
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
"var t_diffuse: texture_2d<f32>;",
"@group(1) @binding(1)",
"var s_diffuse: sampler;",
"",
"@fragment",
"fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {",
"    var tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);",
"    return mix(tex_color, vec4<f32>(0.05, 0.05, 0.05, 0.0), 0.3*in.ambient_occlusion);",
"}",
    ].join("\n")
}

fn build_vertex_data() -> String {
    let mut data_vec = Vec::new();
    for i in 0..(DATA_TOTAL_BITS / 32) + 1 {
        data_vec.push(format!("    @location({i}) data{i}: u32,"));
    }
    data_vec.join("\n")
}

fn build_vs_main_statements() -> String {
    let mut data_unpack_vec = Vec::new();
    let mut data_bits_used = 0;
    for (_, size) in VAR_SIZE_LIST.iter() {
        if data_bits_used % 32 + size < 32 {
            if data_bits_used % 32 == 0 {
                data_unpack_vec.push(format!("f32(model.data{} & {}u)", data_bits_used / 32, get_mask(*size, 0)));
            } else {
                data_unpack_vec.push(format!("f32((model.data{} & {}u) >> {}u)", data_bits_used / 32, get_mask(*size, data_bits_used % 32), data_bits_used % 32));
            }
        } else {
            let data_chunk_1_size = 32 - (data_bits_used % 32);
            let first_or = format!("((model.data{} & {}u) >> {})", data_bits_used / 32, get_mask(data_chunk_1_size, data_bits_used % 32), data_bits_used % 32);
            let second_or = format!("((model.data{} & {}u) << {})", data_bits_used / 32, get_mask(size - data_chunk_1_size, 0), size-data_chunk_1_size);
            data_unpack_vec.push(format!("f32({} | {})", first_or, second_or));
        }

        data_bits_used += size;
    }
    [
        format!("    out.clip_position = camera.view_proj * vec4<f32>({}, {}, {}, 1.0);", data_unpack_vec[0], data_unpack_vec[1], data_unpack_vec[2]),
        format!("    out.tex_coords = vec2<f32>({} * {}, {} * {});", data_unpack_vec[3], 1.0 / TEX_MAX_X as f32, data_unpack_vec[4], 1.0 / TEX_MAX_Y as f32),
        format!("    out.ambient_occlusion = {};", data_unpack_vec[5]),
    ].join("\n")
}

fn get_mask(number_of_ones: u8, number_of_zeroes: u8) -> u32 {
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