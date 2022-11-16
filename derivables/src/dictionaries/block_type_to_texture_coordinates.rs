use fundamentals::enums::block_type::BlockType;
pub static BLOCK_TYPE_TO_TEXTURE_INDICES: phf::Map<BlockType, [usize; 6]> = 
::phf::Map {
    key: 12913932095322966823,
    disps: &[
        (0, 0),
    ],
    entries: &[
        (BlockType::DIRT, [1, 1, 1, 1, 1, 1]),
        (BlockType::WOOD, [0, 0, 0, 0, 0, 0]),
    ],
};

