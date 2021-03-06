//! Dual Numbers
//!
//! This is a dual number implementation scavenged from other dual number libraries and articles around the web, including:
//!
//! * [https://github.com/FreeFull/dual_numbers](https://github.com/FreeFull/dual_numbers)
//! * [https://github.com/ibab/rust-ad](https://github.com/ibab/rust-ad)
//! * [https://github.com/tesch1/cxxduals](https://github.com/tesch1/cxxduals)
//!
//! The difference being is that I have checked each method against Wolfram Alpha for correctness and will
//! keep this implementation up to date and working with the latest stable Rust and `num-traits` crate.
//!
//! ## Usage
//!
//! ```rust
//! extern crate dual_num;
//!
//! use dual_num::{DualNumber, Float, differentiate};
//!
//! fn main() {
//!     // find partial derivative at x=4.0
//!     println!("{:.5}", differentiate(4.0f64, |x| {
//!         x.sqrt() + DualNumber::from_real(1.0)
//!     })); // 0.25000
//! }
//! ```

// Note that the somewhat excessive #[inline] annotations are not harmful here,
// and can improve cross-crate inlining.
//
// Also, for clarity I've avoiding using .0 and .1 outside of the struct impl.
// They're even made private to encourage using .real() and .dual() instead.

extern crate num_traits;

use std::ops::{Add, Sub, Mul, Div, Rem, Neg};
use std::cmp::Ordering;
use std::num::FpCategory;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub use num_traits::{One, Zero, Float, FloatConst, Num};

use num_traits::{Signed, Unsigned, NumCast, ToPrimitive, FromPrimitive};

/// Dual Number structure
///
/// Although `DualNumber` does implement `PartialEq` and `PartialOrd`,
/// it only compares the real part.
///
/// Additionally, `min` and `max` only compare the real parts, and keep the dual parts.
///
/// Lastly, the `Rem` remainder operator is not correctly or fully defined for `DualNumber`, and will panic.
#[derive(Debug, Clone, Copy)]
pub struct DualNumber<T>(T, T);

/// Convenience type
pub type DualNumberF32 = DualNumber<f32>;

/// Convenience type
pub type DualNumberF64 = DualNumber<f64>;

/// Evaluates the function using dual numbers to get the partial derivative at the input point
pub fn differentiate<T: One + Copy, F>(x: T, f: F) -> T where F: Fn(DualNumber<T>) -> DualNumber<T> {
    f(DualNumber::new(x, T::one())).dual()
}

impl<T> DualNumber<T> {
    /// Create a new dual number from its real and dual parts.
    #[inline]
    pub fn new(real: T, dual: T) -> DualNumber<T> {
        DualNumber(real, dual)
    }

    /// Create a new dual number from a real number.
    ///
    /// The dual part is set to zero.
    #[inline]
    pub fn from_real(real: T) -> DualNumber<T> where T: Zero {
        DualNumber::new(real, T::zero())
    }

    /// Returns both real and dual parts as a tuple
    #[inline]
    pub fn into_tuple(self) -> (T, T) {
        (self.0, self.1)
    }

    /// Returns a reference to the real part
    #[inline]
    pub fn real_ref(&self) -> &T { &self.0 }

    /// Returns a reference to the dual part
    #[inline]
    pub fn dual_ref(&self) -> &T { &self.1 }

    /// Returns a mutable reference to the real part
    #[inline]
    pub fn real_mut(&mut self) -> &mut T { &mut self.0 }

    /// Returns a mutable reference to the dual part
    #[inline]
    pub fn dual_mut(&mut self) -> &mut T { &mut self.1 }

    /// Convenience method to take a closure (or any function) that can operate on the dual number in place
    #[inline(always)]
    pub fn map<F>(self, mapper: F) -> Self where F: Fn(DualNumber<T>) -> DualNumber<T> {
        mapper(self)
    }

