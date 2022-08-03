use fundamentals::texture_coords::TextureCoordinates;
pub static STRING_TO_TEXTURE_COORDINATES: phf::Map<&str, [TextureCoordinates; 6]> = 
::phf::Map {
    key: 2980949210194914378,
    disps: &[
        (1, 0),
    ],
    entries: &[
        ("GRASS", [TextureCoordinates {tx: 0.5, ty: 0.33333334},TextureCoordinates {tx: 0.5, ty: 0.33333334},TextureCoordinates {tx: 0.5, ty: 0.33333334},TextureCoordinates {tx: 0.5, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.6666667},TextureCoordinates {tx: 0.0, ty: 0.33333334}]),
        ("DIRT", [TextureCoordinates {tx: 0.0, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.33333334},TextureCoordinates {tx: 0.0, ty: 0.33333334}]),
        ("WOOD", [TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0},TextureCoordinates {tx: 0.0, ty: 0.0}]),
        ("WHITE", [TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0},TextureCoordinates {tx: 0.5, ty: 0.0}]),
    ],
};

