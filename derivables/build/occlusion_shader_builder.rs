use std::{fs::File, io::{BufWriter, Write}, path::Path};

use fundamentals::consts::{NUMBER_OF_CHUNKS_AROUND_PLAYER, CHUNK_DIMENSION};

use crate::shader_builder;

pub fn build_occlusion_shader_file() {
    let occlusion_path = Path::new("../hello-wgpu/src/occlusion_cube.wgsl");
    let mut occlusion_file = BufWriter::new(File::create(&occlusion_path).unwrap());

    writeln!(
        &mut occlusion_file,
         "{}",
         build_occlusion_string()
    ).unwrap();
}

fn build_occlusion_string() -> String {
    let (data_unpack_vec, chunk_index_statement) = shader_builder::build_data_unpack_vec();
    let chunk_pos_length = NUMBER_OF_CHUNKS_AROUND_PLAYER * 3;
    let posx = &data_unpack_vec[0];
    let posy = &data_unpack_vec[1];
    let posz = &data_unpack_vec[2];
    format!("struct CameraUniform {{
        view_proj: mat4x4<f32>,
    }};
    @group(0) @binding(0)
    var<uniform> camera: CameraUniform;
    
    struct ChunkPositions {{
        chunk_positions: array<i32,{chunk_pos_length}>
    }};
    @group(1) @binding(0)
    var<storage> chunkPositions: ChunkPositions;
    struct VertexInput {{
        @location(0) data0: u32,
        @location(1) data1: u32,
    }};
    
    struct VertexOutput {{
        @builtin(position) clip_position:vec4<f32>,
        @location(0) chunk_index: u32,
    }};
    @vertex
    fn vs_main(
        model: VertexInput,
    ) -> VertexOutput {{
        var out: VertexOutput;
        {chunk_index_statement}
        let posx = {posx};
        let posy = {posy};
        let posz = {posz};
        var boundx = f32(posx) + f32(chunkPositions.chunk_positions[3u*chunk_index]*{CHUNK_DIMENSION});
        var boundy = f32(posy) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*{CHUNK_DIMENSION});
        var boundz = f32(posz) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*{CHUNK_DIMENSION});
        if (posx == 0u) {{
            boundx = boundx - 0.1;
        }} else {{
            boundx = boundx + 0.1;
        }}
        if (posy == 0u) {{
            boundy = boundy - 0.1;
        }} else {{
            boundy = boundy + 0.1;
        }}
        if (posz == 0u) {{
            boundz = boundz - 0.1;
        }} else {{
            boundz = boundz + 0.1;
        }}
        out.clip_position = camera.view_proj * vec4<f32>(boundx, boundy, boundz, 1.0);
        out.chunk_index = chunk_index;
        return out;
    }}
    
    @group(2) @binding(0)
    var<storage, read_write> visibility_array: array<u32, {NUMBER_OF_CHUNKS_AROUND_PLAYER}>;
    
    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
        visibility_array[in.chunk_index] = 1u;
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }}")
}