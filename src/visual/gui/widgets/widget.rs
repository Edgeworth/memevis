use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::layouts::layout::Layout;
use crate::visual::gui::ui::Ui;
use eyre::Result;

pub struct Resp {
    pub l: LclLayer,
}

pub trait Widget<L: Layout> {
    fn ui(&mut self, ui: &mut Ui<'_, L>) -> Result<Resp>;
    fn lcl_id(&self, ui: &Ui<'_, L>) -> String;
}

pub fn combine_ids(ids: &[&str]) -> String {
    ids.iter().fold(String::new(), |cur, next| cur + "##" + next)
}
