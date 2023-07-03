#[macro_export]
macro_rules! read_le {
    ( $input:expr, $dst_ty:ty ) => {{
        #[allow(unused_imports)]
        use ::std::io::Read;
        let mut buf = vec![0u8; ::std::mem::size_of::<$dst_ty>()];
        let _res = $input.read_exact(&mut buf[..])?;
        <$dst_ty>::from_le_bytes(buf.try_into().map_err(|e: Vec<u8>| {
            ::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                format!(
                    "CANNOT CONVERT TO LE BYTES, POSSIBLE SIZE MISMATCH: buf's length was {} want {}",
                    e.len(),
                    ::std::mem::size_of::<$dst_ty>(),
                ),
            )
        })?)
    }};
}

#[macro_export]
macro_rules! read_le_unwrap {
    ( $input:expr, $dst_ty:ty ) => {{
        #[allow(unused_imports)]
        use ::std::io::Read;
        let mut buf = vec![0u8; ::std::mem::size_of::<$dst_ty>()];
        let _res = $input.read_exact(&mut buf[..]).unwrap();
        <$dst_ty>::from_le_bytes(
            buf.try_into()
                .map_err(|e: Vec<u8>| {
                    ::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        format!(
                            "CANNOT CONVERT TO LE BYTES, POSSIBLE SIZE MISMATCH: buf's length was {} want {}",
                            e.len(),
                            ::std::mem::size_of::<$dst_ty>(),
                        ),
                    )
                })
                .unwrap(),
        )
    }};
}

#[macro_export]
macro_rules! read_buf {
    ( $input:expr, $size:literal ) => {{
        use ::std::io::Read;
        let mut buf = [0u8; $size];
        $input.read_exact(&mut buf[..])?;
        buf
    }};
}

#[macro_export]
macro_rules! cmp_or_return {
    ( $input:expr, $cmp:expr ) => {{
        let mut buf = vec![0u8; $cmp.len()];
        $input.read_exact(&mut buf[..])?;
        if &buf != $cmp {
            Err(::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                format!("cmpfail want {}, got {}", String::from_utf8_lossy($cmp), String::from_utf8_lossy(&buf)),
            ))
        } else {
            Ok(())
        }
    }};
}