    /// Convenience method to take a closure (or any function) that can operate on the dual number parts in place
    #[inline(always)]
    pub fn map_parts<F>(self, mapper: F) -> Self where F: Fn(T, T) -> DualNumber<T> {
        mapper(self.0, self.1)
    }
}

impl<T: Zero> From<T> for DualNumber<T> {
    fn from(real: T) -> DualNumber<T> {
        DualNumber::from_real(real)
    }
}

impl<T: Copy> DualNumber<T> {
    /// Returns the real part
    #[inline(always)]
    pub fn real(&self) -> T { self.0 }

    /// Returns the dual part
    #[inline(always)]
    pub fn dual(&self) -> T { self.1 }
}

impl<T: Float> DualNumber<T> {
    /// Returns the conjugate of the dual number.
    pub fn conjugate(self) -> Self {
        DualNumber(self.real(), self.dual().neg())
    }
}

impl<T: Display> Display for DualNumber<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let precision = f.precision().unwrap_or(2);

        write!(f, "{:.p$} + \u{03B5}{:.p$}", self.0, self.1, p = precision)
    }
}

impl<T: PartialEq> PartialEq<Self> for DualNumber<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T: PartialOrd> PartialOrd<Self> for DualNumber<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.real_ref(), rhs.real_ref())
    }

    fn lt(&self, rhs: &Self) -> bool { self.0 < rhs.0 }
    fn le(&self, rhs: &Self) -> bool { self.0 <= rhs.0 }
    fn gt(&self, rhs: &Self) -> bool { self.0 > rhs.0 }
    fn ge(&self, rhs: &Self) -> bool { self.0 >= rhs.0 }
}

impl<T: PartialEq> PartialEq<T> for DualNumber<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.0 == *rhs
    }
}

impl<T: PartialOrd> PartialOrd<T> for DualNumber<T> {
    fn partial_cmp(&self, rhs: &T) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.real_ref(), rhs)
    }

    fn lt(&self, rhs: &T) -> bool { self.0 < *rhs }
    fn le(&self, rhs: &T) -> bool { self.0 <= *rhs }
    fn gt(&self, rhs: &T) -> bool { self.0 > *rhs }
    fn ge(&self, rhs: &T) -> bool { self.0 >= *rhs }
}

macro_rules! impl_to_primitive {
    ($($name:ident, $ty:ty),*) => {
        impl<T: ToPrimitive> ToPrimitive for DualNumber<T> {
            $(
                fn $name(&self) -> Option<$ty> {
                    (self.0).$name()
                }
            )*
        }
    }
}

macro_rules! impl_from_primitive {
    ($($name:ident, $ty:ty),*) => {
        impl<T: FromPrimitive> FromPrimitive for DualNumber<T> where T: Zero {
            $(
                fn $name(n: $ty) -> Option<DualNumber<T>> {
                    T::$name(n).map(DualNumber::from_real)
                }
            )*
        }
    }
}

macro_rules! impl_primitive_cast {
    ($($to:ident, $from:ident - $ty:ty),*) => {
        impl_to_primitive!($($to, $ty),*);
        impl_from_primitive!($($from, $ty),*);
    }
}

impl_primitive_cast!(
    to_isize,   from_isize  - isize,
    to_i8,      from_i8     - i8,
    to_i16,     from_i16    - i16,
    to_i32,     from_i32    - i32,
    to_i64,     from_i64    - i64,
    to_usize,   from_usize  - usize,
    to_u8,      from_u8     - u8,
    to_u16,     from_u16    - u16,
    to_u32,     from_u32    - u32,
    to_u64,     from_u64    - u64,
    to_f32,     from_f32    - f32,
    to_f64,     from_f64    - f64
);

impl<T: Num + Copy> Add<T> for DualNumber<T> {
    type Output = DualNumber<T>;

    #[inline]
    fn add(self, rhs: T) -> DualNumber<T> {
        DualNumber::new(self.real() + rhs,
                        self.dual())
    }
}

