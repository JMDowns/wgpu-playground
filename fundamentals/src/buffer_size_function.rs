use super::consts::{NUM_VERTICES_IN_BUCKET, NUM_BUCKETS, NUMBER_OF_CHUNKS_AROUND_PLAYER, NUM_BUCKETS_PER_CHUNK};

pub struct BufferSizeFunctionReturn {
    pub vertex_bucket_size: usize,
    pub index_bucket_size: usize,
    pub number_of_buffers: usize,
    pub number_of_buckets_per_buffer: usize,
    pub number_of_buckets_in_last_buffer: usize,
}

pub fn return_bucket_buffer_size_and_amount_information(vertex_size_in_bytes: usize) -> BufferSizeFunctionReturn {
    const MAX_BUFFER_SIZE: usize = 1073741824;

    let vertex_bucket_size = vertex_size_in_bytes * NUM_VERTICES_IN_BUCKET as usize;
    let number_of_vertex_buckets_per_buffer = std::cmp::min(MAX_BUFFER_SIZE / vertex_bucket_size, NUMBER_OF_CHUNKS_AROUND_PLAYER as usize * NUM_BUCKETS_PER_CHUNK);
    let mut number_of_vertex_pool_buffers = NUM_BUCKETS / number_of_vertex_buckets_per_buffer;
    if NUM_BUCKETS % number_of_vertex_buckets_per_buffer != 0 {
        number_of_vertex_pool_buffers += 1;
    }

    // * 3/2 = 6/4 because for every 4 vertices there are 6 indices
    let index_bucket_size = std::mem::size_of::<i32>() * NUM_VERTICES_IN_BUCKET as usize * 3 / 2;
    let number_of_index_buckets_per_buffer = std::cmp::min(MAX_BUFFER_SIZE / index_bucket_size, NUMBER_OF_CHUNKS_AROUND_PLAYER as usize * NUM_BUCKETS_PER_CHUNK);
    let mut number_of_index_pool_buffers = NUM_BUCKETS / number_of_index_buckets_per_buffer;
    if NUM_BUCKETS % number_of_index_buckets_per_buffer != 0 {
        number_of_index_pool_buffers += 1;
    }

    let number_of_buckets_per_buffer = std::cmp::min(number_of_vertex_buckets_per_buffer, number_of_index_buckets_per_buffer);
    let number_of_buffers = std::cmp::max(number_of_vertex_pool_buffers, number_of_index_pool_buffers);
    let mut number_of_buckets_in_last_buffer = NUM_BUCKETS % number_of_buckets_per_buffer;
    if number_of_buckets_in_last_buffer == 0 {
        number_of_buckets_in_last_buffer = number_of_buckets_per_buffer;
    }

    BufferSizeFunctionReturn { vertex_bucket_size, index_bucket_size, number_of_buffers, number_of_buckets_per_buffer, number_of_buckets_in_last_buffer }
}