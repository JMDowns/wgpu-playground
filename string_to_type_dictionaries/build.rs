use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
fn main() {
let lib_path = Path::new("src/lib.rs");
let string_to_block_types_path = Path::new("src/string_to_block_type.rs");
let string_to_texture_indices_path = Path::new("src/string_to_texture_indices.rs");
let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());
let mut string_to_block_types_file = BufWriter::new(File::create(&string_to_block_types_path).unwrap());
let mut string_to_texture_indices_file = BufWriter::new(File::create(&string_to_texture_indices_path).unwrap());
writeln!(
	&mut lib_file,
"{}{}",
"pub mod string_to_block_type;
",
"pub mod string_to_texture_indices;"
	).unwrap();
writeln!(
	&mut string_to_block_types_file,
	"use fundamentals::enums::block_type::BlockType;\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \n{};\n",
	phf_codegen::Map::new()
		.entry("TRANSPARENT", "BlockType::TRANSPARENT,")
		.entry("WOOD", "BlockType::WOOD,")
		.entry("DIRT", "BlockType::DIRT,")
		.entry("GRASS", "BlockType::GRASS,")
		.entry("WHITE", "BlockType::WHITE,")
		.build()
	).unwrap();
writeln!(
	&mut string_to_texture_indices_file,
	"pub static STRING_TO_TEXTURE_INDICES: phf::Map<&str, [usize; 6]> = \n{};\n",
	phf_codegen::Map::new()
		.entry("TRANSPARENT", "[0, 0, 0, 0, 0, 0]")
		.entry("WOOD", "[1, 1, 1, 1, 1, 1]")
		.entry("DIRT", "[2, 2, 2, 2, 2, 2]")
		.entry("GRASS", "[3, 3, 3, 3, 4, 2]")
		.entry("WHITE", "[5, 5, 5, 5, 5, 5]")
		.build()
	).unwrap();
}