impl<T: Num + Copy> Sub<T> for DualNumber<T> {
    type Output = DualNumber<T>;

    #[inline]
    fn sub(self, rhs: T) -> DualNumber<T> {
        DualNumber::new(self.real() - rhs,
                        self.dual())
    }
}

impl<T: Num + Copy> Mul<T> for DualNumber<T> {
    type Output = DualNumber<T>;

    fn mul(self, rhs: T) -> DualNumber<T> {
        self * DualNumber::from_real(rhs)
    }
}

impl<T: Num + Copy> Div<T> for DualNumber<T> {
    type Output = DualNumber<T>;

    #[inline]
    fn div(self, rhs: T) -> DualNumber<T> {
        self / DualNumber::from_real(rhs)
    }
}

impl<T: Signed + Copy> Neg for DualNumber<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        DualNumber::new(self.real().neg(),
                        self.dual().neg())
    }
}

impl<T: Num + Copy> Add<Self> for DualNumber<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        DualNumber::new(self.real() + rhs.real(),
                        self.dual() + rhs.dual())
    }
}

impl<T: Num + Copy> Sub<Self> for DualNumber<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        DualNumber::new(self.real() - rhs.real(),
                        self.dual() - rhs.dual())
    }
}

impl<T: Num + Copy> Mul<Self> for DualNumber<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        DualNumber::new(
            self.real() * rhs.real(),
            self.real() * rhs.dual() + self.dual() * rhs.real()
        )
    }
}

impl<T: Num + Copy> Div<Self> for DualNumber<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        DualNumber::new(
            self.real() / rhs.real(),
            (self.dual() * rhs.real() - self.real() * rhs.dual()) / (rhs.real() * rhs.real())
        )
    }
}

impl<T: Num + Copy> Rem<Self> for DualNumber<T> {
    type Output = Self;

    /// **UNIMPLEMENTED!!!**
    ///
    /// As far as I know, remainder is not a valid operation on dual numbers,
    /// but is required for the `Float` trait to be implemented.
    fn rem(self, _: Self) -> Self {
        unimplemented!()
    }
}

impl<T> Signed for DualNumber<T> where T: Signed + Copy + PartialOrd {
    #[inline]
    fn abs(&self) -> Self {
        DualNumber::new(self.real().abs(), self.dual() * self.real().signum())
    }

    fn abs_sub(&self, rhs: &Self) -> Self {
        if self.real() > rhs.real() {
            DualNumber::new(self.real() - rhs.real(), self.sub(*rhs).dual())
        } else {
            Self::zero()
        }
    }

    #[inline]
    fn signum(&self) -> Self {
        DualNumber::from_real(self.real().signum())
    }

    #[inline(always)]
    fn is_positive(&self) -> bool {
        self.real().is_positive()
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        self.real().is_negative()
    }
}

impl<T: Unsigned> Unsigned for DualNumber<T> where Self: Num {}

