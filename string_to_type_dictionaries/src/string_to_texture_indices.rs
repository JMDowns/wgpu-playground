use fundamentals::enums::block_type::BlockType;
use fundamentals::texture_coords::TextureCoordinates;
pub static STRING_TO_TEXTURE_COORDINATES: phf::Map<&str, [TextureCoordinates; 6]> = 
::phf::Map {
    key: 15467950696543387533,
    disps: &[
        (1, 0),
    ],
    entries: &[
        ("DIRT", [TextureCoordinates {tx: 0.25, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0}]),
        ("WOOD", [TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0}]),
        ("GRASS", [TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.75, ty: 0.0},TextureCoordinates {tx: 0.25, ty: 0.0}]),
    ],
};

