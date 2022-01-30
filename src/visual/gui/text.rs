use eyre::Result;

use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::layouts::hint::Grav;
use crate::visual::gui::ui::Ui;
use crate::visual::types::{lpt, lz, LclPt, LclRt};

#[derive(Debug, PartialEq, Clone)]
pub struct Frag {
    pub text: String,
    pub grav: (Grav, Grav),
    pub sz: f64,
    pub p: LclPt,
}

impl Frag {
    #[must_use]
    pub fn new(text: &str, sz: f64, p: LclPt) -> Self {
        Self { text: text.to_owned(), grav: (Grav::Begin, Grav::Begin), sz, p }
    }

    #[must_use]
    pub fn pt(self, p: LclPt) -> Self {
        Self { p, ..self }
    }

    #[must_use]
    pub fn hgrav(mut self, g: Grav) -> Self {
        self.grav.0 = g;
        self
    }

    #[must_use]
    pub fn vgrav(mut self, g: Grav) -> Self {
        self.grav.1 = g;
        self
    }

    pub fn layout(&self, ui: &mut Ui<'_>) -> Result<LclLayer> {
        let sz = ui.text_sz(self)?;
        let xoff = match self.grav.0 {
            Grav::Begin => 0.0,
            Grav::Center => sz.w / 2.0,
            Grav::End => sz.w,
        };
        let yoff = match self.grav.1 {
            Grav::Begin => 0.0,
            Grav::Center => sz.h / 2.0,
            Grav::End => sz.h,
        };
        Ok(LclLayer::new(LclRt::ptsz(self.p - lpt(xoff, yoff), sz), lz(0)))
    }
}
