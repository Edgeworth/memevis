use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::ui::Ui;
use eyre::Result;

pub struct Resp {
    pub l: LclLayer,
}

pub trait Widget {
    fn ui(&mut self, ui: &mut Ui) -> Result<Resp>;
    fn lcl_id(&self, ui: &Ui) -> String;
}

pub fn combine_ids(ids: &[&str]) -> String {
    ids.iter().fold(String::new(), |cur, next| cur + "##" + next)
}
