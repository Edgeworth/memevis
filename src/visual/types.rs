use crate::any::{Any, Basic};
use derive_more::{Display, From, Into};
use glium::glutin::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use num::{Num, NumCast, ToPrimitive};
use num_traits::Zero;
use paste::paste;
use rgb::RGBA;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::Neg;

pub trait Number = Clone + Copy + Num + NumCast + Default + PartialOrd + PartialEq;
pub type Col = RGBA<f32>;

macro_rules! binop_vec2_vec2 {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number, U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2)
            }
        }

        paste!{
        impl<T: Number, U: Basic>
                std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
            }
        }
        }

        binop_vec2_vec2!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type, $rhs_f1, $rhs_f2;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_vec4 {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident, $rhs_f3:ident, $rhs_f4:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident, $rhs_f3:ident, $rhs_f4:ident;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number, U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2,
                    self.$lhs_f3 $op o.$rhs_f3, self.$lhs_f4 $op o.$rhs_f4)
            }
        }

        paste!{
        impl<T: Number, U: Basic> std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
                self.$lhs_f3 = self.$lhs_f3 $op o.$rhs_f3;
                self.$lhs_f4 = self.$lhs_f4 $op o.$rhs_f4;
            }
        }
        }

        binop_vec4_vec4!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4;
            $rhs_type, $rhs_f1, $rhs_f2, $rhs_f3, $rhs_f4;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_scalar_scalar {
    ($lhs_type:ty, $lhs_f:ident; $rhs_type:ty, $rhs_f:ident;) => {};

    ($lhs_type:ty, $lhs_f:ident; $rhs_type:ty, $rhs_f:ident; $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f $op o.$rhs_f)
            }
        }

        paste!{
        impl<U: Basic> std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f = self.$lhs_f $op o.$rhs_f;
            }
        }
        }

        binop_scalar_scalar!($lhs_type, $lhs_f; $rhs_type, $rhs_f;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_scalar {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident,
     $lhs_f3:ident, $lhs_f4:ident; $rhs_type:ty $(,)?;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty $(,)?; $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        const _: () = {
            type T = $rhs_type;
            impl<U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $rhs_type) -> Self::Output {
                    <$lhs_type>::new(self.$lhs_f1 $op o, self.$lhs_f2 $op o,
                        self.$lhs_f3 $op o, self.$lhs_f4 $op o)
                }
            }

           impl<U: Basic> std::ops::$op_trait<$lhs_type> for $rhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $lhs_type) -> Self::Output {
                    <$lhs_type>::new(self $op o.$lhs_f1, self $op o.$lhs_f2,
                        self $op o.$lhs_f3, self $op o.$lhs_f4)
                }
            }
        };
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $rhs_type;
                $($op_trait_next, $op_fn_next, $op_next;)*);
       };

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident; $rhs_type:ty
     $(,$rhs_next:ty)+ $(,)?; $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $rhs_type;
            $($op_trait_next, $op_fn_next, $op_next;)*);
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $($rhs_next,)+;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_vec2_left {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident; $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number, U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2,
                    self.$lhs_f3, self.$lhs_f4)
            }
        }

        paste!{
        impl<T: Number, U: Basic> std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
            }
        }
        }

        binop_vec4_vec2_left!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4;
            $rhs_type, $rhs_f1, $rhs_f2; $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec2_scalar {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,)?;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,)?;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        const _: () = {
            type T = $rhs_type;
            impl<U: Basic> std::ops::$op_trait<$rhs_type> for $lhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $rhs_type) -> Self::Output {
                    <$lhs_type>::new(self.$lhs_f1 $op o, self.$lhs_f2 $op o)
                }
            }

           impl<U: Basic> std::ops::$op_trait<$lhs_type> for $rhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $lhs_type) -> Self::Output {
                    <$lhs_type>::new(self $op o.$lhs_f1, self $op o.$lhs_f2)
                }
            }
        };
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type;
                $($op_trait_next, $op_fn_next, $op_next;)*);
       };

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,$rhs_next:ty)+ $(,)?;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type;
            $($op_trait_next, $op_fn_next, $op_next;)*);
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $($rhs_next,)+;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct Rt2D<T: Number, U> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
    _u: PhantomData<U>,
}

