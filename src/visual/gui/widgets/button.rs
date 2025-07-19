use eyre::Result;

use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::layout::Layout;
use crate::visual::gui::layouts::vert_layout::VertLayout;
use crate::visual::gui::ui::Ui;
use crate::visual::gui::widgets::label::Label;
use crate::visual::gui::widgets::widget::{Resp, Widget, combine_ids};

#[derive(Debug)]
pub struct Button<F: FnOnce(&mut Ui<'_>)> {
    label: Label,
    cb: Option<F>,
}

impl<F: FnOnce(&mut Ui<'_>)> Button<F> {
    pub fn new(text: &str, cb: F) -> Self {
        Self { label: Label::new(text), cb: Some(cb) }
    }
}

impl<F: FnOnce(&mut Ui<'_>)> Widget for Button<F> {
    fn ui(&mut self, ui: &mut Ui<'_>) -> Result<Resp> {
        let id = ui.wid(self);
        let l = ui.child(
            &Hint::new(),
            &id,
            |info| Layout::new(VertLayout::new(info)),
            |ui| {
                self.label.ui(ui)?;
                Ok(())
            },
        )?;

        let col = if ui.hovered(&id, l) {
            ui.s.light_col.with_alpha(0.3)
        } else {
            ui.s.light_col.with_alpha(0.2)
        };
        let col = if ui.pressed(&id, l) { ui.s.light_col.with_alpha(0.1) } else { col };
        if ui.clicked(&id, l)
            && let Some(f) = self.cb.take()
        {
            f(ui);
        }

        let scope = ui.push();
        scope.col(col);
        ui.fill_rrt(l.r, 4.0);

        Ok(Resp { l })
    }

    fn lcl_id(&self, ui: &Ui<'_>) -> String {
        combine_ids(&["button", &self.label.lcl_id(ui)])
    }
}
