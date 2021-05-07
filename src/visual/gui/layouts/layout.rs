use crate::visual::gui::layer::{GblTf, LclLayer, PrtLayer, PrtTf};
use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::util::natural_layer_in_parent;
use crate::visual::gui::ui::Ui;
use crate::visual::types::{GblZ, PrtZ, Pt2D};
use dyn_clone::DynClone;
use eyre::Result;
use num_traits::Zero;

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

pub trait LayoutStrategy: DynClone {
    // Gets the layout info for this strategy.
    fn info(&self) -> &LayoutInfo;

    // Compute the layout info for placing a child.
    fn child_info(&mut self, ui: &mut Ui, hint: &Hint, child_id: &str) -> LayoutInfo;

    fn place_layer(&mut self, ui: &mut Ui, l: &LclLayer, child_id: &str);
}

dyn_clone::clone_trait_object!(LayoutStrategy);

#[derive(Clone)]
pub struct Layout {
    strat: Box<dyn LayoutStrategy>,
}

impl Layout {
    pub fn new(strat: impl LayoutStrategy + 'static) -> Self {
        Self { strat: Box::new(strat) }
    }

    pub fn info(&self) -> &LayoutInfo {
        self.strat.info()
    }

    pub fn child(
        &mut self,
        ui: &mut Ui,
        hint: &Hint,
        child_id: &str,
        mut f: impl FnMut(&mut Ui, LayoutInfo) -> Result<Layout>,
    ) -> Result<LclLayer> {
        let info = self.strat.child_info(ui, hint, child_id);
        let mut layout = f(ui, info)?;
        let l: LclLayer = layout.compute_layer().coerce(); // Parent is local here.
        self.strat.place_layer(ui, &l, child_id);
        Ok(l)
    }

    pub fn child_layer(&mut self, ui: &mut Ui, hint: &Hint) -> LclLayer {
        // Just pass nothing for the child id since we won't use it. Some
        // strategies might not support this though.
        let info = self.strat.child_info(ui, hint, "");
        let l = natural_layer_in_parent(&info).coerce();
        self.strat.place_layer(ui, &l, "");
        l
    }

    pub fn compute_layer(&mut self) -> PrtLayer {
        natural_layer_in_parent(self.strat.info())
    }
}
