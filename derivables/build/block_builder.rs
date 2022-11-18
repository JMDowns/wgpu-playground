use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use fundamentals::enums::block_type::BlockTypeSize;

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
    [
        "use fundamentals::enums::block_type::{BlockType, BlockTypeSize};",
        "use crate::dictionaries::block_type_to_texture_coordinates::BLOCK_TYPE_TO_TEXTURE_INDICES;",
        "",
        "#[repr(C)]",
        "#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]",
        "pub struct Block {",
        "    pub block_type: BlockTypeSize",
        "}",
        "",
        "impl Block {",
        "    pub fn new(block_type: BlockType) -> Self {",
        "        Block { block_type: block_type as BlockTypeSize }",
        "    }",
        "",
        "    pub fn is_air(&self) -> bool {",
        "        self.block_type == BlockType::AIR as BlockTypeSize",
        "    }",
        "",
        "    pub fn get_texture_indices(&self) -> [usize; 6] {",
        "        Self::get_texture_indices_from_int(self.block_type)",
        "    }",
        "    pub fn get_texture_indices_from_type(block_type: &BlockType) -> [usize; 6] {",
        "        *BLOCK_TYPE_TO_TEXTURE_INDICES.get(block_type).unwrap()",
        "    }",
        "    pub fn get_texture_indices_from_int(btype_int: BlockTypeSize) -> [usize; 6] {",
        "        let block_type = BlockType::get_block_type_from_int(btype_int);",
        "        *BLOCK_TYPE_TO_TEXTURE_INDICES.get(&block_type).unwrap()",
        "    }",
        "}",
    ].join("\n")
}