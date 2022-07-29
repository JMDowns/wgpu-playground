use formats::formats::block_format::BlockFormat;
mod block_type;

pub fn build_enums(block_type_enum_values: &Vec<BlockFormat>) {
    block_type::build_enum(block_type_enum_values);
}