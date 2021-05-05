use crate::visual::gui::layer::{GblTf, LclLayer, PrtLayer, PrtTf};
use crate::visual::gui::ui::Ui;
use crate::visual::types::{GblZ, LclSz, PrtZ, Pt2D};
use eyre::Result;
use num_traits::Zero;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum SzOpt {
    Wrap,
    Fill,
    Exact,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Grav {
    Begin,
    Center,
    End,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Hint {
    pub opt: (SzOpt, SzOpt),
    pub grav: (Grav, Grav),
    pub min: Option<LclSz>,
    pub max: Option<LclSz>,
    pub req: Option<LclSz>,
}

impl Default for Hint {
    fn default() -> Self {
        Self {
            opt: (SzOpt::Wrap, SzOpt::Wrap),
            grav: (Grav::Begin, Grav::Begin),
            min: None,
            max: None,
            req: None,
        }
    }
}

#[allow(dead_code)]
impl Hint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min(self, sz: LclSz) -> Self {
        Self { min: Some(sz), ..self }
    }

    pub fn max(self, sz: LclSz) -> Self {
        Self { max: Some(sz), ..self }
    }

    pub fn req(self, sz: LclSz) -> Self {
        Self { req: Some(sz), ..self }
    }

    pub fn make_exact(sz: LclSz) -> Self {
        Self { opt: (SzOpt::Exact, SzOpt::Exact), req: Some(sz), ..Default::default() }
    }

    pub fn opt(self, opt: (SzOpt, SzOpt)) -> Self {
        Self { opt, ..self }
    }

    pub fn opt_wh(self, opt: SzOpt) -> Self {
        Self { opt: (opt, opt), ..self }
    }
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct LayoutInfo {
    pub gtf: GblTf,
    pub ptf: PrtTf,
    pub hint: Hint,
}

impl LayoutInfo {
    pub fn zero() -> Self {
        LayoutInfo {
            gtf: GblTf::new(Pt2D::zero(), GblZ::zero()),
            ptf: PrtTf::new(Pt2D::zero(), PrtZ::zero()),
            hint: Hint::default(),
        }
    }

    pub fn hint(self, hint: Hint) -> Self {
        LayoutInfo { hint, ..self }
    }
}

pub trait Layout: Copy {
    fn info(&self) -> &LayoutInfo;
    fn child<UiF, L, ChildL>(
        &mut self,
        ui: &mut Ui<'_, L>,
        hint: &Hint,
        child_id: &str,
        f: UiF,
    ) -> Result<LclLayer>
    where
        L: Layout,
        ChildL: Layout,
        UiF: FnMut(&mut Ui<'_, L>, LayoutInfo) -> Result<ChildL>;
    fn child_layer<L: Layout>(&mut self, ui: &mut Ui<'_, L>, hint: &Hint) -> Result<LclLayer>;
    fn compute_layer(&mut self) -> PrtLayer;
}
