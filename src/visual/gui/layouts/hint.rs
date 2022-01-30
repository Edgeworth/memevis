use crate::visual::types::LclSz;

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
        Self::new()
    }
}

#[allow(dead_code)]
impl Hint {
    #[must_use]
    pub fn new() -> Self {
        Self {
            opt: (SzOpt::Wrap, SzOpt::Wrap),
            grav: (Grav::Begin, Grav::Begin),
            min: None,
            max: None,
            req: None,
        }
    }

    #[must_use]
    pub fn min(self, sz: LclSz) -> Self {
        Self { min: Some(sz), ..self }
    }

    #[must_use]
    pub fn max(self, sz: LclSz) -> Self {
        Self { max: Some(sz), ..self }
    }

    #[must_use]
    pub fn req(self, sz: LclSz) -> Self {
        Self { req: Some(sz), ..self }
    }

    #[must_use]
    pub fn make_exact(sz: LclSz) -> Self {
        Self { opt: (SzOpt::Exact, SzOpt::Exact), req: Some(sz), ..Default::default() }
    }

    #[must_use]
    pub fn opt(self, opt: (SzOpt, SzOpt)) -> Self {
        Self { opt, ..self }
    }

    #[must_use]
    pub fn opt_wh(self, opt: SzOpt) -> Self {
        Self { opt: (opt, opt), ..self }
    }
}
