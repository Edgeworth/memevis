use ahash::HashMap;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use winit::window::CursorIcon;

use crate::visual::gui::layer::LclLayer;
use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::layout::{LayoutInfo, LayoutStrategy};
use crate::visual::gui::layouts::util::compute_child_info;
use crate::visual::gui::ui::Ui;
use crate::visual::types::{lrt, lz, LclPt, LclRt, LclSz, LclZ, ZOrder};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
enum ResizeDir {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Left,
    Top,
    Right,
    Bottom,
    Move,
}

const Z_OFF: LclZ = ZOrder::new(1000);
const HOTSPOT_DP: f64 = 18.0;
const RESIZE_INSET: LclSz = LclSz::new(-16.0, -16.0);

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    l: LclLayer,
    dir: ResizeDir,
    mouse_st: LclPt,
    rt_st: LclRt,
}

impl WindowState {
    fn new(l: LclLayer) -> Self {
        Self { l, dir: ResizeDir::Move, mouse_st: LclPt::zero(), rt_st: LclRt::zero() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizeState {
    wins: HashMap<String, WindowState>,
    top_z: LclZ,
}

impl Default for ResizeState {
    fn default() -> Self {
        Self { wins: HashMap::default(), top_z: lz(0) }
    }
}

#[derive(Debug, Copy, Clone)]
struct ResizeInfo {
    delta_rt: LclRt,
    cursor: CursorIcon,
}

#[derive(Debug, Copy, Clone)]
pub struct ResizeLayout {
    info: LayoutInfo,
    loc: LclPt,
}

impl ResizeLayout {
    #[must_use]
    pub fn new(info: LayoutInfo) -> Self {
        Self { info, loc: LclPt::zero() }
    }

    #[allow(clippy::unused_self)]
    fn hitbox(&self, w: &WindowState) -> LclLayer {
        w.l.inset(RESIZE_INSET)
    }

    #[allow(clippy::unused_self)]
    fn state<'a>(&self, ui: &'a mut Ui<'_>) -> &'a mut ResizeState {
        let id = ui.id().to_owned();
        &mut ui.mem_mut().wid(&id).pos
    }

    fn next_z(&self, ui: &mut Ui<'_>) -> LclZ {
        self.state(ui).top_z += Z_OFF;
        self.state(ui).top_z
    }

    fn handle_click(&self, ui: &mut Ui<'_>, w: &mut WindowState) {
        let ltf = self.info.gtf.inv();
        let mouse_st = ltf.pt(ui.io().mouse_pressed_pt);
        w.dir = resize_dir(&self.hitbox(w), mouse_st);
        w.mouse_st = mouse_st;
        w.rt_st = w.l.r;
        w.l.z = self.next_z(ui); // Move to front.
    }

    fn interact(&self, ui: &mut Ui<'_>, child_id: &str, mut w: WindowState) -> WindowState {
        let ltf = self.info.gtf.inv();
        let hitbox = self.hitbox(&w);
        if ui.pressed(child_id, hitbox) {
            if ui.io().mouse_just_captured {
                self.handle_click(ui, &mut w);
            }
            let mouse_dt = (ltf.pt(ui.io().mouse_pt) - w.mouse_st).to_sz();
            let ResizeInfo { delta_rt, cursor } = get_resize_info(mouse_dt, w.dir, true);
            w.l.r = w.rt_st + delta_rt;
            ui.paint_mut().set_cursor(cursor);
        } else if ui.hovered(child_id, hitbox) {
            let ResizeInfo { cursor, .. } = get_resize_info(
                LclSz::zero(),
                resize_dir(&hitbox, ltf.pt(ui.io().mouse_pt)),
                false,
            );
            ui.paint_mut().set_cursor(cursor);
        }
        w
    }
}

impl LayoutStrategy for ResizeLayout {
    fn info(&self) -> &LayoutInfo {
        &self.info
    }

    fn child_info(&mut self, ui: &mut Ui<'_>, hint: &Hint, child_id: &str) -> LayoutInfo {
        let mut w = self.state(ui).wins.get(child_id).copied();
        let info = if let Some(ref mut w) = w {
            *w = self.interact(ui, child_id, *w);
            self.state(ui).wins.insert(child_id.to_owned(), *w);

            compute_child_info(
                self.info(),
                w.l.r.tl(),
                w.l.z,
                &Hint { req: Some(w.l.r.sz()), ..*hint },
            )
        } else {
            compute_child_info(self.info(), self.loc, self.next_z(ui), hint)
        };

        info
    }

    fn place_layer(&mut self, ui: &mut Ui<'_>, l: &LclLayer, child_id: &str) {
        // Update saved layer.
        self.state(ui).wins.entry(child_id.to_owned()).or_insert_with(|| WindowState::new(*l)).l =
            *l;
        self.loc.y = l.r.b();
    }
}

fn get_resize_info(d: LclSz, resize_type: ResizeDir, is_captured: bool) -> ResizeInfo {
    match resize_type {
        ResizeDir::TopLeft => {
            ResizeInfo { delta_rt: lrt(d.w, d.h, -d.w, -d.h), cursor: CursorIcon::NwResize }
        }
        ResizeDir::TopRight => {
            ResizeInfo { delta_rt: lrt(0, d.h, d.w, -d.h), cursor: CursorIcon::NeResize }
        }
        ResizeDir::BottomLeft => {
            ResizeInfo { delta_rt: lrt(d.w, 0, -d.w, d.h), cursor: CursorIcon::SwResize }
        }
        ResizeDir::BottomRight => {
            ResizeInfo { delta_rt: lrt(0, 0, d.w, d.h), cursor: CursorIcon::SeResize }
        }
        ResizeDir::Left => {
            ResizeInfo { delta_rt: lrt(d.w, 0, -d.w, 0), cursor: CursorIcon::WResize }
        }
        ResizeDir::Top => {
            ResizeInfo { delta_rt: lrt(0, d.h, 0, -d.h), cursor: CursorIcon::NResize }
        }
        ResizeDir::Right => ResizeInfo { delta_rt: lrt(0, 0, d.w, 0), cursor: CursorIcon::EResize },
        ResizeDir::Bottom => {
            ResizeInfo { delta_rt: lrt(0, 0, 0, d.h), cursor: CursorIcon::SResize }
        }
        ResizeDir::Move => ResizeInfo {
            delta_rt: lrt(d.w, d.h, 0, 0),
            cursor: if is_captured { CursorIcon::Grabbing } else { CursorIcon::Grab },
        },
    }
}

fn resize_dir(l: &LclLayer, mouse: LclPt) -> ResizeDir {
    #[derive(Debug)]
    struct HotspotMatch {
        l: bool,
        t: bool,
        r: bool,
        b: bool,
    }
    let m = HotspotMatch {
        l: mouse.x < l.r.x + HOTSPOT_DP,
        t: mouse.y < l.r.y + HOTSPOT_DP,
        r: mouse.x > l.r.r() - HOTSPOT_DP,
        b: mouse.y > l.r.b() - HOTSPOT_DP,
    };
    match m {
        HotspotMatch { t: true, l: true, .. } => ResizeDir::TopLeft,
        HotspotMatch { t: true, r: true, .. } => ResizeDir::TopRight,
        HotspotMatch { b: true, l: true, .. } => ResizeDir::BottomLeft,
        HotspotMatch { b: true, r: true, .. } => ResizeDir::BottomRight,
        HotspotMatch { l: true, t: false, r: false, b: false } => ResizeDir::Left,
        HotspotMatch { l: false, t: true, r: false, b: false } => ResizeDir::Top,
        HotspotMatch { l: false, t: false, r: true, b: false } => ResizeDir::Right,
        HotspotMatch { l: false, t: false, r: false, b: true } => ResizeDir::Bottom,
        _ => ResizeDir::Move,
    }
}
