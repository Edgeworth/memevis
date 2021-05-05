use crate::visual::types::Rt2D;

pub mod colors;
pub mod gui;
pub mod io;
pub mod render;
pub mod types;
pub mod vis;

impl<U> From<Rt2D<u32, U>> for glium::Rect {
    fn from(r: Rt2D<u32, U>) -> Self {
        glium::Rect { left: r.x, bottom: r.y, width: r.w, height: r.h }
    }
}
