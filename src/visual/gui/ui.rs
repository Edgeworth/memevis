use crate::visual::colors::{GREEN, RED};
use crate::visual::gui::layer::{LclLayer, PrtLayer};
use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::layout::{Layout, LayoutInfo};
use crate::visual::gui::style::{PaintCtxScope, Style};
use crate::visual::gui::text::Frag;
use crate::visual::gui::widgets::button::Button;
use crate::visual::gui::widgets::label::Label;
use crate::visual::gui::widgets::widget::{combine_ids, Resp, Widget};
use crate::visual::gui::widgets::window::Window;
use crate::visual::io::Io;
use crate::visual::render::painter::{PaintCtx, Painter};
use crate::visual::render::texture::TextureLayer;
use crate::visual::types::{LclPt, LclRt, LclSz, Pt, MAX_Z};
use crate::visual::vis::{Memory, Vis};
use eyre::Result;
use lyon::math::Angle;
use lyon::path::Path;
use num_traits::Zero;
use parking_lot::MutexGuard;
use std::cell::Cell;
use std::sync::Arc;

pub struct Ui {
    pub s: Style,
    v: Vis,
    l: Layout,
    id: String,
    pctx: Arc<Cell<PaintCtx>>,
}

impl Ui {
    pub fn new(v: Vis, l: Layout, id: &str) -> Self {
        let s = Style::new();
        let pctx = PaintCtx { tf: l.info().gtf, col: s.light_col, ..Default::default() };
        Self { s, v, l, id: id.to_owned(), pctx: Arc::new(Cell::new(pctx)) }
    }

    pub fn mem(&self) -> MutexGuard<'_, Memory> {
        self.v.mem()
    }

    pub fn io(&self) -> MutexGuard<'_, Io> {
        self.v.io()
    }

    pub fn paint(&self) -> MutexGuard<'_, Painter> {
        self.v.paint()
    }

    pub fn pctx(&self) -> PaintCtx {
        self.pctx.get()
    }

    pub fn push(&self) -> PaintCtxScope {
        let restore_pctx = self.pctx.get();
        let pctx = Arc::clone(&self.pctx);
        PaintCtxScope::new(pctx, restore_pctx)
    }
}

// Layout
#[allow(dead_code)]
impl Ui {
    pub fn child<LayoutF, UiF>(
        &mut self,
        hint: &Hint,
        child_id: &str,
        mut layout_f: LayoutF,
        mut ui_f: UiF,
    ) -> Result<LclLayer>
    where
        LayoutF: FnMut(LayoutInfo) -> Layout,
        UiF: FnMut(&mut Ui) -> Result<()>,
    {
        // Copy - layouts see a frozen version of themselves from
        // accessing via Ui.
        let mut layout = self.l.clone();
        let layer = layout.child(self, hint, child_id, |ui, params| {
            let mut ui = Ui::new(ui.v.clone(), layout_f(params), child_id);
            ui_f(&mut ui)?;
            Ok(ui.l)
        })?;
        if self.mem().debug {
            let _scope = self.push().z(MAX_Z).col(GREEN);
            self.stroke_rt(layer.r);
        }
        self.l = layout;
        Ok(layer)
    }

    pub fn child_layer(&mut self, hint: &Hint) -> LclLayer {
        // Copy - layouts see a frozen version of themselves from
        // accessing via Ui.
        let mut layout = self.l.clone();
        let layer = layout.child_layer(self, hint);
        if self.mem().debug {
            let _scope = self.push().z(MAX_Z).col(RED);
            self.stroke_rt(layer.r);
        }
        self.l = layout;
        layer
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

    pub fn wid(&self, w: &impl Widget) -> String {
        combine_ids(&[&self.id(), &w.lcl_id(self)])
    }

    pub fn hovered(&self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let mut io = self.v.io();
        let contained = l.contains(io.mouse_pt);
        if contained {
            io.mouse_req(l.z, id);
        }
        io.has_mouse.as_deref() == Some(id) && contained
    }

    pub fn scrolled(&self, id: &str, l: LclLayer) -> Pt {
        if self.hovered(id, l) { self.io().mouse_scroll } else { Pt::zero() }
    }

    pub fn pressed(&self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let mut io = self.v.io();
        let capture = io.is_mouse_pressed
            && (io.mouse_captured.as_deref() == Some(id) || l.contains(io.mouse_pt));
        if capture {
            io.mouse_capture(l.z, id); // Prolong mouse capture.
        }
        io.has_mouse.as_deref() == Some(id) && capture
    }

    pub fn clicked(&self, id: &str, l: LclLayer) -> bool {
        let l = self.l.info().gtf.layer(l);
        let io = self.v.io();
        io.has_mouse.as_deref() == Some(id)
            && io.mouse_just_released
            && io.mouse_captured.as_deref() == Some(id)
            && l.contains(io.mouse_pt)
    }
}

// Widgets.
impl Ui {
    pub fn label(&mut self, text: &str) -> Result<Resp> {
        Label::new(text).ui(self)
    }

