use fundamentals::enums::block_type::BlockType;
use fundamentals::texture_coords::TextureCoordinates;
pub static BLOCK_TYPE_TO_TEXTURE_COORDINATES: phf::Map<BlockType, [TextureCoordinates; 6]> = 
::phf::Map {
    key: 12913932095322966823,
    disps: &[
        (0, 0),
    ],
    entries: &[
        (BlockType::DIRT, [TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 }]),
        (BlockType::WOOD, [TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 }]),
    ],
};

