use fundamentals::enums::block_type::BlockType;
use fundamentals::texture_coords::TextureCoordinates;
pub static BLOCK_TYPE_TO_TEXTURE_COORDINATES: phf::Map<BlockType, [TextureCoordinates; 6]> = 
::phf::Map {
    key: 2126027241312876569,
    disps: &[
        (1, 0),
    ],
    entries: &[
        (BlockType::WOOD, [TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 },TextureCoordinates { tx: 0.0, ty: 0.0 }]),
        (BlockType::WHITE, [TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 },TextureCoordinates { tx: 0.5, ty: 0.0 }]),
        (BlockType::GRASS, [TextureCoordinates { tx: 0.5, ty: 0.33333334 },TextureCoordinates { tx: 0.5, ty: 0.33333334 },TextureCoordinates { tx: 0.5, ty: 0.33333334 },TextureCoordinates { tx: 0.5, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.6666667 },TextureCoordinates { tx: 0.0, ty: 0.33333334 }]),
        (BlockType::DIRT, [TextureCoordinates { tx: 0.0, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.33333334 },TextureCoordinates { tx: 0.0, ty: 0.33333334 }]),
    ],
};

