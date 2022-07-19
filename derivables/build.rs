use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use string_to_type_dictionaries::string_to_block_type::STRING_TO_BLOCK_TYPE;
use formats::formats;
use phf_codegen;
use serde_json;

fn main() {
    let block_type_to_color_path = Path::new("src/dictionaries/").join("block_type_to_color.rs");
    let mut block_type_to_color_file = BufWriter::new(File::create(&block_type_to_color_path).unwrap());

    let blocks_json = std::fs::read_to_string("../data/blocks.json").unwrap();
    let vec_block_format: Vec<formats::block_format::BlockFormat> = serde_json::from_str(&blocks_json).unwrap();

    writeln!(
        &mut block_type_to_color_file,
         "{}\npub static BLOCK_TYPE_TO_COLOR: phf::Map<BlockType, wgpu::Color> = \n{};\n",
         get_imports(),
         get_map(&vec_block_format)
    ).unwrap();
}

fn get_imports() -> String {
    String::from([
        "use fundamentals::enums::block_type::BlockType;",
    ].join("\n"))
}

fn get_map(vec_block_format: &Vec<formats::block_format::BlockFormat>) -> String {
    let mut map = phf_codegen::Map::new();
    for block in vec_block_format {
        let mut rstr = format!("{}", block.color.r);
        let mut gstr = format!("{}", block.color.g);
        let mut bstr = format!("{}", block.color.b);
        if block.color.r == 0.0 {
            rstr = String::from("0.0");
        }
        if block.color.g == 0.0 {
            gstr = String::from("0.0");
        }
        if block.color.b == 0.0 {
            bstr = String::from("0.0");
        }
        map.entry(STRING_TO_BLOCK_TYPE.get(&block.block_type).unwrap(), &format!("wgpu::Color {{ r: {}, g: {}, b: {}, a: 0.0}}", rstr, gstr, bstr) );
    }
    map.build().to_string()
}