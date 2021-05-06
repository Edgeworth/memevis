use crate::visual::colors::{GREEN, RED};
use crate::visual::gui::layer::{GblTf, LclLayer, PrtLayer};
use crate::visual::gui::layouts::layout::{Hint, Layout, LayoutInfo};
use crate::visual::gui::layouts::vert_layout::VertLayout;
use crate::visual::gui::text::Frag;
use crate::visual::gui::widgets::button::Button;
use crate::visual::gui::widgets::label::Label;
use crate::visual::gui::widgets::widget::{combine_ids, Resp, Widget};
use crate::visual::gui::widgets::window::Window;
use crate::visual::io::Io;
use crate::visual::render::painter::{PaintCtx, Painter};
use crate::visual::render::texture::TextureLayer;
use crate::visual::types::{lsz, Col, LclPt, LclRt, LclSz, LclZ, Pt, MAX_Z};
use crate::visual::vis::{FontId, Memory, Vis};
use eyre::Result;
use lyon::math::Angle;
use lyon::path::Path;
use num_traits::Zero;
use rgb::{ComponentMap, RGBA};
use std::cell::Cell;
use std::rc::Rc;

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

pub struct PaintCtxScope {
    pctx: Rc<Cell<PaintCtx>>,
    restore_pctx: PaintCtx,
}

#[allow(dead_code)]
impl PaintCtxScope {
    pub fn tf(&self, tf: GblTf) -> &Self {
        self.pctx.set(self.pctx.get().tf(tf));
        self
    }

    pub fn col(&self, col: Col) -> &Self {
        self.pctx.set(self.pctx.get().col(col));
        self
    }

    pub fn z(&self, z: LclZ) -> &Self {
        self.pctx.set(self.pctx.get().z(z));
        self
    }

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

pub struct Ui<'a, L: Layout> {
    pub s: Style,
    v: &'a mut Vis,
    l: L,
    id: String,
    pctx: Rc<Cell<PaintCtx>>,
}

impl<'a, L: Layout> Ui<'a, L> {
    pub fn new(c: &'a mut Vis, l: L, id: &str) -> Self {
        let s = Style::new();
        let pctx = PaintCtx { tf: l.info().gtf, col: s.light_col, ..Default::default() };
        Self { s, v: c, l, id: id.to_owned(), pctx: Rc::new(Cell::new(pctx)) }
    }

    pub fn m(&mut self) -> &mut Memory {
        &mut self.v.mem
    }

    pub fn io(&mut self) -> &mut Io {
        &mut self.v.io
    }

    pub fn p(&mut self) -> &mut Painter {
        &mut self.v.p
    }

    pub fn pctx(&self) -> PaintCtx {
        self.pctx.get()
    }

    pub fn push(&mut self, pctx: PaintCtx) -> PaintCtxScope {
        let restore_pctx = self.pctx.replace(pctx);
        PaintCtxScope { pctx: Rc::clone(&self.pctx), restore_pctx }
    }
}

