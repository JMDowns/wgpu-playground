use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
fn main() {
let lib_path = Path::new("src/lib.rs");
let string_to_block_types_path = Path::new("src/string_to_block_type.rs");
let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());
let mut string_to_block_types_file = BufWriter::new(File::create(&string_to_block_types_path).unwrap());
writeln!(
	&mut lib_file,
"pub mod string_to_block_type;"
	).unwrap();
writeln!(
	&mut string_to_block_types_file,
	"use fundamentals::enums::block_type::BlockType;\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \n{};\n",
	phf_codegen::Map::new()
		.entry("STONE", "BlockType::	STONE,")
		.entry("DIRT", "BlockType::	DIRT,")
		.entry("GRASS", "BlockType::	GRASS,")
		.entry("WATER", "BlockType::	WATER,")
		.entry("DEAD GRASS", "BlockType::	DEAD_GRASS,")
		.build()
	).unwrap();
}