impl<T: Num + Zero + Copy> Zero for DualNumber<T> {
    #[inline]
    fn zero() -> DualNumber<T> {
        DualNumber::new(T::zero(), T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.real().is_zero()
    }
}

impl<T: Num + One + Copy> One for DualNumber<T> {
    #[inline]
    fn one() -> DualNumber<T> {
        DualNumber::new(T::one(), T::zero())
    }
}

impl<T: Num + Copy> Num for DualNumber<T> {
    type FromStrRadixErr = <T as Num>::FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<DualNumber<T>, Self::FromStrRadixErr> {
        <T as Num>::from_str_radix(str, radix).map(DualNumber::from_real)
    }
}

impl<T: Float> NumCast for DualNumber<T> {
    #[inline]
    fn from<N: ToPrimitive>(n: N) -> Option<DualNumber<T>> {
        <T as NumCast>::from(n).map(DualNumber::from_real)
    }
}

macro_rules! impl_float_const {
    ($($c:ident),*) => {
        $(
            #[inline(always)]
            fn $c() -> DualNumber<T> { DualNumber::from_real(T::$c()) }
        )*
    }
}

impl<T: FloatConst + Zero> FloatConst for DualNumber<T> {
    impl_float_const!(
        E,
        FRAC_1_PI,
        FRAC_1_SQRT_2,
        FRAC_2_PI,
        FRAC_2_SQRT_PI,
        FRAC_PI_2,
        FRAC_PI_3,
        FRAC_PI_4,
        FRAC_PI_6,
        FRAC_PI_8,
        LN_10,
        LN_2,
        LOG10_E,
        LOG2_E,
        PI,
        SQRT_2
    );
}

macro_rules! impl_real_constant {
    ($($prop:ident),*) => {
        $(
            #[inline]
            fn $prop() -> Self { DualNumber::from_real(<T as Float>::$prop()) }
        )*
    }
}

macro_rules! impl_single_boolean_op {
    ($op:ident REAL) => {
        #[inline]
        fn $op(self) -> bool {self.real().$op()}
    };
    ($op:ident OR) =>   { fn $op(self) -> bool {self.real().$op() || self.dual().$op()} };
    ($op:ident AND) =>  { fn $op(self) -> bool {self.real().$op() && self.dual().$op()} };
}

macro_rules! impl_boolean_op {
    ($($op:ident $t:tt),*) => {
        $(impl_single_boolean_op!($op $t);)*
    };
}

macro_rules! impl_real_op {
    ($($op:ident),*) => {
        $(
            #[inline]
            fn $op(self) -> Self { DualNumber::new(self.real().$op(), T::zero()) }
        )*
    }
}

impl<T> Float for DualNumber<T> where T: Float + Signed + FloatConst {
    impl_real_constant!(
        nan,
        infinity,
        neg_infinity,
        neg_zero,
        min_positive_value,
        epsilon,
        min_value,
        max_value
    );

    impl_boolean_op!(
        is_nan              OR,
        is_infinite         OR,
        is_finite           AND,
        is_normal           AND,
        is_sign_positive    REAL,
        is_sign_negative    REAL
    );

    fn classify(self) -> FpCategory {
        self.real().classify()
    }

    impl_real_op!(
        floor,
        ceil,
        round,
        trunc
    );

    fn fract(self) -> Self {
        DualNumber::new(self.real().fract(), self.dual())
    }

    #[inline]
    fn signum(self) -> Self {
        DualNumber::from_real(self.real().signum())
    }

    #[inline]
    fn abs(self) -> Self {
        DualNumber::new(self.real().abs(), self.dual() * self.real().signum())
    }

    fn max(self, other: Self) -> Self {
        if self.real() > other.real() { self } else { other }
    }

    fn min(self, other: Self) -> Self {
        if self.real() < other.real() { other } else { self }
    }

    fn abs_sub(self, rhs: Self) -> Self {
        if self.real() > rhs.real() {
            DualNumber::new(self.real() - rhs.real(), (self - rhs).dual())
        } else {
            Self::zero()
        }
    }

    fn mul_add(self, a: Self, b: Self) -> Self {
        DualNumber::new(self.real().mul_add(a.real(), b.real()),
                        self.dual() * a.real() + self.real() * a.dual() + b.dual())
    }

    #[inline]
    fn recip(self) -> Self {
        Self::one() / self
    }

    fn powi(self, n: i32) -> Self {
        let nf = <T as NumCast>::from(n).expect("Invalid value");

        DualNumber::new(self.real().powi(n),
                        nf * self.real().powi(n - 1) * self.dual())
    }

    fn powf(self, n: Self) -> Self {
        let real = self.real().powf(n.real());

        DualNumber::new(real,
                        n.real() * self.real().powf(n.real() - T::one()) * self.dual() +
                            real * self.real().ln() * n.dual())
    }

    fn exp(self) -> Self {
        let real = self.real().exp();

        DualNumber::new(real, self.dual() * real)
    }

