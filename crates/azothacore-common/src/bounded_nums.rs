use core::fmt;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    time::Duration,
};

use flagset::{flags, FlagSet};

/// FromI128 attempts to convert i128 to the types that implement this
/// and default to a sensible default if not possible.
///
/// For integral types implementing this, i128 is usually taken from the integer
/// values.
///
/// For other types implementing this, we tend to use the bit-representation
/// of i128 (ignoring its actual numerical value) instead.
pub trait FromI128: Copy + PartialOrd + serde::de::DeserializeOwned {
    fn from_i128(v: i128) -> Self;
}

impl FromI128 for u64 {
    fn from_i128(v: i128) -> Self {
        v.try_into().unwrap_or(if v < 0 { 0 } else { u64::MAX })
    }
}

impl FromI128 for u32 {
    fn from_i128(v: i128) -> Self {
        v.try_into().unwrap_or(if v < 0 { 0 } else { u32::MAX })
    }
}

impl FromI128 for i64 {
    fn from_i128(v: i128) -> Self {
        v.try_into().unwrap_or(if v > i64::MAX as i128 { i64::MAX } else { i64::MIN })
    }
}

impl FromI128 for i32 {
    fn from_i128(v: i128) -> Self {
        v.try_into().unwrap_or(if v > i32::MAX as i128 { i32::MAX } else { i32::MIN })
    }
}

impl FromI128 for Duration {
    fn from_i128(v: i128) -> Self {
        let v = v as u128;
        let seconds_part = (v >> 64) as _;
        let nano_part = (v << 96 >> 96) as u32 % 1_000_000_000;
        Duration::new(seconds_part, nano_part)
    }
}

impl FromI128 for f64 {
    fn from_i128(v: i128) -> Self {
        // Effectively handle the type by taking the number of bits it should and truncating the rest
        let v = (v << 64 >> 64) as u64;
        Self::from_bits(v)
    }
}

impl FromI128 for f32 {
    fn from_i128(v: i128) -> Self {
        // Effectively handle the type by taking the number of bits it should and truncating the rest
        let v = (v << 96 >> 96) as u64;
        Self::from_bits(v as u32)
    }
}

pub trait Assign: Sized {
    fn assign(&mut self, v: Self) {
        *self = v;
    }
}

impl Assign for u64 {}
impl Assign for u32 {}
impl Assign for i64 {}
impl Assign for i32 {}
impl Assign for Duration {}
impl Assign for f64 {}
impl Assign for f32 {}

flags! {
    enum BoundNumType: u8 {
        Upper              = 0b00001,
        Lower              = 0b00010,
        UpperDefault       = 0b00100,
        LowerDefault       = 0b01000,
    }
}

#[repr(transparent)]
#[derive(serde::Serialize, Debug, Copy, Clone, Eq, Ord, Hash)]
#[serde(transparent)]
pub struct BoundedNum<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>(
    N,
);

pub type UpperBoundedNum<N, const MAX: i128, const DEFAULT: i128> = BoundedNum<N, 0b01, 0, 0, MAX, DEFAULT, DEFAULT>;
pub type UpperBoundedNumMissingDefault<N, const MAX: i128, const MAX_DEFAULT: i128, const MISSING_DEFAULT: i128> =
    BoundedNum<N, 0b0101, 0, 0, MAX, MAX_DEFAULT, MISSING_DEFAULT>;
pub type LowerBoundedNum<N, const MIN: i128, const DEFAULT: i128> = BoundedNum<N, 0b10, MIN, DEFAULT, 0, 0, DEFAULT>;
pub type LowerBoundedNumMissingDefault<N, const MIN: i128, const MIN_DEFAULT: i128, const MISSING_DEFAULT: i128> =
    BoundedNum<N, 0b1010, MIN, MIN_DEFAULT, 0, 0, MISSING_DEFAULT>;

pub type RangedBoundedNumMissingDefault<N, const MIN: i128, const MIN_DEFAULT: i128, const MAX: i128, const MAX_DEFAULT: i128, const MISSING_DEFAULT: i128> =
    BoundedNum<N, 0b01111, MIN, MIN_DEFAULT, MAX, MAX_DEFAULT, MISSING_DEFAULT>;
