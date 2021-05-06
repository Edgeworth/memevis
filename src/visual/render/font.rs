use crate::any::Any;
use crate::visual::gui::layer::GblLayer;
use crate::visual::render::atlas::AtlasHandle;
use crate::visual::render::painter::{PaintCtx, Painter};
use crate::visual::types::{grt, gsz, tpt, GblRt, GblSz, Rt2D};
use eyre::{eyre, Result};
use harfbuzz_rs::ClusterLevel::MonotoneCharacters;
use harfbuzz_rs::GlyphInfo;
use rgb::RGBA8;
use std::collections::HashMap;
use {freetype as ft, harfbuzz_rs as hb};

static FONT_DATA: &[u8] = include_bytes!("../../assets/OpenSans-Regular.ttf");

pub struct Font {
    _ft: ft::Library,
    render_font: ft::Face,
    layout_font: hb::Owned<hb::Font<'static>>,
    m: HashMap<(u32, u32), RenderedGlyph>,
}

#[derive(Debug)]
struct RenderedGlyph {
    bb: GblRt,
    tex: AtlasHandle,
}

#[derive(Debug)]
struct LayoutGlyph {
    info: GlyphInfo,
    adv: GblSz,
    off: GblSz,
}

#[derive(Debug)]
struct LayoutInfo {
    glyphs: Vec<LayoutGlyph>,
    px_size: u32,
    height_dp: f64,
    line_gap_dp: f64,
}

impl Font {
    pub fn new() -> Result<Self> {
        let freetype = ft::Library::init()?; // TODO: Need to pull this out for multiple fonts.
        let render_font = freetype.new_memory_face(FONT_DATA.to_owned(), 0)?;
        let layout_font = hb::Font::new(hb::Face::from_bytes(FONT_DATA, 0));

        Ok(Self { _ft: freetype, render_font, layout_font, m: HashMap::new() })
    }

    fn ensure_glyph(
        &mut self,
        p: &mut Painter,
        dp_to_px: f64,
        id: u32,
        px: u32,
    ) -> Result<&RenderedGlyph> {
        #[allow(clippy::map_entry)]
        if !self.m.contains_key(&(id, px)) {
            self.render_font.set_pixel_sizes(px, px)?;

            self.render_font.load_glyph(id, ft::face::LoadFlag::RENDER)?;
            let g = self.render_font.glyph();
            let b = g.bitmap();
            let bb = Rt2D::<i32, Any>::new(g.bitmap_left(), g.bitmap_top(), b.width(), b.rows());

            let tex = p.alloc(bb.sz().to_u32().coerce())?;
            for y in 0..bb.h {
                for x in 0..bb.w {
                    let v = b.buffer()[(y * b.pitch() + x) as usize];
                    p.write_px(tex, tpt(x, bb.h - y - 1), RGBA8::new(255, 255, 255, v));
                }
            }

            let vmetrics =
                self.render_font.size_metrics().ok_or_else(|| eyre!("missing font vmetrics"))?;
            let ascender_dp = vmetrics.ascender as f64 / 64.0 / dp_to_px;
            let bb = bb.to_f64() / dp_to_px;
            let bb = grt(bb.x, ascender_dp - bb.y, bb.w, bb.h);
            self.m.insert((id, px), RenderedGlyph { bb, tex });
        }
        Ok(self.m.get(&(id, px)).unwrap())
    }

    fn layout_line(&mut self, dp_to_px: f64, text: &str, dp: f64) -> Result<LayoutInfo> {
        // Take the |dp_size| to be the size of the EM square. The number of pixels
        // of each side will be |px_size|.
        let px_size = (dp * dp_to_px).round() as u32;
        self.layout_font.set_ppem(px_size, px_size);
        self.layout_font.set_scale(px_size as i32 * 64, px_size as i32 * 64);
        self.render_font.set_pixel_sizes(px_size, px_size)?;

        let buf = hb::UnicodeBuffer::new().set_cluster_level(MonotoneCharacters).add_str(text);
        let shape = hb::shape(&self.layout_font, buf, &[]);
        let positions = shape.get_glyph_positions();
        let infos = shape.get_glyph_infos();
        let vmetrics =
            self.render_font.size_metrics().ok_or_else(|| eyre!("missing font vmetrics"))?;
        let height_dp = (vmetrics.ascender - vmetrics.descender) as f64 / 64.0 / dp_to_px;
        let line_gap_dp = vmetrics.height as f64 / 64.0 / dp_to_px;

        let mut layout_info = LayoutInfo {
            glyphs: Vec::with_capacity(positions.len()),
            px_size,
            height_dp,
            line_gap_dp,
        };
        for (pos, info) in positions.iter().zip(infos) {
            let adv = gsz(pos.x_advance, pos.y_advance);
            let adv = adv / 64.0 / dp_to_px;
            let off = gsz(pos.x_offset, pos.y_offset);
            let off = off / 64.0 / dp_to_px;

            layout_info.glyphs.push(LayoutGlyph { info: *info, adv, off });
        }
        Ok(layout_info)
    }

    pub fn layout(&mut self, dp_to_px: f64, text: &str, dp: f64) -> Result<GblSz> {
        let layout = self.layout_line(dp_to_px, text, dp)?;
        let mut sz = gsz(0, layout.height_dp);
        for LayoutGlyph { adv, .. } in layout.glyphs {
            sz.w += adv.w;
        }
        Ok(sz)
    }

    // TODO: use unicode grapheme segmentation
    pub fn draw(
        &mut self,
        p: &mut Painter,
        dp_to_px: f64,
        text: &str,
        dp: f64,
        l: &GblLayer,
    ) -> Result<()> {
        let mut cursor = l.r.tl();
        let layout = self.layout_line(dp_to_px, text, dp)?;
        for LayoutGlyph { info, adv, off } in layout.glyphs {
            if let Ok(info) = self.ensure_glyph(p, dp_to_px, info.codepoint, layout.px_size) {
                let bb = GblRt::ptsz(info.bb.tl() + cursor + off, info.bb.sz());
                let layer = p.get_tex(info.tex).with_rect(bb);
                p.tex(PaintCtx::new().z(l.z.coerce()), layer);
            }
            cursor += adv;
        }
        Ok(())
    }
}
