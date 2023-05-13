use fundamentals::enums::block_type::BlockType;
pub static BLOCK_TYPE_TO_TEXTURE_INDICES: phf::Map<BlockType, [usize; 6]> = 
::phf::Map {
    key: 2126027241312876569,
    disps: &[
        (1, 0),
    ],
    entries: &[
        (BlockType::WOOD, [0, 0, 0, 0, 0, 0]),
        (BlockType::DIRT, [1, 1, 1, 1, 1, 1]),
        (BlockType::WHITE, [4, 4, 4, 4, 4, 4]),
        (BlockType::GRASS, [2, 2, 2, 2, 3, 1]),
    ],
};