impl<T: Number, U: Basic> Zero for Rt2D<T, U> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero(), T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number, U: Basic> Rt2D<T, U> {
    pub const fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h, _u: PhantomData }
    }

    pub const fn ptsz(p: Pt2D<T, U>, sz: Sz2D<T, U>) -> Self {
        Self { x: p.x, y: p.y, w: sz.w, h: sz.h, _u: PhantomData }
    }

    pub fn from_sz(sz: Sz2D<T, U>) -> Self {
        Self::ptsz(Pt2D::zero(), sz)
    }

    pub fn coerce<V: Basic>(&self) -> Rt2D<T, V> {
        Rt2D::ptsz(self.tl().coerce(), self.sz().coerce())
    }

    pub fn b(&self) -> T {
        self.y + self.h
    }

    pub fn r(&self) -> T {
        self.x + self.w
    }

    pub fn bl(&self) -> Pt2D<T, U> {
        Pt2D::new(self.x, self.b())
    }

    pub fn br(&self) -> Pt2D<T, U> {
        Pt2D::new(self.r(), self.b())
    }

    pub fn tl(&self) -> Pt2D<T, U> {
        Pt2D::new(self.x, self.y)
    }

    pub fn tr(&self) -> Pt2D<T, U> {
        Pt2D::new(self.r(), self.y)
    }

    pub fn sz(&self) -> Sz2D<T, U> {
        Sz2D::new(self.w, self.h)
    }

    pub fn center(&self) -> Pt2D<T, U> {
        Pt2D::new(
            self.x + self.w / num::cast::<i32, T>(2).unwrap(),
            self.y + self.h / num::cast::<i32, T>(2).unwrap(),
        )
    }

    pub fn with_sz(&self, sz: Sz2D<T, U>) -> Rt2D<T, U> {
        Rt2D::ptsz(self.tl(), sz)
    }

    pub fn inset(&self, d: Sz2D<T, U>) -> Rt2D<T, U> {
        self.inset_xy(d.w, d.h)
    }

    pub fn inset_xy(&self, dx: T, dy: T) -> Rt2D<T, U> {
        let v2 = num::cast::<i32, T>(2).unwrap();
        let wsub = if v2 * dx < self.w { v2 * dx } else { self.w };
        let hsub = if v2 * dy < self.h { v2 * dy } else { self.h };
        Rt2D::new(self.x + wsub / v2, self.y + hsub / v2, self.w - wsub, self.h - hsub)
    }

    pub fn to_f64(self) -> Rt2D<f64, U> {
        Rt2D::new(
            NumCast::from(self.x).unwrap(),
            NumCast::from(self.y).unwrap(),
            NumCast::from(self.w).unwrap(),
            NumCast::from(self.h).unwrap(),
        )
    }

    pub fn contains(&self, p: Pt2D<T, U>) -> bool {
        p.x >= self.x && p.y >= self.y && p.x <= self.r() && p.y <= self.b()
    }
}

impl<T: Number, U: Basic> From<Sz2D<T, U>> for Rt2D<T, U> {
    fn from(sz: Sz2D<T, U>) -> Self {
        Rt2D::ptsz(Pt2D::zero(), sz)
    }
}

impl<T: Number, U: Basic> From<Rt2D<T, U>> for lyon::math::Rect {
    fn from(p: Rt2D<T, U>) -> Self {
        lyon::math::Rect::new(p.tl().into(), p.sz().into())
    }
}

binop_vec4_scalar!(Rt2D<T, U>, x, y, w, h; f64, u32, i32; Mul, mul, *; Div, div, /;);
binop_vec4_vec2_left!(Rt2D<T, U>, x, y, w, h; Pt2D<T, U>, x, y; Add, add, +; Sub, sub, -;);
binop_vec4_vec4!(Rt2D<T, U>, x, y, w, h; Rt2D<T, U>, x, y, w, h; Add, add, +; Sub, sub, -;);

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct Pt2D<T: Number, U> {
    pub x: T,
    pub y: T,
    _u: PhantomData<U>,
}

impl<T: Number + Neg<Output = T>, U> Neg for Pt2D<T, U> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y, _u: PhantomData }
    }
}

impl<T: Number, U: Basic> Zero for Pt2D<T, U> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number, U> Pt2D<T, U> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y, _u: PhantomData }
    }

    pub const fn coerce<V>(&self) -> Pt2D<T, V> {
        Pt2D::new(self.x, self.y)
    }

    pub fn to_f64(&self) -> Pt2D<f64, U> {
        Pt2D::new(NumCast::from(self.x).unwrap(), NumCast::from(self.y).unwrap())
    }

    pub fn to_f32(&self) -> Pt2D<f32, U> {
        Pt2D::new(NumCast::from(self.x).unwrap(), NumCast::from(self.y).unwrap())
    }

    pub fn to_arr(&self) -> [T; 2] {
        [self.x, self.y]
    }

    pub fn to_sz(&self) -> Sz2D<T, U> {
        Sz2D::new(self.x, self.y)
    }

    pub fn as_sz(&self) -> Sz2D<T, U> {
        Sz2D::new(self.x, self.y)
    }

    pub fn offset(&self, dx: T, dy: T) -> Pt2D<T, U> {
        Pt2D::new(self.x + dx, self.y + dy)
    }
}

