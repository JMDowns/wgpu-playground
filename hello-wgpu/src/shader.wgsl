struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ChunkPositions {
    chunk_positions: array<i32,3>
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
    let chunk_index = 0u;
    out.clip_position = camera.view_proj * vec4<f32>(f32((model.data0 & 127u)) + f32(chunkPositions.chunk_positions[3u*chunk_index]*64), f32((model.data0 & 16256u) >> 7u) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*64), f32((model.data0 & 2080768u) >> 14u) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*64), 1.0);
    out.tex_index = (model.data0 & 6291456u) >> 21u;
    out.tex_coords = vec2<f32>(f32((model.data0 & 1065353216u) >> 23u), f32(((model.data0 & 3221225472u) >> 30u) | ((model.data1 & 31u) << 2u)));
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
