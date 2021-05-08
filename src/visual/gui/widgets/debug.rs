use eyre::Result;
use num_traits::Zero;

use crate::visual::gui::ui::Ui;
use crate::visual::types::ZOrder;

pub fn debug_pane(ui: &mut Ui<'_>) -> Result<()> {
    ui.button("[debug] show layout", |ui| ui.mem_mut().debug = !ui.mem().debug)?;
    let ft = ui.io().prev_end_frame_time - ui.io().prev_begin_frame_time;
    let rt = (ui.io().begin_frame_time - ui.io().prev_begin_frame_time).as_secs_f32();
    ui.label(&format!("[debug] frame ms: {:.2}", ft.as_secs_f32() * 1000.0))?;
    ui.label(&format!("[debug] render ms: {:.2}", rt * 1000.0))?;
    ui.label(&format!("[debug] fps: {:.2}", 1.0 / rt))?;
    ui.label(&format!("[debug] z-order: {}", ui.info().gtf.z(ZOrder::zero())))?;
    Ok(())
}