    pub fn button<F: FnOnce(&mut Ui)>(&mut self, text: &str, cb: F) -> Result<Resp> {
        Button::new(text, cb).ui(self)
    }

    pub fn window(&mut self, title: &str, f: impl FnMut(&mut Ui) -> Result<()>) -> Result<Resp> {
        Window::new(title, f).ui(self)
    }
}

// Drawing.
#[allow(dead_code)]
impl Ui {
    pub fn text_sz(&mut self, f: &Frag) -> Result<LclSz> {
        let sz = self.v.layout_text(&f.text, f.sz)?;
        Ok(self.l.info().gtf.inv().sz(sz))
    }

    pub fn text(&mut self, f: &Frag) -> Result<()> {
        let l = f.layout(self)?;
        let l = self.l.info().gtf.layer(l);
        self.v.draw_text(&f.text, f.sz, &l)
    }

    pub fn fill_path(&self, p: Path) {
        self.v.paint().fill_path(self.pctx.get(), p);
    }

    pub fn fill_circ(&self, p: LclPt, radius: f64) {
        self.v.paint().fill_circ(self.pctx.get(), p, radius);
    }

    pub fn fill_poly(&self, pts: Vec<LclPt>) {
        self.v.paint().fill_poly(self.pctx.get(), pts);
    }

    pub fn fill_quad(&self, v: [LclPt; 4]) {
        self.v.paint().fill_quad(self.pctx.get(), v);
    }

    pub fn fill_rt(&self, r: LclRt) {
        self.v.paint().fill_rt(self.pctx.get(), r);
    }

    pub fn fill_rrt(&self, r: LclRt, radius: f64) {
        self.v.paint().fill_rrt(self.pctx.get(), r, radius);
    }

    pub fn stroke_line(&self, st: LclPt, en: LclPt) {
        self.v.paint().stroke_line(self.pctx.get(), st, en);
    }

    pub fn stroke_path(&self, p: Path) {
        self.v.paint().stroke_path(self.pctx.get(), p);
    }

    pub fn stroke_circ(&self, p: LclPt, radius: f64) {
        self.v.paint().stroke_circ(self.pctx.get(), p, radius);
    }

    pub fn stroke_ellipse(&self, p: LclPt, radii: LclSz, rot: Angle) {
        self.v.paint().stroke_ellipse(self.pctx.get(), p, radii, rot);
    }

    pub fn stroke_poly(&self, pts: Vec<LclPt>) {
        self.v.paint().stroke_poly(self.pctx.get(), pts);
    }

    pub fn stroke_quad(&self, v: [LclPt; 4]) {
        self.v.paint().stroke_quad(self.pctx.get(), v);
    }

    pub fn stroke_rt(&self, r: LclRt) {
        self.v.paint().stroke_rt(self.pctx.get(), r);
    }

    pub fn stroke_rrt(&self, r: LclRt, radius: f64) {
        self.v.paint().stroke_rrt(self.pctx.get(), r, radius);
    }

    pub fn stroke_tri(&self, v: [LclPt; 3]) {
        self.v.paint().stroke_tri(self.pctx.get(), v);
    }

    pub fn tex(&self, tex: TextureLayer) {
        self.v.paint().tex(self.pctx.get(), tex);
    }
}
