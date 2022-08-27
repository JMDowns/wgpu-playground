struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ChunkPositions {
    chunk_positions: array<vec3<i32>,147>
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
    let chunk_index = (model.data1 & 510u) >> 1u;
    out.clip_position = camera.view_proj * vec4<f32>(f32(model.data0 & 127u) + f32(chunkPositions.chunk_positions[chunk_index][0]*64), f32((model.data0 & 16256u) >> 7u) + f32(chunkPositions.chunk_positions[chunk_index][1]*64), f32((model.data0 & 2080768u) >> 14u) + f32(chunkPositions.chunk_positions[chunk_index][2]*64), 1.0);
    out.tex_coords = vec2<f32>(f32((model.data0 & 65011712u) >> 21u) * 0.0625, f32((model.data0 & 2080374784u) >> 26u) * 0.0625);
    out.ambient_occlusion = f32(((model.data0 & 2147483648u) >> 31u) | ((model.data1 & 1u) << 1u));
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
