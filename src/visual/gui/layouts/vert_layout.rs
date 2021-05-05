use crate::visual::gui::layer::{LclLayer, PrtLayer};
use crate::visual::gui::layouts::layout::{Hint, Layout, LayoutInfo};
use crate::visual::gui::layouts::util::{compute_child_info, natural_layer_in_parent};
use crate::visual::gui::ui::Ui;
use crate::visual::types::{lz, LclPt, LclSz};
use eyre::Result;
use num_traits::Zero;

#[derive(Debug, Copy, Clone)]
pub struct VertLayout {
    info: LayoutInfo,
    loc: LclPt,
    sz: LclSz,
}

impl VertLayout {
    pub fn new(info: LayoutInfo) -> Self {
        Self { info, loc: LclPt::zero(), sz: LclSz::zero() }
    }

    fn advance_cursor(&mut self, l: &LclLayer) {
        self.loc.y += l.r.h;
        self.sz.w = self.sz.w.max(l.r.w);
        self.sz.h += l.r.h;
        self.info.hint.min =
            self.info.hint.min.iter().chain(&[self.sz]).copied().reduce(LclSz::max);
    }
}

impl Layout for VertLayout {
    fn info(&self) -> &LayoutInfo {
        &self.info
    }

    fn child<UiF, L, ChildL>(
        &mut self,
        ui: &mut Ui<'_, L>,
        hint: &Hint,
        _child_id: &str,
        mut f: UiF,
    ) -> Result<LclLayer>
    where
        L: Layout,
        ChildL: Layout,
        UiF: FnMut(&mut Ui<'_, L>, LayoutInfo) -> Result<ChildL>,
    {
        let info = compute_child_info(&self.info, self.loc.coerce(), lz(1), hint);
        let mut layout = f(ui, info)?;
        let layer: LclLayer = layout.compute_layer().coerce(); // Parent is local here.
        self.advance_cursor(&layer);
        Ok(layer)
    }

    fn child_layer<L: Layout>(&mut self, _: &mut Ui<'_, L>, hint: &Hint) -> Result<LclLayer> {
        let info = compute_child_info(self.info(), self.loc, lz(1), hint);
        let l = natural_layer_in_parent(&info).coerce();
        self.advance_cursor(&l);
        Ok(l)
    }

    fn compute_layer(&mut self) -> PrtLayer {
        natural_layer_in_parent(self.info())
    }
}
