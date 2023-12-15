#[macro_export]
/// hex_str is ByteArrayToHexStr in TC/AC
macro_rules! hex_str {
    ( $b:expr ) => {{
        ::std::format!("{:X}", $crate::HexFmt($b))
    }};
}