    fn exp2(self) -> Self {
        let real = self.real().exp2();

        DualNumber::new(real, self.dual() * T::LN_2() * real)
    }

    fn ln(self) -> Self {
        DualNumber::new(self.real().ln(), self.dual() / self.real())
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        DualNumber::new(self.real().log10(), self.dual() / (self.real() * T::LN_2()))
    }

    #[inline]
    fn log10(self) -> Self {
        DualNumber::new(self.real().log10(), self.dual() / (self.real() * T::LN_10()))
    }

    #[inline]
    fn sqrt(self) -> Self {
        let real = self.real().sqrt();

        DualNumber::new(real, self.dual() / (T::from(2).unwrap() * real))
    }

    #[inline]
    fn cbrt(self) -> Self {
        let real = self.real().cbrt();

        DualNumber::new(real, self.dual() / (T::from(3).unwrap() * real))
    }

    fn hypot(self, other: Self) -> Self {
        let real = self.real().hypot(other.real());

        DualNumber::new(real, (self.real() * other.dual() + other.real() * self.dual()) / real)
    }

    fn sin(self) -> Self { DualNumber::new(self.real().sin(), self.dual() * self.real().cos()) }
    fn cos(self) -> Self { DualNumber::new(self.real().cos(), self.dual().neg() * self.real().sin()) }

    fn tan(self) -> Self {
        let t = self.real().tan();

        DualNumber::new(t, self.dual() * (t * t + T::one()))
    }

    fn asin(self) -> Self { DualNumber::new(self.real().asin(), self.dual() / (T::one() - self.real().powi(2)).sqrt()) }
    fn acos(self) -> Self { DualNumber::new(self.real().acos(), self.dual().neg() / (T::one() - self.real().powi(2)).sqrt()) }
    fn atan(self) -> Self { DualNumber::new(self.real().atan(), self.dual() / (self.real().powi(2) + T::one()).sqrt()) }

    fn atan2(self, other: Self) -> Self {
        DualNumber::new(
            self.real().atan2(other.real()),
            (other.real() * self.dual() - self.real() * other.dual()) /
                (self.real().powi(2) + other.real().powi(2))
        )
    }

    fn sin_cos(self) -> (Self, Self) {
        let (s, c) = self.real().sin_cos();

        let sn = DualNumber::new(s, self.dual() * c);
        let cn = DualNumber::new(c, self.dual().neg() * s);

        (sn, cn)
    }

    fn exp_m1(self) -> Self { DualNumber::new(self.real().exp_m1(), self.dual() * self.real().exp()) }

    fn ln_1p(self) -> Self { DualNumber::new(self.real().ln_1p(), self.dual() / (self.real() + T::one())) }

    fn sinh(self) -> Self { DualNumber::new(self.real().sinh(), self.dual() * self.real().cosh()) }
    fn cosh(self) -> Self { DualNumber::new(self.real().cosh(), self.dual() * self.real().sinh()) }

    fn tanh(self) -> Self {
        let real = self.real().tanh();

        DualNumber::new(real, self.dual() * (T::one() - real.powi(2)))
    }

    fn asinh(self) -> Self { DualNumber::new(self.real().asinh(), self.dual() / (self.real().powi(2) + T::one()).sqrt()) }

    fn acosh(self) -> Self {
        DualNumber::new(self.real().acosh(),
                        self.dual() /
                            ((self.real() + T::one()).sqrt() *
                                (self.real() - T::one()).sqrt()))
    }

    fn atanh(self) -> Self { DualNumber::new(self.real().atanh(), self.dual() / (T::one() - self.real().powi(2))) }

    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) { self.real().integer_decode() }

    #[inline]
    fn to_degrees(self) -> Self { DualNumber::from_real(self.real().to_degrees()) }

    #[inline]
    fn to_radians(self) -> Self { DualNumber::from_real(self.real().to_radians()) }
}