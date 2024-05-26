/// Client expected level limitation, like as used in DBC item max levels for "until max player level"
/// use as default max player level, must be fit max level for used client
/// also see LEVEL_LIMIT_MAX and LEVEL_LIMIT_MAX_STRONG define
/// DEFAULT_MAX_LEVEL in TC/AC
pub const LEVEL_LIMIT_MAX_DEFAULT: u8 = 110;

/// client supported max level for player/pets/etc. Avoid overflow or client stability affected.
/// MAX_LEVEL in TC/AC
pub const LEVEL_LIMIT_MAX: u8 = 110;

/// Server side limitation. Base at used code requirements.
/// also see LEVEL_LIMIT_MAX
/// STRONG_MAX_LEVEL in TC/AC
pub const LEVEL_LIMIT_MAX_STRONG: u8 = 255;
