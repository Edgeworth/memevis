use eyre::Result;

use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::ui::Ui;

pub struct Resp {
    pub l: LclLayer,
}

pub trait Widget {
    fn ui(&mut self, ui: &mut Ui<'_>) -> Result<Resp>;
    fn lcl_id(&self, ui: &Ui<'_>) -> String;
}

#[must_use]
pub fn combine_ids(ids: &[&str]) -> String {
    ids.iter().fold(String::new(), |cur, next| cur + "##" + next)
}
