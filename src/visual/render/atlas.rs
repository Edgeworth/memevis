use std::cmp::max;

use eyre::{Result, eyre};
use rgb::RGBA8;

use crate::visual::Rt2D;
use crate::visual::render::texture::{TexId, TexStore, TextureLayer};
use crate::visual::types::{TexPt, TexRt, TexSz, TexUvRect, trt};

pub type AtlasHandle = i32;

#[derive(Debug)]
pub struct Atlas {
    p: Packer,
    tex: TexId,
    hnds: Vec<TexRt>,
    used_area: u32,
}

static DEFAULT_SIZE: TexSz = TexSz::new(2048, 2048);
static PAD: TexSz = TexSz::new(1, 1);

impl Atlas {
    pub fn new(ts: &mut TexStore) -> Self {
        Self {
            p: Packer::new(DEFAULT_SIZE),
            tex: ts.insert(DEFAULT_SIZE),
            hnds: Vec::new(),
            used_area: 0,
        }
    }

    pub fn alloc(&mut self, sz: TexSz) -> Result<AtlasHandle> {
        if sz.area() == 0 {
            return Err(eyre!("can't pack empty size"));
        }
        let sz = sz + 2 * PAD;
        let rect = self.p.pack(sz).ok_or_else(|| {
            eyre!(
                "atlas full at {:.2}% capacity",
                self.used_area as f64 / self.p.sz.area() as f64 * 100.0
            )
        })?;
        self.used_area += sz.area();
        self.hnds.push(rect);
        Ok((self.hnds.len() - 1) as AtlasHandle)
    }

    pub fn write_px(&self, ts: &mut TexStore, hnd: AtlasHandle, p: TexPt, c: RGBA8) {
        ts.get_mut(self.tex).write(self.hnds[hnd as usize].tl() + p + PAD, c);
    }

    #[must_use]
    pub fn get_layer(&self, tex: AtlasHandle) -> TextureLayer {
        let r = self.hnds[tex as usize].to_f64();
        let sz = self.p.sz.to_f64();
        let uv = TexUvRect::new(r.x / sz.w, r.y / sz.h, r.w / sz.w, r.h / sz.h);
        let uv = uv.inset(PAD.to_f64() / sz);
        TextureLayer::new(Rt2D::default(), uv, self.tex)
    }
}

#[derive(Debug)]
struct Packer {
    sz: TexSz,
    cur: TexPt,
    maxh: u32,
}

impl Packer {
    fn new(s: TexSz) -> Self {
        Self { sz: s, cur: (0, 0).into(), maxh: 0 }
    }

    fn pack(&mut self, sz: TexSz) -> Option<TexRt> {
        if self.cur.x + sz.w <= self.sz.w && self.cur.y + sz.h <= self.sz.h {
            // Pack horizontally
            let r = TexRt::ptsz(self.cur, sz);
            self.cur.x += sz.w;
            self.maxh = max(self.maxh, sz.h);
            Some(r)
        } else if self.cur.y + self.maxh + sz.h <= self.sz.h {
            // Pack vertically
            self.cur.y += self.maxh;
            let r = trt(0, self.cur.y, sz.w, sz.h);
            self.cur.x = sz.w;
            self.maxh = sz.h;
            Some(r)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packer() {
        let mut p = Packer::new((5, 6).into());
        assert_eq!(Some(trt(0, 0, 1, 1)), p.pack((1, 1).into()));
        assert_eq!(Some(trt(1, 0, 1, 1)), p.pack((1, 1).into()));
        assert_eq!(Some(trt(2, 0, 2, 2)), p.pack((2, 2).into()));
        assert_eq!(Some(trt(0, 2, 2, 2)), p.pack((2, 2).into()));
        assert_eq!(Some(trt(2, 2, 1, 3)), p.pack((1, 3).into()));
        assert_eq!(Some(trt(3, 2, 2, 1)), p.pack((2, 1).into()));
        assert_eq!(Some(trt(0, 5, 1, 1)), p.pack((1, 1).into()));
        assert_eq!(None, p.pack((1, 2).into()));
        assert_eq!(None, p.pack((5, 1).into()));
    }
}
