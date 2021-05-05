use crate::visual::gui::layouts::layout::{Hint, Layout, SzOpt};
use crate::visual::gui::layouts::vert_layout::VertLayout;
use crate::visual::gui::ui::Ui;
use crate::visual::gui::widgets::label::Label;
use crate::visual::gui::widgets::widget::{combine_ids, Resp, Widget};
use eyre::Result;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Button<L: Layout, F: FnOnce(&mut Ui<'_, L>)> {
    label: Label,
    cb: Option<F>,
    _u: PhantomData<L>,
}

impl<L: Layout, F: FnOnce(&mut Ui<'_, L>)> Button<L, F> {
    pub fn new(text: &str, cb: F) -> Self {
        Self { label: Label::new(text), cb: Some(cb), _u: PhantomData }
    }
}

impl<L: Layout, F: FnOnce(&mut Ui<'_, L>)> Widget<L> for Button<L, F> {
    fn ui(&mut self, ui: &mut Ui<'_, L>) -> Result<Resp> {
        let id = ui.wid(self);
        let l = ui.child(&Hint::new().opt_wh(SzOpt::Wrap), &id, VertLayout::new, |ui| {
            self.label.ui(ui)?;
            Ok(())
        })?;

        let col =
            if ui.hovered(&id, l) { ui.s.light_col.alpha(0.3) } else { ui.s.light_col.alpha(0.2) };
        let col = if ui.pressed(&id, l) { ui.s.light_col.alpha(0.1) } else { col };
        if ui.clicked(&id, l) {
            if let Some(f) = self.cb.take() {
                f(ui)
            }
        }

        let _scope = ui.push(ui.pctx().col(col));
        ui.fill_rrt(l.r, 4.0);

        Ok(Resp { l })
    }

    fn lcl_id(&self, ui: &Ui<'_, L>) -> String {
        combine_ids(&["button", &self.label.lcl_id(ui)])
    }
}
