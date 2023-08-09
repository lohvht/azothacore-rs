#[macro_export]
macro_rules! az_error {
    ($msg:literal $(,)?) => {{
        $crate::format_err(::std::format_args!($msg))
    }};
    ($err:expr $(,)?) => {{
        $crate::az_error!("{}", $err)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::AzothaError::General(::std::format!($fmt, $($arg)*))
    }};
}
