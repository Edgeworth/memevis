use crate::visual::gui::layouts::layout::{Hint, Layout};
use crate::visual::gui::text::Frag;
use crate::visual::gui::ui::Ui;
use crate::visual::gui::widgets::widget::{Resp, Widget};
use crate::visual::types::LclPt;
use eyre::Result;
use num_traits::Zero;

#[derive(Debug)]
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Label {
        Self { text: text.to_owned() }
    }
}

impl<L: Layout> Widget<L> for Label {
    fn ui(&mut self, ui: &mut Ui<'_, L>) -> Result<Resp> {
        let f = Frag::new(&self.text, ui.s.font_sz, LclPt::zero());
        let sz = ui.text_sz(&f)?;
        let l = ui.child_layer(&Hint::make_exact(sz))?;
        ui.text(&f.pt(l.r.tl()))?;
        Ok(Resp { l })
    }

    fn lcl_id(&self, _ui: &Ui<'_, L>) -> String {
        self.text.clone()
    }
}