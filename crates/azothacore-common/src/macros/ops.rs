#[macro_export]
macro_rules! deref_boilerplate {
    ( $newtype:ty, $deref_type:ty, $field:tt ) => {
        impl ::std::ops::Deref for $newtype {
            type Target = $deref_type;

            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl ::std::ops::DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}

#[macro_export]
macro_rules! f32b {
    ( $v:expr ) => {{
        f32::to_bits($v) as i128
    }};
}

#[macro_export]
macro_rules! f64b {
    ( $v:expr ) => {{
        f64::to_bits($v) as i128
    }};
}

#[macro_export]
macro_rules! durationb {
    { $dur:expr } => {{
        durationb!(0, $dur.as_nanos())
    }};
    ( $s:expr, $ns:expr ) => {{
        let secs = ($s as u128) << 64;
        let nano_secs = ($ns as u128) & 0xFFFFFFFF;

        (secs | nano_secs) as i128
    }};
}

#[macro_export]
macro_rules! durationb_ms {
    ( $ms:expr ) => {{
        let ms = $ms as u128 * 1000000;
        let (s, ms) = if ms > 1_000_000_000 {
            (ms / 1_000_000_000, ms % 1_000_000_000)
        } else {
            (0, ms)
        };
        $crate::durationb!(s, ms)
    }};
}

#[macro_export]
macro_rules! durationb_s {
    ( $s:expr ) => {{
        $crate::durationb!($s, 0)
    }};
}

#[macro_export]
macro_rules! durationb_mins {
    ( $mins:expr ) => {{
        $crate::durationb_s!($mins * 60)
    }};
}

#[macro_export]
macro_rules! durationb_hours {
    ( $hours:expr ) => {{
        $crate::durationb_mins!($hours * 60)
    }};
}

#[macro_export]
macro_rules! durationb_days {
    ( $days:expr ) => {{
        $crate::durationb_hours!($days * 24)
    }};
}