pub type RangedBoundedNum<N, const MIN: i128, const MAX: i128, const MISSING_DEFAULT: i128> =
    BoundedNum<N, 0b011, MIN, MISSING_DEFAULT, MAX, MISSING_DEFAULT, MISSING_DEFAULT>;

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> Assign
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> Deref
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    type Target = N;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> DerefMut
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> PartialOrd<N>
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn partial_cmp(&self, other: &N) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<
        N: FromI128,
        const TYPE_MASK_FIRST: u8,
        const PRED1_FIRST: i128,
        const DEFAULT1_FIRST: i128,
        const PRED2_FIRST: i128,
        const DEFAULT2_FIRST: i128,
        const DEFAULT3_FIRST: i128,
        const TYPE_MASK_SECOND: u8,
        const PRED1_SECOND: i128,
        const DEFAULT1_SECOND: i128,
        const PRED2_SECOND: i128,
        const DEFAULT2_SECOND: i128,
        const DEFAULT3_SECOND: i128,
    > PartialOrd<BoundedNum<N, TYPE_MASK_SECOND, PRED1_SECOND, DEFAULT1_SECOND, PRED2_SECOND, DEFAULT2_SECOND, DEFAULT3_SECOND>>
    for BoundedNum<N, TYPE_MASK_FIRST, PRED1_FIRST, DEFAULT1_FIRST, PRED2_FIRST, DEFAULT2_FIRST, DEFAULT3_FIRST>
{
    fn partial_cmp(
        &self,
        other: &BoundedNum<N, TYPE_MASK_SECOND, PRED1_SECOND, DEFAULT1_SECOND, PRED2_SECOND, DEFAULT2_SECOND, DEFAULT3_SECOND>,
    ) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<
        N: FromI128,
        const TYPE_MASK_FIRST: u8,
        const PRED1_FIRST: i128,
        const DEFAULT1_FIRST: i128,
        const PRED2_FIRST: i128,
        const DEFAULT2_FIRST: i128,
        const DEFAULT3_FIRST: i128,
        const TYPE_MASK_SECOND: u8,
        const PRED1_SECOND: i128,
        const DEFAULT1_SECOND: i128,
        const PRED2_SECOND: i128,
        const DEFAULT2_SECOND: i128,
        const DEFAULT3_SECOND: i128,
    > PartialEq<BoundedNum<N, TYPE_MASK_SECOND, PRED1_SECOND, DEFAULT1_SECOND, PRED2_SECOND, DEFAULT2_SECOND, DEFAULT3_SECOND>>
    for BoundedNum<N, TYPE_MASK_FIRST, PRED1_FIRST, DEFAULT1_FIRST, PRED2_FIRST, DEFAULT2_FIRST, DEFAULT3_FIRST>
{
    fn eq(&self, other: &BoundedNum<N, TYPE_MASK_SECOND, PRED1_SECOND, DEFAULT1_SECOND, PRED2_SECOND, DEFAULT2_SECOND, DEFAULT3_SECOND>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<N: FromI128 + Display, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> Display
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! bounded_num_partial_ord_cmp_eq_to_n_generic {
    ( $n_typ:ty ) => {
        impl<const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
            PartialOrd<BoundedNum<$n_typ, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>> for $n_typ
        {
            fn partial_cmp(&self, other: &BoundedNum<$n_typ, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }
        impl<const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
            PartialEq<BoundedNum<$n_typ, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>> for $n_typ
        {
            fn eq(&self, other: &BoundedNum<$n_typ, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>) -> bool {
                self.eq(&other.0)
            }
        }
    };
}

bounded_num_partial_ord_cmp_eq_to_n_generic!(u64);
bounded_num_partial_ord_cmp_eq_to_n_generic!(u32);
bounded_num_partial_ord_cmp_eq_to_n_generic!(i64);
bounded_num_partial_ord_cmp_eq_to_n_generic!(i32);
bounded_num_partial_ord_cmp_eq_to_n_generic!(f64);
bounded_num_partial_ord_cmp_eq_to_n_generic!(f32);
bounded_num_partial_ord_cmp_eq_to_n_generic!(Duration);

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> PartialEq<N>
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn eq(&self, other: &N) -> bool {
        self.0.eq(other)
    }
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> Default
    for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn default() -> Self {
        Self(N::from_i128(DEFAULT3))
    }
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
    BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn bound_flags() -> FlagSet<BoundNumType> {
        FlagSet::<BoundNumType>::new(TYPE_MASK).unwrap_or_else(|e| panic!("unexpected error when using bounded num: {e}. type mask was: {TYPE_MASK}"))
    }
}

impl<N: FromI128 + fmt::Debug, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
    From<N> for BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn from(value: N) -> Self {
        let fs = Self::bound_flags();

        let pred1 = N::from_i128(PRED1);
        let v = if fs.contains(BoundNumType::Lower) && value < pred1 {
            let def = if fs.contains(BoundNumType::LowerDefault) { DEFAULT1 } else { DEFAULT3 };
            N::from_i128(def)
        } else {
            value
        };
        let pred2 = N::from_i128(PRED2);
        let v = if fs.contains(BoundNumType::Upper) && v > pred2 {
            let def: i128 = if fs.contains(BoundNumType::UpperDefault) { DEFAULT2 } else { DEFAULT3 };
            N::from_i128(def)
        } else {
            v
        };

        Self(v)
    }
}

// We don't do a blanket definition b/c we need to change one specific implmentation
// in Duration, and since blanket definitions are not "opt" out, the easier way to do this
// is via a macro (well there is implementing via newtypes)
macro_rules! bounded_num_generic_deserialise_impl {
    ( $typ:ty ) => {
        impl<'de, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
            serde::Deserialize<'de> for BoundedNum<$typ, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <$typ>::deserialize(deserializer).map(Self::from)
            }
        }
    };
}

