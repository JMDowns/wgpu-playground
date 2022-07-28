use fundamentals::enums::block_type::BlockType;
use fundamentals::texture_coords::TextureCoordinates;
pub static BLOCK_TYPE_TO_TEXTURE_COORDINATES: phf::Map<BlockType, [TextureCoordinates; 6]> = 
::phf::Map {
    key: 15467950696543387533,
    disps: &[
        (1, 0),
    ],
    entries: &[
        (BlockType::GRASS, [TextureCoordinates { tx: 0.0, ty: 0.5 },TextureCoordinates { tx: 0.0, ty: 0.5 },TextureCoordinates { tx: 0.0, ty: 0.5 },TextureCoordinates { tx: 0.0, ty: 0.5 },TextureCoordinates { tx: 0.0, ty: 0.75 },TextureCoordinates { tx: 0.0, ty: 0.25 }]),
        (BlockType::WOOD, [TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 }]),
        (BlockType::DIRT, [TextureCoordinates { tx: 0.0, ty: 0.25 },TextureCoordinates { tx: 0.0, ty: 0.25 },TextureCoordinates { tx: 0.0, ty: 0.25 },TextureCoordinates { tx: 0.0, ty: 0.25 },TextureCoordinates { tx: 0.0, ty: 0.25 },TextureCoordinates { tx: 0.0, ty: 0.25 }]),
    ],
};

