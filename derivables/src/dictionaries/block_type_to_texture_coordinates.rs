use fundamentals::enums::block_type::BlockType;
pub static BLOCK_TYPE_TO_TEXTURE_INDICES: phf::Map<BlockType, [usize; 6]> = 
::phf::Map {
    key: 7485420634051515786,
    disps: &[
        (3, 0),
    ],
    entries: &[
        (BlockType::DIRT, [2, 2, 2, 2, 2, 2]),
        (BlockType::TRANSPARENT, [0, 0, 0, 0, 0, 0]),
        (BlockType::GRASS, [3, 3, 3, 3, 4, 2]),
        (BlockType::WOOD, [1, 1, 1, 1, 1, 1]),
        (BlockType::WHITE, [5, 5, 5, 5, 5, 5]),
    ],
};

