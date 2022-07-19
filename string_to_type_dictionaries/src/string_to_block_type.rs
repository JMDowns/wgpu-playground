use fundamentals::enums::block_type::BlockType;
pub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = 
::phf::Map {
    key: 10121458955350035957,
    disps: &[
        (3, 0),
    ],
    entries: &[
        ("DIRT", BlockType::DIRT),
        ("STONE", BlockType::STONE),
        ("GRASS", BlockType::GRASS),
        ("DEAD_GRASS", BlockType::DEAD_GRASS),
        ("WATER", BlockType::WATER),
    ],
};

