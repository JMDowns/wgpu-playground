use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use ::formats::formats::block_format::BlockFormat;

pub fn build_enum(vec_block_format: &Vec<BlockFormat>){
    let block_type_enum_values = vec_block_format.iter().map(|ef| format!("{},", ef.block_type.replace(" ", "_").to_uppercase())).collect::<Vec<String>>();
    
    let block_types_file_path = Path::new("src/enums/block_type.rs");
    let mut block_types_file = BufWriter::new(File::create(&block_types_file_path).unwrap());
    writeln!(
        &mut block_types_file,
         "{}\n{}\n{}\n{}",
         build_block_type_imports(),
         build_enum_text(&block_type_enum_values),
         build_int_to_block_type_conversion(block_type_enum_values.len()),
         build_traits()
    ).unwrap();
}

fn build_block_type_imports() -> String {
    String::from([
        "use num_derive::FromPrimitive;",
        "use phf_shared;",
        "use std::fmt;",
        "use core::hash::{Hash, Hasher};"
    ].join("\n"))
}

fn build_enum_text(block_type_enum_values: &Vec<String>) -> String {
    [
    "#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]",
    "pub enum BlockType {",
    "\tAIR = 0,",
    format!("\t{}", block_type_enum_values.join("\n\t")).as_str(),
    "}"
    ].join("\n")
    
}

fn get_block_type_size(num_block_types: usize) -> u8 {
    if num_block_types < 256 {
        return 8;
    } else if (num_block_types as u32) < 65536 {
        return 16;
    } else if (num_block_types as u64) < 4294967296 {
        return 32;
    } else if (num_block_types as u128) < 18446744073709551616 {
        return 64;
    }
    return 128;
}

fn build_int_to_block_type_conversion(num_block_types: usize) -> String {
    let type_size = get_block_type_size(num_block_types);
    [
    format!("pub type BlockTypeSize = u{};", type_size).as_str(),
    "impl BlockType {",
    "    pub fn get_block_type_from_int(btype: BlockTypeSize) -> Self {",
    format!("        let btype_option = num::FromPrimitive::from_u{}(btype as BlockTypeSize);", get_block_type_size(num_block_types)).as_str(),
    "        btype_option.unwrap()",
    "    }",
    "   pub fn get_random_type() -> Self {",
    format!("       num::FromPrimitive::from_u{type_size}(fastrand::u{type_size}(1..{})).unwrap()", num_block_types+1).as_str(),
    "   }",
    "}"
    ].join("\n")
}

fn build_traits() -> String {
    String::from(
["impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, \"BlockType::{:?}\", self)
    }
}",
"impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as BlockTypeSize).hash(state);
    }
}",
"impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}"].join("\n")
    )
}