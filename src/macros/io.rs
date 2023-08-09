#[macro_export]
macro_rules! read_le {
    ( $input:expr, $dst_ty:ty ) => {{
        #[allow(unused_imports)]
        use ::std::io::Read;
        let mut buf = vec![0u8; ::std::mem::size_of::<$dst_ty>()];
        match $input.read_exact(&mut buf[..]) {
            Err(e) => Err(e),
            Ok(_) => {
                let bytes_res = buf.try_into().map_err(|e: Vec<u8>| {
                    ::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        format!(
                            "CANNOT CONVERT TO LE BYTES, POSSIBLE SIZE MISMATCH: buf's length was {} want {}",
                            e.len(),
                            ::std::mem::size_of::<$dst_ty>(),
                        ),
                    )
                });
                match bytes_res {
                    Err(e) => Err(e),
                    Ok(bytes) => Ok(<$dst_ty>::from_le_bytes(bytes)),
                }
            },
        }
    }};
}

#[macro_export]
macro_rules! read_le_unwrap {
    ( $input:expr, $dst_ty:ty ) => {{
        #[allow(unused_imports)]
        use $crate::read_le;
        read_le!($input, $dst_ty).unwrap()
    }};
}

#[macro_export]
macro_rules! read_buf {
    ( $input:expr, $size:literal ) => {{
        #[allow(unused_imports)]
        use ::std::io::Read;
        let mut buf = [0u8; $size];
        match $input.read_exact(&mut buf[..]) {
            Err(e) => Err(e),
            Ok(_) => Ok(buf),
        }
    }};
}

#[macro_export]
macro_rules! cmp_or_return {
    ( $input:expr, $cmp:expr ) => {{
        cmp_or_return!($input, $cmp, "cmpfail want {}, got {}")
    }};
    ( $input:expr, $cmp:expr, $format:expr ) => {{
        #[allow(unused_imports)]
        use ::std::io::Read;
        let mut buf = vec![0u8; $cmp.len()];
        match $input.read_exact(&mut buf[..]) {
            Err(e) => Err(e),
            Ok(_) => {
                if &buf != $cmp {
                    Err(::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        format!($format, String::from_utf8_lossy($cmp), String::from_utf8_lossy(&buf)),
                    ))
                } else {
                    Ok(buf)
                }
            },
        }
    }};
}

#[macro_export]
macro_rules! sanity_check_read_all_bytes_from_reader {
    ( $rdr:expr ) => {{
        let mut buf_remain = vec![];
        match $rdr.read_to_end(&mut buf_remain) {
            Err(e) => Err(e),
            Ok(_) => {
                if !buf_remain.is_empty() {
                    Err(::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        format!(
                            "SANITY_CHECK: somehow file isn't fully consumed. please check again! {} bytes left",
                            buf_remain.len(),
                        ),
                    ))
                } else {
                    Ok(())
                }
            },
        }
    }};
}
