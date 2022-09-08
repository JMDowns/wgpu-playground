struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ChunkPositions {
    chunk_positions: array<vec4<i32>,611>
};
@group(2) @binding(0)
var<storage> chunkPositions: ChunkPositions;
struct VertexInput {
    @location(0) data0: u32,
    @location(1) data1: u32,
};

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ambient_occlusion: f32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let chunk_index = (((model.data0 & 3221225472u) >> 30u) | ((model.data1 & 255u) << 2u));
    out.clip_position = camera.view_proj * vec4<f32>(f32(model.data0 & 63u) + f32(chunkPositions.chunk_positions[chunk_index][0]*32), f32((model.data0 & 4032u) >> 6u) + f32(chunkPositions.chunk_positions[chunk_index][1]*32), f32((model.data0 & 258048u) >> 12u) + f32(chunkPositions.chunk_positions[chunk_index][2]*32), 1.0);
    out.tex_coords = vec2<f32>(f32((model.data0 & 8126464u) >> 18u) * 0.0625, f32((model.data0 & 260046848u) >> 23u) * 0.0625);
    out.ambient_occlusion = f32((model.data0 & 805306368u) >> 28u);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return mix(tex_color, vec4<f32>(0.05, 0.05, 0.05, 0.0), 0.3*in.ambient_occlusion);
}
