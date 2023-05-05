use crate::consts::MAX_MEMORY_USAGE_MB;

use super::consts::{NUM_VERTICES_IN_BUCKET, NUMBER_OF_CHUNKS_AROUND_PLAYER, NUM_BUCKETS_PER_CHUNK, MIN_MEMORY_USAGE_MB};

pub struct BufferSizeFunctionReturn {
    pub vertex_bucket_size: usize,
    pub index_bucket_size: usize,
    pub num_initial_buffers: usize,
    pub number_of_buckets_per_buffer: usize,
    pub num_max_buffers: usize,
}

pub fn return_bucket_buffer_size_and_amount_information(vertex_size_in_bytes: usize) -> BufferSizeFunctionReturn {
    const MAX_BUFFER_SIZE: usize = 1073741824; // 2 GB

    const BUFFER_SIZE: usize = 268435456; // Half a GB per buffer

    let num_initial_buffers: usize = MIN_MEMORY_USAGE_MB as usize / 1024 * 2;
    let num_max_buffers: usize = MAX_MEMORY_USAGE_MB as usize / 1024 * 2;

    let vertex_bucket_size = vertex_size_in_bytes * NUM_VERTICES_IN_BUCKET as usize;
    let number_of_vertex_buckets_per_buffer = std::cmp::min(BUFFER_SIZE / vertex_bucket_size, NUMBER_OF_CHUNKS_AROUND_PLAYER as usize * NUM_BUCKETS_PER_CHUNK);

    // * 3/2 = 6/4 because for every 4 vertices there are 6 indices
    let index_bucket_size = std::mem::size_of::<i32>() * NUM_VERTICES_IN_BUCKET as usize * 3 / 2;
    let number_of_index_buckets_per_buffer = std::cmp::min(BUFFER_SIZE / index_bucket_size, NUMBER_OF_CHUNKS_AROUND_PLAYER as usize * NUM_BUCKETS_PER_CHUNK);

    let number_of_buckets_per_buffer = std::cmp::min(number_of_vertex_buckets_per_buffer, number_of_index_buckets_per_buffer);

    BufferSizeFunctionReturn { vertex_bucket_size, index_bucket_size, num_initial_buffers, num_max_buffers, number_of_buckets_per_buffer }
}