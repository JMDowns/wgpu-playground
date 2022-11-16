struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ChunkPositions {
    chunk_positions: array<i32,171>
};
@group(2) @binding(0)
var<storage> chunkPositions: ChunkPositions;
struct VertexInput {
    @location(0) data0: u32,
    @location(1) data1: u32,
};

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) tex_index: u32,
    @location(2) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let chunk_index = (model.data1 & 252u) >> 2u;
    out.clip_position = camera.view_proj * vec4<f32>(f32((model.data0 & 63u)) + f32(chunkPositions.chunk_positions[3u*chunk_index]*32), f32((model.data0 & 4032u) >> 6u) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*32), f32((model.data0 & 258048u) >> 12u) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*32), 1.0);
    out.tex_index = (model.data0 & 3932160u) >> 18u;
    out.tex_coords = vec2<f32>(f32((model.data0 & 264241152u) >> 22u), f32(((model.data0 & 4026531840u) >> 28u) | ((model.data1 & 3u) << 4u)));
    return out;
}

@group(1) @binding(0)
var diffuse_texture_array: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   var tex_color = textureSample(diffuse_texture_array[in.tex_index], sampler_diffuse, in.tex_coords);
    return tex_color;
}
