use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};

const fn get_block_type_size() -> u8 {
    if fundamentals::consts::NUM_BLOCK_TYPES < 256 {
        return 8;
    } else if (fundamentals::consts::NUM_BLOCK_TYPES as u32) < 65536 {
        return 16;
    } else if (fundamentals::consts::NUM_BLOCK_TYPES as u64) < 4294967296 {
        return 32;
    } else if (fundamentals::consts::NUM_BLOCK_TYPES as u128) < 18446744073709551616 {
        return 64;
    }
    return 128;
}

pub fn build_block_file() {
    let block_path = Path::new("src/block.rs");
    let mut block_file = BufWriter::new(File::create(&block_path).unwrap());

    writeln!(
        &mut block_file,
         "{}",
         build_block_string()
    ).unwrap();
}

fn build_block_string() -> String {
    let num_str = get_block_type_size().to_string();
    [
        "use fundamentals::enums::block_type::BlockType;",
        "use crate::dictionaries::block_type_to_texture_coordinates::BLOCK_TYPE_TO_TEXTURE_INDICES;",
        "",
        "#[repr(C)]",
        "#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]",
        "pub struct Block {",
        format!("    pub block_type: u{}", num_str).as_str(),
        "}",
        "",
        "impl Block {",
        "    pub fn new(bt: BlockType) -> Self {",
        format!("        Block {{ block_type: bt as u{} }}", num_str).as_str(),
        "    }",
        "",
        "    pub fn is_air(&self) -> bool {",
        "        self.block_type == 0",
        "    }",
        "",
        "    pub fn get_texture_indices(&self) -> &[usize; 6] {",
        format!("        let btype_option = num::FromPrimitive::from_u{}(self.block_type);", num_str).as_str(),
        "        let btype = match btype_option {",
        "           Some(bt) => bt,",
        "           None => BlockType::AIR",
        "        };",
        "        BLOCK_TYPE_TO_TEXTURE_INDICES.get(&btype).unwrap()",
        "    }",
        "}",
    ].join("\n")
}