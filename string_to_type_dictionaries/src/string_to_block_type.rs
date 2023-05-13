use fundamentals::enums::block_type::BlockType;
pub static STRING_TO_BLOCK_TYPE: phf::Map<&str, BlockType> = 
::phf::Map {
    key: 8694567506910003252,
    disps: &[
        (3, 0),
    ],
    entries: &[
        ("TRANSPARENT", BlockType::TRANSPARENT,),
        ("WOOD", BlockType::WOOD,),
        ("WHITE", BlockType::WHITE,),
        ("DIRT", BlockType::DIRT,),
        ("GRASS", BlockType::GRASS,),
    ],
};

