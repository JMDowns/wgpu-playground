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
    @location(0) tex_index: u32,
    @location(2) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let chunk_index = 0u;
    out.clip_position = camera.view_proj * vec4<f32>(f32((model.data0 & 31u)) + f32(chunkPositions.chunk_positions[3u*chunk_index]*16), f32((model.data0 & 992u) >> 5u) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*16), f32((model.data0 & 31744u) >> 10u) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*16), 1.0);
    out.tex_index = (model.data0 & 98304u) >> 15u;
    let tex_corner_index = (model.data0 & 393216u) >> 17u;
    switch tex_corner_index { 
        case 0u: {
            out.tex_coords = vec2<f32>(0.0,0.0); 
            break;
        }
        case 1u: {
            out.tex_coords = vec2<f32>(0.0,1.0); 
            break;
        }
        case 2u: {
            out.tex_coords = vec2<f32>(1.0,0.0); 
            break;
        }
        case 3u: {
            out.tex_coords = vec2<f32>(1.0,1.0); 
            break;
        }
        default: {
            out.tex_coords = vec2<f32>(0.0,0.0); 
            break;
        }
    };
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
