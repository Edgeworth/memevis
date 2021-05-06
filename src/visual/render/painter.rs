use crate::visual::colors::WHITE;
use crate::visual::gui::layer::GblTf;
use crate::visual::render::atlas::{Atlas, AtlasHandle};
use crate::visual::render::texture::{TexStore, TextureLayer};
use crate::visual::types::{lz, Col, LclPt, LclRt, LclSz, LclZ, TexPt, TexSz};
use eyre::Result;
use glium::glutin::window::CursorIcon;
use lyon::math::Angle;
use lyon::path::Path;
use rgb::RGBA8;

pub type TexHandle = AtlasHandle;

#[derive(Debug)]
pub(super) enum PaintOp {
    FillPath {
        p: Path,
    },
    FillCirc {
        center: LclPt,
        radius: f64,
    },
    FillPoly {
        pts: Vec<LclPt>,
    },
    FillQuad {
        v: [LclPt; 4],
    },
    FillRt {
        r: LclRt,
    },
    FillRRt {
        r: LclRt,
        radius: f64,
    },
    StrokeLine {
        st: LclPt,
        en: LclPt,
    },
    StrokePath {
        p: Path,
    },
    StrokeCirc {
        center: LclPt,
        radius: f64,
    },
    StrokeEllipse {
        center: LclPt,
        radii: LclSz,
        rot: Angle,
    },
    StrokePoly {
        pts: Vec<LclPt>,
        is_closed: bool,
    },
    StrokeQuad {
        v: [LclPt; 4],
    },
    StrokeRt {
        r: LclRt,
    },
    StrokeRRt {
        r: LclRt,
        radius: f64,
    },
    StrokeTri {
        v: [LclPt; 3],
    },
    Texture {
        tex: TextureLayer,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PaintCtx {
    pub tf: GblTf,
    pub z: LclZ,
    pub col: Col,
    pub line_width: f64,
}

impl Default for PaintCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl PaintCtx {
    pub fn new() -> Self {
        Self {
            line_width: 1.0,
            z: lz(0),
            col: WHITE,
            tf: GblTf::default(),
        }
    }

    pub fn tf(self, tf: GblTf) -> Self {
        Self { tf, ..self }
    }

    pub fn col(self, col: Col) -> Self {
        Self { col, ..self }
    }

    pub fn z(self, z: LclZ) -> Self {
        Self { z, ..self }
    }

    pub fn line_width(self, line_width: f64) -> Self {
        Self { line_width, ..self }
    }
}

#[derive(Debug)]
pub struct Painter {
    pub(super) ops: Vec<(PaintCtx, PaintOp)>,
    pub(super) ts: TexStore,
    pub(super) cursor: CursorIcon,
    atlas: Atlas,
}

impl Painter {
    pub fn new() -> Self {
        let mut ts = TexStore::new();
        let atlas = Atlas::new(&mut ts);
        Self {
            ops: Vec::new(),
            ts,
            cursor: CursorIcon::Default,
            atlas,
        }
    }

    pub fn begin(&mut self) {
        self.ops.clear();
        self.cursor = CursorIcon::Default;
    }

    pub fn fill_path(&mut self, pctx: PaintCtx, p: Path) {
        self.ops.push((pctx, PaintOp::FillPath { p }));
    }

    pub fn fill_circ(&mut self, pctx: PaintCtx, center: LclPt, radius: f64) {
        self.ops.push((pctx, PaintOp::FillCirc { center, radius }));
    }

    pub fn fill_poly(&mut self, pctx: PaintCtx, pts: Vec<LclPt>) {
        self.ops.push((pctx, PaintOp::FillPoly { pts }));
    }

    pub fn fill_quad(&mut self, pctx: PaintCtx, v: [LclPt; 4]) {
        self.ops.push((pctx, PaintOp::FillQuad { v }));
    }

    pub fn fill_rt(&mut self, pctx: PaintCtx, r: LclRt) {
        self.ops.push((pctx, PaintOp::FillRt { r }));
    }

    pub fn fill_rrt(&mut self, pctx: PaintCtx, r: LclRt, radius: f64) {
        self.ops.push((pctx, PaintOp::FillRRt { r, radius }));
    }

    pub fn stroke_line(&mut self, pctx: PaintCtx, st: LclPt, en: LclPt) {
        self.ops.push((pctx, PaintOp::StrokeLine { st, en }));
    }

    pub fn stroke_path(&mut self, pctx: PaintCtx, p: Path) {
        self.ops.push((pctx, PaintOp::StrokePath { p }));
    }

    pub fn stroke_circ(&mut self, pctx: PaintCtx, center: LclPt, radius: f64) {
        self.ops
            .push((pctx, PaintOp::StrokeCirc { center, radius }));
    }

    pub fn stroke_ellipse(&mut self, pctx: PaintCtx, center: LclPt, radii: LclSz, rot: Angle) {
        self.ops
            .push((pctx, PaintOp::StrokeEllipse { center, radii, rot }));
    }

    pub fn stroke_poly(&mut self, pctx: PaintCtx, pts: Vec<LclPt>) {
        self.ops.push((
            pctx,
            PaintOp::StrokePoly {
                pts,
                is_closed: true,
            },
        ));
    }

    pub fn stroke_quad(&mut self, pctx: PaintCtx, v: [LclPt; 4]) {
        self.ops.push((pctx, PaintOp::StrokeQuad { v }));
    }

    pub fn stroke_rt(&mut self, pctx: PaintCtx, r: LclRt) {
        self.ops.push((pctx, PaintOp::StrokeRt { r }));
    }

    pub fn stroke_rrt(&mut self, pctx: PaintCtx, r: LclRt, radius: f64) {
        self.ops.push((pctx, PaintOp::StrokeRRt { r, radius }));
    }

    pub fn stroke_tri(&mut self, pctx: PaintCtx, v: [LclPt; 3]) {
        self.ops.push((pctx, PaintOp::StrokeTri { v }));
    }

    pub fn tex(&mut self, pctx: PaintCtx, tex: TextureLayer) {
        self.ops.push((pctx, PaintOp::Texture { tex }));
    }

    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.cursor = cursor;
    }

    pub fn alloc(&mut self, sz: TexSz) -> Result<TexHandle> {
        self.atlas.alloc(sz)
    }

    pub fn get_tex(&mut self, hnd: TexHandle) -> TextureLayer {
        self.atlas.get_layer(hnd)
    }

    pub fn write_px(&mut self, hnd: TexHandle, p: TexPt, c: RGBA8) {
        self.atlas.write_px(&mut self.ts, hnd, p, c)
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}
