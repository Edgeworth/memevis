use num_traits::Zero;

use crate::visual::gui::layer::{LclLayer, PrtLayer, PrtTf};
use crate::visual::gui::layouts::hint::{Hint, SzOpt};
use crate::visual::gui::layouts::layout::LayoutInfo;
use crate::visual::types::{lsz, LclPt, LclRt, LclSz, LclZ};

fn clamp(mut sz: LclSz, min: Option<LclSz>, max: Option<LclSz>) -> LclSz {
    if let Some(min) = min {
        sz = min.max(&sz);
    }
    if let Some(max) = max {
        sz = max.min(&sz);
    }
    sz
}

fn select_sz(
    min: Option<LclSz>,
    max: Option<LclSz>,
    parent_req: Option<LclSz>,
    child_req: Option<LclSz>,
    opt: SzOpt,
) -> Option<LclSz> {
    let sz = match opt {
        SzOpt::Wrap => min.or(child_req).or(parent_req).or(max),
        SzOpt::Fill => parent_req.or(max).or(child_req).or(min),
        SzOpt::Exact => child_req.or(min).or(max),
    };
    sz.map(|sz| clamp(sz, min, max))
}

fn select_sz_2d(
    min: Option<LclSz>,
    max: Option<LclSz>,
    parent_req: Option<LclSz>,
    child_req: Option<LclSz>,
    opt: (SzOpt, SzOpt),
) -> Option<LclSz> {
    let w_sz = select_sz(min, max, parent_req, child_req, opt.0);
    let h_sz = select_sz(min, max, parent_req, child_req, opt.1);
    if let Some(w_sz) = w_sz {
        if let Some(h_sz) = h_sz {
            return Some(lsz(w_sz.w, h_sz.h));
        }
    }
    None
}

pub(super) fn compute_child_info(
    info: &LayoutInfo,
    offset: LclPt,
    z: LclZ,
    child: &Hint,
) -> LayoutInfo {
    let ptf = PrtTf::new(offset.coerce(), z.coerce());
    let gtf = ptf.concat(&info.gtf.coerce());

    let h = info.hint;
    // Get maximum rect inside parent and map it to child coord space.
    let max = h.max.map(|v| LclRt::ptsz(offset, v - offset.to_sz()));
    let max = max.map(|v| ptf.inv().rt(v.coerce()).sz());

    // Choose smallest out of child max and max from parent.
    let max = child.max.iter().chain(max.iter()).copied().reduce(|a, b| a.min(&b));

    // Compute requested size
    let parent_req = h.req.map(|v| ptf.inv().sz((v - offset.to_sz()).coerce()));
    let req = select_sz_2d(child.min, max, parent_req, child.req, child.opt);
    LayoutInfo {
        ptf,
        gtf,
        hint: Hint { opt: child.opt, grav: child.grav, min: child.min, max, req },
    }
}

fn natural_sz(h: &Hint) -> Option<LclSz> {
    select_sz_2d(h.min, h.max, None, h.req, h.opt)
}

pub(super) fn natural_layer(h: &Hint) -> LclLayer {
    LclLayer::from_sz(natural_sz(h).unwrap_or_else(LclSz::zero))
}

