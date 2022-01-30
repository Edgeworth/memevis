use num_traits::Zero;

use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::layout::{LayoutInfo, LayoutStrategy};
use crate::visual::gui::layouts::util::compute_child_info;
use crate::visual::gui::ui::Ui;
use crate::visual::types::{lz, LclPt, LclSz};

#[derive(Debug, Copy, Clone)]
pub struct VertLayout {
    info: LayoutInfo,
    loc: LclPt,
    sz: LclSz,
}

impl VertLayout {
    #[must_use]
    pub fn new(info: LayoutInfo) -> Self {
        Self { info, loc: LclPt::zero(), sz: LclSz::zero() }
    }

    fn advance_cursor(&mut self, l: &LclLayer) {
        self.loc.y += l.r.h;
        self.sz.w = self.sz.w.max(l.r.w);
        self.sz.h += l.r.h;
        self.info.hint.min =
            self.info.hint.min.iter().chain(&[self.sz]).copied().reduce(|a, b| a.max(&b));
    }
}

impl LayoutStrategy for VertLayout {
    fn info(&self) -> &LayoutInfo {
        &self.info
    }

    fn child_info(&mut self, _ui: &mut Ui<'_>, hint: &Hint, _child_id: &str) -> LayoutInfo {
        compute_child_info(&self.info, self.loc.coerce(), lz(1), hint)
    }

    fn place_layer(&mut self, _ui: &mut Ui<'_>, l: &LclLayer, _: &str) {
        self.advance_cursor(l);
    }
}