impl<T: Number, U> From<[T; 2]> for Pt2D<T, U> {
    fn from([x, y]: [T; 2]) -> Self {
        Pt2D::new(x, y)
    }
}

impl<T: Number, U> From<(T, T)> for Pt2D<T, U> {
    fn from((x, y): (T, T)) -> Self {
        Pt2D::new(x, y)
    }
}

impl<T: Number, U> From<&(T, T)> for Pt2D<T, U> {
    fn from((ref x, ref y): &(T, T)) -> Self {
        Pt2D::new(*x, *y)
    }
}

impl<T: Number, U> From<Pt2D<T, U>> for lyon::math::Point {
    fn from(p: Pt2D<T, U>) -> Self {
        lyon::math::Point::new(NumCast::from(p.x).unwrap(), NumCast::from(p.y).unwrap())
    }
}

binop_vec2_vec2!(Pt2D<T, U>, x, y; Pt2D<T, U>, x, y; Add, add, +; Sub, sub, -;);
binop_vec2_vec2!(Pt2D<T, U>, x, y; Sz2D<T, U>, w, h; Add, add, +; Sub, sub, -;);

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", w, h)]
pub struct Sz2D<T: Number, U> {
    pub w: T,
    pub h: T,
    _u: PhantomData<U>,
}

impl<T: Number, U: Basic> Zero for Sz2D<T, U> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number, U> Sz2D<T, U> {
    pub const fn new(w: T, h: T) -> Self {
        Self { w, h, _u: PhantomData }
    }

    pub const fn coerce<V>(&self) -> Sz2D<T, V> {
        Sz2D::new(self.w, self.h)
    }

    pub fn to_f64(&self) -> Sz2D<f64, U> {
        Sz2D::new(NumCast::from(self.w).unwrap(), NumCast::from(self.h).unwrap())
    }

    pub fn to_u32(&self) -> Sz2D<u32, U> {
        Sz2D::new(NumCast::from(self.w).unwrap(), NumCast::from(self.h).unwrap())
    }

    pub fn area(&self) -> T {
        self.w * self.h
    }

    pub fn min(self, o: Sz2D<T, U>) -> Self {
        Self::new(if self.w < o.w { self.w } else { o.w }, if self.h < o.h { self.h } else { o.h })
    }

    pub fn max(self, o: Sz2D<T, U>) -> Self {
        Self::new(if self.w > o.w { self.w } else { o.w }, if self.h > o.h { self.h } else { o.h })
    }
}

impl<T: Number, U> From<[T; 2]> for Sz2D<T, U> {
    fn from([w, h]: [T; 2]) -> Self {
        Sz2D::new(w, h)
    }
}

impl<T: Number, U> From<(T, T)> for Sz2D<T, U> {
    fn from((w, h): (T, T)) -> Self {
        Sz2D::new(w, h)
    }
}

impl<T: Number, U> From<Sz2D<T, U>> for (T, T) {
    fn from(sz: Sz2D<T, U>) -> Self {
        (sz.w, sz.h)
    }
}

impl<T: Number, U> From<Sz2D<T, U>> for lyon::math::Size {
    fn from(sz: Sz2D<T, U>) -> Self {
        lyon::math::Size::new(NumCast::from(sz.w).unwrap(), NumCast::from(sz.h).unwrap())
    }
}

impl<T: Number, U> From<Sz2D<T, U>> for lyon::math::Vector {
    fn from(sz: Sz2D<T, U>) -> Self {
        lyon::math::Vector::new(NumCast::from(sz.w).unwrap(), NumCast::from(sz.h).unwrap())
    }
}

binop_vec2_vec2!(Sz2D<T, U>, w, h; Sz2D<T, U>, w, h; Add, add, +; Sub, sub, -; Div, div, /;);
binop_vec2_scalar!(Sz2D<T, U>, w, h; f64, u32, i32; Mul, mul, *; Div, div, /;);

#[derive(
    Debug,
    Display,
    Default,
    Eq,
    PartialEq,
    Hash,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
#[display(fmt = "Z({})", z)]
pub struct ZOrder<U> {
    pub z: i32,
    _u: PhantomData<U>,
}

impl<U: Basic> Zero for ZOrder<U> {
    fn zero() -> Self {
        Self::new(0)
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<U> ZOrder<U> {
    pub const fn new(z: i32) -> Self {
        Self { z, _u: PhantomData }
    }

    pub const fn coerce<V>(&self) -> ZOrder<V> {
        ZOrder::new(self.z)
    }
}

binop_scalar_scalar!(ZOrder<U>, z; ZOrder<U>, z; Add, add, +; Sub, sub, -;);

#[derive(
    Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Display, From, Into,
)]
pub struct TexType;