// Layout
#[allow(dead_code)]
impl<'a, L: Layout> Ui<'a, L> {
    pub fn child<ChildL, LayoutF, UiF>(
        &mut self,
        hint: &Hint,
        child_id: &str,
        mut layout_f: LayoutF,
        mut ui_f: UiF,
    ) -> Result<LclLayer>
    where
        ChildL: Layout,
        LayoutF: FnMut(LayoutInfo) -> ChildL,
        UiF: FnMut(&mut Ui<'_, ChildL>) -> Result<()>,
    {
        // Copy - layouts see a frozen version of themselves from
        // accessing via Ui.
        let mut layout = self.l;
        let layer = layout.child(self, hint, child_id, |ui, params| {
            let mut ui = Ui::new(ui.v, layout_f(params), child_id);
            ui_f(&mut ui)?;
            Ok(ui.l)
        })?;
        if self.m().debug {
            let _scope = self.push(self.pctx().z(MAX_Z).col(GREEN));
            self.stroke_rt(layer.r);
        }
        self.l = layout;
        Ok(layer)
    }

    pub fn child_layer(&mut self, hint: &Hint) -> Result<LclLayer> {
        // Copy - layouts see a frozen version of themselves from
        // accessing via Ui.
        let mut layout = self.l;
        let layer = layout.child_layer(self, hint)?;
        if self.m().debug {
            let _scope = self.push(self.pctx().z(MAX_Z).col(RED));
            self.stroke_rt(layer.r);
        }
        self.l = layout;
        Ok(layer)
    }

    pub fn compute_layer(&mut self) -> PrtLayer {
        self.l.compute_layer()
    }

    pub fn info(&self) -> &LayoutInfo {
        self.l.info()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn wid(&self, w: &impl Widget<L>) -> String {
        combine_ids(&[&self.id(), &w.lcl_id(self)])
    }

    pub fn hovered(&mut self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let io = &mut self.v.io;
        let contained = l.contains(io.mouse_pt);
        if contained {
            io.mouse_req(l.z, id);
        }
        io.has_mouse.as_deref() == Some(id) && contained
    }

    pub fn scrolled(&mut self, id: &str, l: LclLayer) -> Pt {
        if self.hovered(id, l) { self.io().mouse_scroll } else { Pt::zero() }
    }

    pub fn pressed(&mut self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let io = &mut self.v.io;
        let capture = io.is_mouse_pressed
            && (io.mouse_captured.as_deref() == Some(id) || l.contains(io.mouse_pt));
        if capture {
            io.mouse_capture(l.z, id); // Prolong mouse capture.
        }
        io.has_mouse.as_deref() == Some(id) && capture
    }

    pub fn clicked(&mut self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let io = &mut self.v.io;
        io.has_mouse.as_deref() == Some(id)
            && io.mouse_just_released
            && io.mouse_captured.as_deref() == Some(id)
            && l.contains(io.mouse_pt)
    }
}

// Widgets.
impl<'a, L: Layout> Ui<'a, L> {
    pub fn label(&mut self, text: &str) -> Result<Resp> {
        Label::new(text).ui(self)
    }

    pub fn button<F: FnOnce(&mut Ui<'_, L>)>(&mut self, text: &str, cb: F) -> Result<Resp> {
        Button::new(text, cb).ui(self)
    }

    pub fn window(
        &mut self,
        title: &str,
        f: impl FnMut(&mut Ui<'_, VertLayout>) -> Result<()>,
    ) -> Result<Resp> {
        Window::new(title, f).ui(self)
    }
}

// Drawing.
#[allow(dead_code)]
impl<'a, L: Layout> Ui<'a, L> {
    pub fn text_sz(&mut self, f: &Frag) -> Result<LclSz> {
        let sz = self.v.layout_text(&f.text, f.sz)?;
        Ok(self.l.info().gtf.inv().sz(sz))
    }

    pub fn text(&mut self, f: &Frag) -> Result<()> {
        let l = f.layout(self)?;
        let l = self.l.info().gtf.layer(l);
        self.v.draw_text(&f.text, f.sz, &l)
    }

    pub fn fill_path(&mut self, p: Path) {
        self.v.p.fill_path(self.pctx.get(), p);
    }

    pub fn fill_circ(&mut self, p: LclPt, radius: f64) {
        self.v.p.fill_circ(self.pctx.get(), p, radius);
    }

    pub fn fill_poly(&mut self, pts: Vec<LclPt>) {
        self.v.p.fill_poly(self.pctx.get(), pts);
    }

    pub fn fill_quad(&mut self, v: [LclPt; 4]) {
        self.v.p.fill_quad(self.pctx.get(), v);
    }

    pub fn fill_rt(&mut self, r: LclRt) {
        self.v.p.fill_rt(self.pctx.get(), r);
    }

    pub fn fill_rrt(&mut self, r: LclRt, radius: f64) {
        self.v.p.fill_rrt(self.pctx.get(), r, radius);
    }

    pub fn stroke_line(&mut self, st: LclPt, en: LclPt) {
        self.v.p.stroke_line(self.pctx.get(), st, en);
    }

    pub fn stroke_path(&mut self, p: Path) {
        self.v.p.stroke_path(self.pctx.get(), p);
    }

    pub fn stroke_circ(&mut self, p: LclPt, radius: f64) {
        self.v.p.stroke_circ(self.pctx.get(), p, radius);
    }

    pub fn stroke_ellipse(&mut self, p: LclPt, radii: LclSz, rot: Angle) {
        self.v.p.stroke_ellipse(self.pctx.get(), p, radii, rot);
    }

    pub fn stroke_poly(&mut self, pts: Vec<LclPt>) {
        self.v.p.stroke_poly(self.pctx.get(), pts);
    }

    pub fn stroke_quad(&mut self, v: [LclPt; 4]) {
        self.v.p.stroke_quad(self.pctx.get(), v);
    }

    pub fn stroke_rt(&mut self, r: LclRt) {
        self.v.p.stroke_rt(self.pctx.get(), r);
    }

    pub fn stroke_rrt(&mut self, r: LclRt, radius: f64) {
        self.v.p.stroke_rrt(self.pctx.get(), r, radius);
    }

    pub fn stroke_tri(&mut self, v: [LclPt; 3]) {
        self.v.p.stroke_tri(self.pctx.get(), v);
    }

    pub fn tex(&mut self, tex: TextureLayer) {
        self.v.p.tex(self.pctx.get(), tex);
    }
}
