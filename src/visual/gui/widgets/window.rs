use crate::visual::gui::layouts::layout::{Hint, Layout, SzOpt};
use crate::visual::gui::layouts::vert_layout::VertLayout;
use crate::visual::gui::ui::Ui;
use crate::visual::gui::widgets::widget::{Resp, Widget};
use crate::visual::types::{lz, LclRt};
use eyre::Result;
use num_traits::Zero;

#[derive(Debug, Clone)]
pub struct Window<F: FnMut(&mut Ui<'_, VertLayout>) -> Result<()>> {
    title: String,
    f: F,
}

impl<F: FnMut(&mut Ui<'_, VertLayout>) -> Result<()>> Window<F> {
    pub fn new(title: &str, f: F) -> Self {
        Self { title: title.to_owned(), f }
    }
}

impl<L: Layout, F: FnMut(&mut Ui<'_, VertLayout>) -> Result<()>> Widget<L> for Window<F> {
    fn ui(&mut self, ui: &mut Ui<'_, L>) -> Result<Resp> {
        let id = ui.wid(self);
        let mut title_r = LclRt::zero();
        let l = ui.child(&Hint::new().opt_wh(SzOpt::Exact), &id, VertLayout::new, |ui| {
            title_r = ui.label(&self.title)?.l.r;
            title_r = ui.info().ptf.rt(title_r).coerce();
            (self.f)(ui)?;
            Ok(())
        })?;

        let scope = ui.push(ui.pctx().z(l.z - lz(1)).col(ui.s.dark_col.alpha(0.95)));
        ui.fill_rrt(l.r, 4.0);
        title_r.w = l.r.w; // Expand to width of window.

        scope.col(ui.s.acc3_col.alpha(0.95));
        ui.stroke_rrt(title_r, 4.0);

        Ok(Resp { l })
    }

    fn lcl_id(&self, _: &Ui<'_, L>) -> String {
        self.title.clone()
    }
}
