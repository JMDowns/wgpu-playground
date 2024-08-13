// Steps:
// Figure out bit structure of vertices, and input data to vertex builder
// Write bitpacking info in a file somewhere so shader builder and vertex builder can read it
// Format
// DATA_NAME BIT_START BIT_LENGTH BIT_SUBSLICE_START(OPTIONAL) BIT_SUBSLICE_LENGTH(OPTIONAL)
// Repeat above
// So, for a grid aligned vertex with 1024 (10 bits) of gas ids, and 3 bits of vertex orientation, we have
// GAS_ID 0 10
// VERTEX_ORIENTATION 10 3 

use std::fs::File;
use std::io;
use std::path::Path;
use std::fmt;
use std::io::Write;
use std::str::FromStr;
use std::thread::current;
use fundamentals::bitpacking_spec::BitpackingSpec;

fn write_bitpacking_specs<P: AsRef<Path>>(named_specs: Vec<(String, Vec<BitpackingSpec>)>, file_path: P) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    
    for (spec_name, specs) in named_specs {
        writeln!(file, "MODEL NAME {}", spec_name)?;
        for spec in specs {
            writeln!(file, "{}", spec)?;
        }
    }
    
    Ok(())
}

fn calculate_bits_needed(max_value: usize) -> usize {
    (max_value as f64).log2().ceil() as usize
}

fn create_bitpacking_specs(max_values: &[(String, usize)]) -> Vec<BitpackingSpec> {
    let mut specs = Vec::new();
    let mut current_bit = 0;
    let chunk_size = 32;

    for (data_name, max_value) in max_values {
        let bits_needed = calculate_bits_needed(*max_value);
        let mut bits_remaining = bits_needed;
        let mut subslice_start = 0;
        let mut has_sliced = false;

        while bits_remaining > 0 {
            if (current_bit % chunk_size) + bits_remaining <= chunk_size {
                specs.push(BitpackingSpec {
                    data_name: data_name.clone(),
                    bit_start: current_bit,
                    bit_length: bits_remaining,
                    bit_subslice_start: if has_sliced { Some(subslice_start) }  else { None },
                    bit_subslice_length: if has_sliced { Some(bits_remaining) }  else { None },
                });
                current_bit = (current_bit + bits_remaining);
                bits_remaining = 0;
            } else {
                has_sliced = true;
                let remaining_bits_in_chunk = chunk_size - current_bit;
                specs.push(BitpackingSpec {
                    data_name: data_name.clone(),
                    bit_start: current_bit,
                    bit_length: remaining_bits_in_chunk,
                    bit_subslice_start: Some(subslice_start),
                    bit_subslice_length: Some(remaining_bits_in_chunk),
                });
                subslice_start += remaining_bits_in_chunk;
                bits_remaining -= remaining_bits_in_chunk;
                current_bit = ((current_bit + chunk_size - 1) / chunk_size) * chunk_size;
            }
        }
    }

    specs
}

fn get_number_u32s_for_bitpacking_spec(specs: &Vec<BitpackingSpec>) -> usize {
    let mut num_bits = 0;
    for spec in specs {
        num_bits += spec.bit_length;
    }

    return num_bits.div_ceil(32);
}

fn get_grid_aligned_subvoxel_spec() -> (String, Vec<BitpackingSpec>) {
    return ("GRID_ALIGNED_SUBVOXEL_VERTEX".to_string(), create_bitpacking_specs(&[
        (String::from("GAS_ID"), fundamentals::consts::MAX_GRID_ALIGNED_SUBVOXEL_OBJECTS as usize), 
        (String::from("VERTEX_ORIENTATION"), 8)
        ]));
}

pub fn write_specs() {
    let grid_aligned_subvoxel_spec = get_grid_aligned_subvoxel_spec();
    write_bitpacking_specs(vec![grid_aligned_subvoxel_spec], "output.txt");
}

// Since we pack data in 32-bit integers, we need to slice data that's over the 32 bit line
// The BIT_SUBSLICE parameters are used here to denote the start and length of the bits that are contained in a data slice.
// So, for a grid aligned vertex with 30 bits of gas ids, and 3 bits of vertex orientation, we have
// 0 30 GAS_ID
// 30 31 VERTEX_ORIENTATION 0 2
// 0 1 VERTEX_ORIENTATION 2 1

// This file will be read by the vertex builders to decide how to construct each vertex bitpacking/unpacking in both the rust code and shader code