pub(super) fn natural_layer_in_parent(info: &LayoutInfo) -> PrtLayer {
    info.ptf.layer(natural_layer(&info.hint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::types::{lpt, lz};

    const OPTS: [SzOpt; 3] = [SzOpt::Wrap, SzOpt::Fill, SzOpt::Exact];

    #[test]
    fn test_child_info_fill() {
        let child_info = compute_child_info(
            &LayoutInfo::zero().hint(Hint::new().opt_wh(SzOpt::Fill).req(lsz(200, 200))),
            lpt(10, 15),
            lz(1),
            &Hint::new().opt_wh(SzOpt::Fill).req(lsz(100, 100)),
        );
        assert_eq!(natural_layer(&child_info.hint), LclLayer::from_sz(lsz(190, 185)));

        // FILL child with a requested size to a parent with an unknown size should use requested size.
        let child_info = compute_child_info(
            &LayoutInfo::zero().hint(Hint::new().opt_wh(SzOpt::Fill).min(lsz(50, 50))),
            lpt(10, 15),
            lz(1),
            &Hint::new().opt_wh(SzOpt::Fill).req(lsz(100, 100)),
        );
        assert_eq!(natural_layer(&child_info.hint), LclLayer::from_sz(lsz(100, 100)));
    }

    #[test]
    fn test_child_info_wrap() {
        let child_info = compute_child_info(
            &LayoutInfo::zero().hint(Hint::new().opt_wh(SzOpt::Wrap).req(lsz(200, 200))),
            lpt(10, 15),
            lz(1),
            &Hint::new().opt_wh(SzOpt::Wrap).req(lsz(300, 300)).min(lsz(100, 100)),
        );
        assert_eq!(natural_layer(&child_info.hint), LclLayer::from_sz(lsz(100, 100)));
    }

    #[test]
    fn test_child_info_exact() {
        // Exact should not use parent request if there is no info - useful for windows.
        let child_info = compute_child_info(
            &LayoutInfo::zero().hint(Hint::new().opt_wh(SzOpt::Exact).req(lsz(1000, 1000))),
            lpt(10, 15),
            lz(1),
            &Hint::new().opt_wh(SzOpt::Exact),
        );
        assert_eq!(natural_layer(&child_info.hint), LclLayer::from_sz(lsz(0, 0)));

        // Exact should fall back to wrap behaviour if no requested size - useful for windows.
        let child_info = compute_child_info(
            &LayoutInfo::zero().hint(Hint::new().opt_wh(SzOpt::Exact).req(lsz(1000, 1000))),
            lpt(10, 15),
            lz(1),
            &Hint::new().opt_wh(SzOpt::Exact).min(lsz(100, 100)),
        );
        assert_eq!(natural_layer(&child_info.hint), LclLayer::from_sz(lsz(100, 100)));
    }

    #[test]
    fn test_natural_layer() {
        // Exact with a requested size should be exact.
        assert_eq!(natural_layer(&Hint::make_exact(lsz(100, 50))), LclLayer::from_sz(lsz(100, 50)));

        // Exact should fall back to wrap behaviour if no requested size.
        assert_eq!(
            natural_layer(&Hint::new().opt_wh(SzOpt::Wrap).min(lsz(200, 200))),
            LclLayer::from_sz(lsz(200, 200))
        );

        // Wrap with a requested size, but no min size should be requested.
        assert_eq!(
            natural_layer(&Hint::new().opt_wh(SzOpt::Wrap).req(lsz(100, 50))),
            LclLayer::from_sz(lsz(100, 50))
        );

        // Min should override requested size for wrap.
        assert_eq!(
            natural_layer(&Hint::new().opt_wh(SzOpt::Wrap).req(lsz(300, 350)).min(lsz(200, 200))),
            LclLayer::from_sz(lsz(200, 200))
        );

        let sz = lsz(100, 5);
        for opt_w in OPTS.iter().copied() {
            // Test combinations of selecting only one size.
            for opt_h in OPTS.iter().copied() {
                let opt = (opt_w, opt_h);
                assert_eq!(natural_layer(&Hint::new().opt(opt).min(sz)), LclLayer::from_sz(sz));
                assert_eq!(natural_layer(&Hint::new().opt(opt).max(sz)), LclLayer::from_sz(sz));
                assert_eq!(natural_layer(&Hint::new().opt(opt).req(sz)), LclLayer::from_sz(sz));

                assert_eq!(
                    natural_layer(&Hint::new().opt(opt).min(sz).max(sz)),
                    LclLayer::from_sz(sz)
                );
                assert_eq!(
                    natural_layer(&Hint::new().opt(opt).min(sz).req(sz)),
                    LclLayer::from_sz(sz)
                );
                assert_eq!(
                    natural_layer(&Hint::new().opt(opt).max(sz).req(sz)),
                    LclLayer::from_sz(sz)
                );

                assert_eq!(
                    natural_layer(&Hint::new().opt(opt).min(sz).max(sz).req(sz)),
                    LclLayer::from_sz(sz)
                );
            }
        }
    }
}
