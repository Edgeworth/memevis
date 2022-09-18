use std::marker::PhantomData;

use derive_more::Display;
use lyon::math::Translation;
use lyon::path::Path;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::any::Basic;
use crate::visual::types::{GblType, LclType, PrtType, Pt2D, Rt2D, Sz2D, ZOrder};

#[derive(Debug, Default, PartialEq, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "Layer({r}; {z})")]
pub struct Layer<U> {
    pub r: Rt2D<f64, U>,
    pub z: ZOrder<U>,
    _u: PhantomData<U>,
}

impl<U: Basic> Layer<U> {
    #[must_use]
    pub const fn new(r: Rt2D<f64, U>, z: ZOrder<U>) -> Self {
        Self { r, z, _u: PhantomData }
    }

    #[must_use]
    pub fn coerce<V: Basic>(&self) -> Layer<V> {
        Layer::new(self.r.coerce(), self.z.coerce())
    }

    #[must_use]
    pub fn from_sz(sz: Sz2D<f64, U>) -> Self {
        Self::new(Rt2D::from_sz(sz), ZOrder::zero())
    }

    #[must_use]
    pub fn contains(&self, p: Pt2D<f64, U>) -> bool {
        self.r.contains(p)
    }

    #[must_use]
    pub fn inset(&self, d: Sz2D<f64, U>) -> Self {
        Self::new(self.r.inset(d), self.z)
    }
}

pub type GblLayer = Layer<GblType>;
pub type PrtLayer = Layer<PrtType>;
pub type LclLayer = Layer<LclType>;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct LayerTf<S, T> {
    off: Pt2D<f64, T>,
    z: ZOrder<T>,
    _s: PhantomData<S>,
    _t: PhantomData<T>,
}

impl<S: Basic, T: Basic> LayerTf<S, T> {
    #[must_use]
    pub const fn new(off: Pt2D<f64, T>, z: ZOrder<T>) -> Self {
        Self { off, z, _s: PhantomData, _t: PhantomData }
    }

    #[must_use]
    pub fn concat<U: Basic>(&self, tf: &LayerTf<T, U>) -> LayerTf<S, U> {
        LayerTf::new(self.off.coerce() + tf.off, tf.z(self.z))
    }

    #[must_use]
    pub const fn coerce<U: Basic, V: Basic>(&self) -> LayerTf<U, V> {
        LayerTf::new(self.off.coerce(), self.z.coerce())
    }

    #[must_use]
    pub fn inv(&self) -> LayerTf<T, S> {
        LayerTf::new(-self.off.coerce(), ZOrder::new(-self.z.z))
    }

    #[must_use]
    pub fn layer(&self, l: Layer<S>) -> Layer<T> {
        Layer::new(self.rt(l.r), self.z(l.z))
    }

    #[must_use]
    pub fn pt(&self, p: Pt2D<f64, S>) -> Pt2D<f64, T> {
        Pt2D::new(p.x + self.off.x, p.y + self.off.y)
    }

    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn sz(&self, sz: Sz2D<f64, S>) -> Sz2D<f64, T> {
        Sz2D::new(sz.w, sz.h)
    }

    #[must_use]
    pub fn rt(&self, r: Rt2D<f64, S>) -> Rt2D<f64, T> {
        Rt2D::ptsz(self.pt(r.tl()), self.sz(r.sz()))
    }

    #[must_use]
    pub fn z(&self, z: ZOrder<S>) -> ZOrder<T> {
        self.z + z.coerce()
    }

    #[must_use]
    pub fn path(&self, p: Path) -> Path {
        p.transformed(&Translation::new(self.off.x as f32, self.off.y as f32))
    }
}

pub type PrtTf = LayerTf<LclType, PrtType>;
pub type GblTf = LayerTf<LclType, GblType>;
