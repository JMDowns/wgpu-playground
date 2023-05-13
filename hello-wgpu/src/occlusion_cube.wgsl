struct CameraUniform {
        view_proj: mat4x4<f32>,
    };
    @group(0) @binding(0)
    var<uniform> camera: CameraUniform;
    
    struct ChunkPositions {
        chunk_positions: array<i32,21>
    };
    @group(1) @binding(0)
    var<storage> chunkPositions: ChunkPositions;
    struct VertexInput {
        @location(0) data0: u32,
        @location(1) data1: u32,
    };
    
    struct VertexOutput {
        @builtin(position) clip_position:vec4<f32>,
        @location(0) chunk_index: u32,
    };
    @vertex
    fn vs_main(
        model: VertexInput,
    ) -> VertexOutput {
        var out: VertexOutput;
            let chunk_index = (model.data1 & 112u) >> 4u;
        let posx = (model.data0 & 63u);
        let posy = (model.data0 & 4032u) >> 6u;
        let posz = (model.data0 & 258048u) >> 12u;
        var boundx = f32(posx) + f32(chunkPositions.chunk_positions[3u*chunk_index]*32);
        var boundy = f32(posy) + f32(chunkPositions.chunk_positions[3u*chunk_index+1u]*32);
        var boundz = f32(posz) + f32(chunkPositions.chunk_positions[3u*chunk_index+2u]*32);
        if (posx == 0u) {
            boundx = boundx - 0.1;
        } else {
            boundx = boundx + 0.1;
        }
        if (posy == 0u) {
            boundy = boundy - 0.1;
        } else {
            boundy = boundy + 0.1;
        }
        if (posz == 0u) {
            boundz = boundz - 0.1;
        } else {
            boundz = boundz + 0.1;
        }
        out.clip_position = camera.view_proj * vec4<f32>(boundx, boundy, boundz, 1.0);
        out.chunk_index = chunk_index;
        return out;
    }
    
    @group(2) @binding(0)
    var<storage, read_write> visibility_array: array<u32, 7>;
    
    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        visibility_array[in.chunk_index] = 1u;
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