pub type TexRt = Rt2D<u32, TexType>;
pub type TexPt = Pt2D<u32, TexType>;
pub type TexSz = Sz2D<u32, TexType>;
pub type TexCoord = Pt2D<f64, TexType>;
pub type TexUvRect = Rt2D<f64, TexType>;

pub fn trt<X: ToPrimitive, Y: ToPrimitive, W: ToPrimitive, H: ToPrimitive>(
    x: X,
    y: Y,
    w: W,
    h: H,
) -> TexRt {
    Rt2D::new(x.to_u32().unwrap(), y.to_u32().unwrap(), w.to_u32().unwrap(), h.to_u32().unwrap())
}

pub fn tpt<X: ToPrimitive, Y: ToPrimitive>(x: X, y: Y) -> TexPt {
    Pt2D::new(x.to_u32().unwrap(), y.to_u32().unwrap())
}

pub fn tsz<W: ToPrimitive, H: ToPrimitive>(w: W, h: H) -> TexSz {
    Sz2D::new(w.to_u32().unwrap(), h.to_u32().unwrap())
}

impl<T: ToPrimitive> From<PhysicalSize<T>> for TexSz {
    fn from(sz: PhysicalSize<T>) -> Self {
        tsz(sz.width, sz.height)
    }
}

#[derive(
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Display,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct GblType;

pub type GblRt = Rt2D<f64, GblType>;
pub type GblPt = Pt2D<f64, GblType>;
pub type GblSz = Sz2D<f64, GblType>;
pub type GblZ = ZOrder<GblType>;

pub fn grt<X: ToPrimitive, Y: ToPrimitive, W: ToPrimitive, H: ToPrimitive>(
    x: X,
    y: Y,
    w: W,
    h: H,
) -> GblRt {
    Rt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap(), w.to_f64().unwrap(), h.to_f64().unwrap())
}

pub fn gpt<X: ToPrimitive, Y: ToPrimitive>(x: X, y: Y) -> GblPt {
    Pt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap())
}

pub fn gsz<W: ToPrimitive, H: ToPrimitive>(w: W, h: H) -> GblSz {
    Sz2D::new(w.to_f64().unwrap(), h.to_f64().unwrap())
}

impl<T: ToPrimitive> From<LogicalSize<T>> for GblSz {
    fn from(sz: LogicalSize<T>) -> Self {
        gsz(sz.width, sz.height)
    }
}

impl<T: ToPrimitive> From<LogicalPosition<T>> for GblPt {
    fn from(p: LogicalPosition<T>) -> Self {
        gpt(p.x, p.y)
    }
}

#[derive(
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Display,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct LclType;

pub type LclRt = Rt2D<f64, LclType>;
pub type LclPt = Pt2D<f64, LclType>;
pub type LclSz = Sz2D<f64, LclType>;
pub type LclZ = ZOrder<LclType>;

pub fn lrt<X: ToPrimitive, Y: ToPrimitive, W: ToPrimitive, H: ToPrimitive>(
    x: X,
    y: Y,
    w: W,
    h: H,
) -> LclRt {
    Rt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap(), w.to_f64().unwrap(), h.to_f64().unwrap())
}

pub fn lpt<X: ToPrimitive, Y: ToPrimitive>(x: X, y: Y) -> LclPt {
    Pt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap())
}

pub fn lsz<W: ToPrimitive, H: ToPrimitive>(w: W, h: H) -> LclSz {
    Sz2D::new(w.to_f64().unwrap(), h.to_f64().unwrap())
}

pub fn lz<Z: ToPrimitive>(z: Z) -> LclZ {
    ZOrder::new(z.to_i32().unwrap())
}

#[derive(
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Display,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct PrtType;

pub type PrtRt = Rt2D<f64, PrtType>;
pub type PrtPt = Pt2D<f64, PrtType>;
pub type PrtSz = Sz2D<f64, PrtType>;
pub type PrtZ = ZOrder<PrtType>;

pub type Rt = Rt2D<f64, Any>;
pub type Pt = Pt2D<f64, Any>;
pub type Sz = Sz2D<f64, Any>;

pub fn rt<X: ToPrimitive, Y: ToPrimitive, W: ToPrimitive, H: ToPrimitive>(
    x: X,
    y: Y,
    w: W,
    h: H,
) -> Rt {
    Rt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap(), w.to_f64().unwrap(), h.to_f64().unwrap())
}

pub fn pt<X: ToPrimitive, Y: ToPrimitive>(x: X, y: Y) -> Pt {
    Pt2D::new(x.to_f64().unwrap(), y.to_f64().unwrap())
}

pub const MAX_Z: LclZ = ZOrder::new(100000000);
