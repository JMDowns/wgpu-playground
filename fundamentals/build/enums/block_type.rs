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
         "{}\n{}\n{}",
         build_block_type_imports(),
         build_enum_text(&block_type_enum_values),
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
    format!("#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]\npub enum BlockType\n{{\n\tAIR = 0,\n\t{}\n}}\n",
        block_type_enum_values.join("\n\t"))
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
        (*self as u8).hash(state);
    }
}",
"impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}"].join("\n")
    )
}