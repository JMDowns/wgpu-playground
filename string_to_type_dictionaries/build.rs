use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
fn main() {
let lib_path = Path::new("src/lib.rs");
let string_to_block_types_path = Path::new("src/string_to_block_type.rs");
let string_to_texture_coords_path = Path::new("src/string_to_texture_coords.rs");
let mut lib_file = BufWriter::new(File::create(&lib_path).unwrap());
let mut string_to_block_types_file = BufWriter::new(File::create(&string_to_block_types_path).unwrap());
let mut string_to_texture_coords_file = BufWriter::new(File::create(&string_to_texture_coords_path).unwrap());
writeln!(
	&mut lib_file,
"{}{}",
"pub mod string_to_block_type;
",
"pub mod string_to_texture_coords;"
	).unwrap();
writeln!(
	&mut string_to_block_types_file,
	"use fundamentals::enums::block_type::BlockType;\npub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = \n{};\n",
	phf_codegen::Map::new()
		.entry("WOOD", "BlockType::WOOD,")
		.entry("DIRT", "BlockType::DIRT,")
		.entry("GRASS", "BlockType::GRASS,")
		.build()
	).unwrap();
writeln!(
	&mut string_to_texture_coords_file,
	"use fundamentals::texture_coords::TextureCoordinates;\npub static STRING_TO_TEXTURE_COORDINATES: phf::Map<&str, [TextureCoordinates; 6]> = \n{};\n",
	phf_codegen::Map::new()
		.entry("WOOD", "[TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0}]")
		.entry("DIRT", "[TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0}]")
		.entry("GRASS", "[TextureCoordinates {tx: 0.0, ty: 0.5},TextureCoordinates {tx: 0.0, ty: 0.5},TextureCoordinates {tx: 0.0, ty: 0.5},TextureCoordinates {tx: 0.0, ty: 0.5},TextureCoordinates {tx: 0.5, ty: 0.5},TextureCoordinates {tx: 0.5, ty: 0.0}]")
		.build()
	).unwrap();
}
