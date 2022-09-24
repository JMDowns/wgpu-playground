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
    let chunk_index = 0u;
    var chunk_position = vec3<i32>(chunkPositions.chunk_positions[3u*chunk_index], chunkPositions.chunk_positions[3u*chunk_index+1u], chunkPositions.chunk_positions[3u*chunk_index+2u]);
    out.clip_position = camera.view_proj * vec4<f32>(f32(model.data0 & 63u) + f32(chunk_position.x*32), f32((model.data0 & 4032u) >> 6u) + f32(chunk_position.y*32), f32((model.data0 & 258048u) >> 12u) + f32(chunk_position.z*32), 1.0);
    out.tex_coords = vec2<f32>(f32((model.data0 & 1835008u) >> 18u) * 0.25, f32((model.data0 & 14680064u) >> 21u) * 0.25);
    out.ambient_occlusion = f32((model.data0 & 50331648u) >> 24u);
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
