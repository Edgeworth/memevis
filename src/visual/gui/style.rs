use std::cell::Cell;
use std::sync::Arc;

use rgb::{ComponentMap, RGBA};

use crate::visual::gui::layer::GblTf;
use crate::visual::render::painter::PaintCtx;
use crate::visual::types::{lsz, Col, LclSz, LclZ};
use crate::visual::vis::FontId;

pub struct PaintCtxScope {
    pctx: Arc<Cell<PaintCtx>>,
    restore_pctx: PaintCtx,
}

#[allow(dead_code)]
impl PaintCtxScope {
    #[must_use]
    pub fn new(pctx: Arc<Cell<PaintCtx>>, restore_pctx: PaintCtx) -> Self {
        Self { pctx, restore_pctx }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn tf(&self, tf: GblTf) -> &Self {
        self.pctx.set(self.pctx.get().tf(tf));
        self
    }

    #[allow(clippy::must_use_candidate)]
    pub fn col(&self, col: Col) -> &Self {
        self.pctx.set(self.pctx.get().col(col));
        self
    }

    #[allow(clippy::must_use_candidate)]
    pub fn z(&self, z: LclZ) -> &Self {
        self.pctx.set(self.pctx.get().z(z));
        self
    }

    #[allow(clippy::must_use_candidate)]
    pub fn line_width(&self, line_width: f64) -> &Self {
        self.pctx.set(self.pctx.get().line_width(line_width));
        self
    }
}

impl Drop for PaintCtxScope {
    fn drop(&mut self) {
        self.pctx.set(self.restore_pctx);
    }
}

#[derive(Debug)]
pub struct Style {
    pub pad: LclSz,
    pub font: FontId,
    pub font_sz: f64,
    pub dark_col: Col,
    pub light_col: Col,
    pub acc1_col: Col,
    pub acc2_col: Col,
    pub acc3_col: Col,
}

impl Style {
    #[must_use]
    pub fn new() -> Self {
        let f = |c| c as f32 / 255.0;
        Self {
            pad: lsz(8, 8),
            font: 0,
            font_sz: 12.0,
            dark_col: RGBA::new(7, 7, 7, 255).map(f),
            light_col: RGBA::new(233, 241, 247, 255).map(f),
            acc1_col: RGBA::new(249, 220, 92, 255).map(f),
            acc2_col: RGBA::new(250, 130, 76, 255).map(f),
            acc3_col: RGBA::new(60, 145, 230, 255).map(f),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}