bounded_num_generic_deserialise_impl!(u64);
bounded_num_generic_deserialise_impl!(u32);
bounded_num_generic_deserialise_impl!(i64);
bounded_num_generic_deserialise_impl!(i32);
bounded_num_generic_deserialise_impl!(f64);
bounded_num_generic_deserialise_impl!(f32);

impl<'de, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128> serde::Deserialize<'de>
    for BoundedNum<Duration, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let humantime_duration = s.parse::<humantime::Duration>().map_err(serde::de::Error::custom)?;

        let d: Duration = humantime_duration.into();
        Ok(Self::from(d))
    }
}

impl<N: FromI128, const TYPE_MASK: u8, const PRED1: i128, const DEFAULT1: i128, const PRED2: i128, const DEFAULT2: i128, const DEFAULT3: i128>
    BoundedNum<N, TYPE_MASK, PRED1, DEFAULT1, PRED2, DEFAULT2, DEFAULT3>
{
    /// NOTE: This is not an implementation of the std::borrow::ToOwned trait
    /// Its only to share the same function signature of it such that we can
    /// use if for ergonormic purposes
    pub fn to_owned(&self) -> N {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use figment::{
        providers::{Format, Toml},
        Figment,
    };

    use super::*;
    use crate::{durationb, durationb_ms, durationb_s, f32b, f64b};

    #[test]
    fn it_tests_f32_f64_impl_of_from_i128() {
        // f32 tests
        for expected in [
            1.0,
            0.0,
            -1.0,
            f32::MAX,
            f32::MIN,
            f32::MIN_POSITIVE,
            // f32::NAN, // by definition NAN cannot equal to NAN
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert_eq!(f32::from_i128(f32b!(expected)), expected);
        }

        // f64 tests
        for expected in [
            1.0,
            0.0,
            -1.0,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            // f64::NAN, // by definition NAN cannot equal to NAN
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert_eq!(f64::from_i128(f64b!(expected)), expected);
        }

        // we dont usually use i128 directly with floats, these are mainly for sanity
        // checks
        // Rust's integer types are 2s complements, effectively
        // if truncating a MIN i128 => 10000....00, which will be 0 after truncation
        assert_eq!(f32::from_i128(i128::MIN), 0.0);
        assert_eq!(f32::from_i128(i128::MIN), 0.0);
        // if truncating a MAX i128 => 0111....11, which will be NAN after truncation
        assert!(f64::from_i128(i128::MAX).is_nan());
        assert!(f64::from_i128(i128::MAX).is_nan());

        // Duration
    }

    #[test]
    fn it_tests_integral_types_impl_of_from_i128() {
        for (v, expected_u32, expected_u64, expected_i32, expected_i64) in [
            (i128::MAX, u32::MAX, u64::MAX, i32::MAX, i64::MAX),
            (i128::MIN, 0, 0, i32::MIN, i64::MIN),
            (0, 0, 0, 0, 0),
            (1, 1, 1, 1, 1),
            (-1, 0, 0, -1, -1),
        ] {
            assert_eq!(u32::from_i128(v), expected_u32);
            assert_eq!(u64::from_i128(v), expected_u64);
            assert_eq!(i32::from_i128(v), expected_i32);
            assert_eq!(i64::from_i128(v), expected_i64);
        }
    }

    #[test]
    fn it_tests_duration_impl_of_from_i128() {
        for (i, (v, expected_duration)) in [
            (0, Duration::ZERO),
            (1, Duration::new(0, 1)),
            (durationb!(12, 34), Duration::new(12, 34)),
            (durationb_ms!(10_034), Duration::from_millis(10_034)),
            (durationb_s!(7), Duration::from_secs(7)),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(Duration::from_i128(v), expected_duration, "Asserts dont match for test case {i}");
        }
    }

    #[test]
    fn it_tests_bounded_num_traits_f32() {
        const LOWER: f32 = -100.0;
        const UPPER: f32 = 100.0;
        const LOWER_DEFAULT: f32 = -95.0;
        const UPPER_DEFAULT: f32 = 95.0;
        const DEFAULT: f32 = 50.0;
        #[derive(serde::Deserialize, Debug, PartialEq, Default)]
        struct TestF32Struct {
            #[serde(default)]
            ub:   UpperBoundedNum<f32, { f32b!(UPPER) }, { f32b!(DEFAULT) }>,
            #[serde(default)]
            ubmd: UpperBoundedNumMissingDefault<f32, { f32b!(UPPER) }, { f32b!(UPPER_DEFAULT) }, { f32b!(DEFAULT) }>,
            #[serde(default)]
            lb:   LowerBoundedNum<f32, { f32b!(LOWER) }, { f32b!(DEFAULT) }>,
            #[serde(default)]
            lbmd: LowerBoundedNumMissingDefault<f32, { f32b!(LOWER) }, { f32b!(LOWER_DEFAULT) }, { f32b!(DEFAULT) }>,
            #[serde(default)]
            rb:   RangedBoundedNum<f32, { f32b!(LOWER) }, { f32b!(UPPER) }, { f32b!(DEFAULT) }>,
            #[serde(default)]
            rbmd:
                RangedBoundedNumMissingDefault<f32, { f32b!(LOWER) }, { f32b!(LOWER_DEFAULT) }, { f32b!(UPPER) }, { f32b!(UPPER_DEFAULT) }, { f32b!(DEFAULT) }>,
        }

        for (i, (test, expected)) in [
            (
                r#"
            ub = 1
            ubmd = 1
            lb = 1
            lbmd = 1
            rb = 1
            rbmd = 1
            "#,
                TestF32Struct {
                    ub:   1.0.into(),
                    ubmd: 1.0.into(),
                    lb:   1.0.into(),
                    lbmd: 1.0.into(),
                    rb:   1.0.into(),
                    rbmd: 1.0.into(),
                },
            ),
            (
                r#"
            ub = 101
            ubmd = 101
            lb = -101
            lbmd = -101
            rb = -101
            rbmd = -101
            "#,
                TestF32Struct {
                    ub:   DEFAULT.into(),
                    ubmd: UPPER_DEFAULT.into(),
                    lb:   DEFAULT.into(),
                    lbmd: LOWER_DEFAULT.into(),
                    rb:   DEFAULT.into(),
                    rbmd: LOWER_DEFAULT.into(),
                },
            ),
            (
                r#"
            ub = 101
            ubmd = 101
            lb = -101
            lbmd = -101
            rb = 101
            rbmd = 101
            "#,
                TestF32Struct {
                    ub:   DEFAULT.into(),
                    ubmd: UPPER_DEFAULT.into(),
                    lb:   DEFAULT.into(),
                    lbmd: LOWER_DEFAULT.into(),
                    rb:   DEFAULT.into(),
                    rbmd: UPPER_DEFAULT.into(),
                },
            ),
            (
                r#""#,
                TestF32Struct {
                    ub:   DEFAULT.into(),
                    ubmd: DEFAULT.into(),
                    lb:   DEFAULT.into(),
                    lbmd: DEFAULT.into(),
                    rb:   DEFAULT.into(),
                    rbmd: DEFAULT.into(),
                },
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let value: TestF32Struct = Figment::new().merge(Toml::string(test)).extract().unwrap();
            assert_eq!(value, expected, "TestF32Struct error at test case {i}");
        }
    }

    #[test]
    fn it_tests_bounded_num_traits_duration() {
        const LOWER: u64 = 10;
        const UPPER: u64 = 70;
        const LOWER_DEFAULT: u64 = 15;
        const UPPER_DEFAULT: u64 = 60;
        const DEFAULT: u64 = 30;
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestDurationStruct {
            #[serde(default)]
            ub:   UpperBoundedNum<Duration, { durationb_s!(UPPER) }, { durationb_s!(DEFAULT) }>,
            #[serde(default)]
            ubmd: UpperBoundedNumMissingDefault<Duration, { durationb_s!(UPPER) }, { durationb_s!(UPPER_DEFAULT) }, { durationb_s!(DEFAULT) }>,
            #[serde(default)]
            lb:   LowerBoundedNum<Duration, { durationb_s!(LOWER) }, { durationb_s!(DEFAULT) }>,
            #[serde(default)]
            lbmd: LowerBoundedNumMissingDefault<Duration, { durationb_s!(LOWER) }, { durationb_s!(LOWER_DEFAULT) }, { durationb_s!(DEFAULT) }>,
            #[serde(default)]
            rb:   RangedBoundedNum<Duration, { durationb_s!(LOWER) }, { durationb_s!(UPPER) }, { durationb_s!(DEFAULT) }>,
            #[serde(default)]
            rbmd: RangedBoundedNumMissingDefault<
                Duration,
                { durationb_s!(LOWER) },
                { durationb_s!(LOWER_DEFAULT) },
                { durationb_s!(UPPER) },
                { durationb_s!(UPPER_DEFAULT) },
                { durationb_s!(DEFAULT) },
            >,
        }

        for (i, (test, expected)) in [
            (
                r#"
            ub = "20s"
            ubmd = "20sec"
            lb = "20second"
            lbmd = "20seconds"
            rb = "20s"
            rbmd = "20secs"
            "#,
                TestDurationStruct {
                    ub:   Duration::from_secs(20).into(),
                    ubmd: Duration::from_secs(20).into(),
                    lb:   Duration::from_secs(20).into(),
                    lbmd: Duration::from_secs(20).into(),
                    rb:   Duration::from_secs(20).into(),
                    rbmd: Duration::from_secs(20).into(),
                },
            ),
            (
                r#"
            ub = "1min11s"
            ubmd = "71secs"
            lb = "9s"
            lbmd = "8seconds"
            rb = "7secs"
            rbmd = "5s"
            "#,
                TestDurationStruct {
                    ub:   Duration::from_secs(DEFAULT).into(),
                    ubmd: Duration::from_secs(UPPER_DEFAULT).into(),
                    lb:   Duration::from_secs(DEFAULT).into(),
                    lbmd: Duration::from_secs(LOWER_DEFAULT).into(),
                    rb:   Duration::from_secs(DEFAULT).into(),
                    rbmd: Duration::from_secs(LOWER_DEFAULT).into(),
                },
            ),
            (
                r#"
            ub = "1min11s"
            ubmd = "71secs"
            lb = "9s"
            lbmd = "8seconds"
            rb = "1m 11s"
            rbmd = "1min 13s"
            "#,
                TestDurationStruct {
                    ub:   Duration::from_secs(DEFAULT).into(),
                    ubmd: Duration::from_secs(UPPER_DEFAULT).into(),
                    lb:   Duration::from_secs(DEFAULT).into(),
                    lbmd: Duration::from_secs(LOWER_DEFAULT).into(),
                    rb:   Duration::from_secs(DEFAULT).into(),
                    rbmd: Duration::from_secs(UPPER_DEFAULT).into(),
                },
            ),
            (
                r#""#,
                TestDurationStruct {
                    ub:   Duration::from_secs(DEFAULT).into(),
                    ubmd: Duration::from_secs(DEFAULT).into(),
                    lb:   Duration::from_secs(DEFAULT).into(),
                    lbmd: Duration::from_secs(DEFAULT).into(),
                    rb:   Duration::from_secs(DEFAULT).into(),
                    rbmd: Duration::from_secs(DEFAULT).into(),
                },
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let value: TestDurationStruct = Figment::new().merge(Toml::string(test)).extract().unwrap();
            assert_eq!(value, expected, "TestDurationStruct error at test case {i}");
        }
    }
}
