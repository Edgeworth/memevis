use crate::visual::gui::layer::GblLayer;
use crate::visual::gui::layouts::layout::{Hint, Layout, LayoutInfo};
use crate::visual::gui::layouts::resize_layout::{ResizeLayout, ResizeState};
use crate::visual::gui::ui::Ui;
use crate::visual::io::Io;
use crate::visual::render::font::Font;
use crate::visual::render::painter::Painter;
use crate::visual::types::{GblSz, LclSz};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type FontId = u32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WidgetMemory {
    pub pos: ResizeState,
    pub graph: GraphState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub wid: HashMap<String, WidgetMemory>,
    pub debug: bool,
    #[serde(skip)]
    path: PathBuf,
}

impl Memory {
    pub fn from_path(p: impl AsRef<Path>) -> Self {
        let mut f = File::open(&p);
        if f.is_err() {
            Self::new(&p).write_json().expect("could not write json");
            f = File::open(&p);
        }
        let f = f.expect("could not open json file");
        Self {
            path: p.as_ref().to_path_buf(),
            ..serde_json::from_reader(f).expect("json invalid")
        }
    }

    pub fn new(p: impl AsRef<Path>) -> Self {
        Self {
            wid: HashMap::new(),
            debug: false,
            path: p.as_ref().to_path_buf(),
        }
    }

    pub fn wid(&mut self, id: &str) -> &mut WidgetMemory {
        self.wid.entry(id.to_owned()).or_default()
    }

    pub fn exit(&mut self) -> Result<()> {
        self.write_json()
    }

    fn write_json(&self) -> Result<()> {
        Ok(serde_json::to_writer_pretty(
            File::create(&self.path)?,
            self,
        )?)
    }
}

pub struct Vis {
    pub p: Painter,
    pub io: Io,
    pub mem: Memory,
    f: HashMap<FontId, Font>,
}

impl Vis {
    pub fn new(dp_to_px: f64, scr_sz: GblSz) -> Result<Self> {
        const VIS_PATH: &str = "vis.json";
        let mut f = HashMap::new();
        f.insert(0, Font::new()?);
        let io = Io::new(dp_to_px, scr_sz);
        let mem = Memory::from_path(VIS_PATH);
        Ok(Self {
            p: Painter::new(),
            io,
            mem,
            f,
        })
    }

    pub fn begin(&mut self) -> Result<Ui<'_, impl Layout>> {
        self.p.begin();
        self.io.begin();
        let scr_sz: LclSz = self.io.scr_sz.coerce();
        Ok(Ui::new(
            self,
            ResizeLayout::new(LayoutInfo::zero().hint(Hint::make_exact(scr_sz))),
            "top",
        ))
    }

    pub fn end(&mut self) {
        self.io.end();
    }

    pub fn exit(&mut self) -> Result<()> {
        self.mem.exit()
    }

    pub fn font(&mut self, id: FontId) -> &mut Font {
        self.f.get_mut(&id).unwrap()
    }

    pub fn layout_text(&mut self, text: &str, dp: f64) -> Result<GblSz> {
        self.f
            .get_mut(&0)
            .unwrap()
            .layout(self.io.dp_to_px, text, dp)
    }

    pub fn draw_text(&mut self, text: &str, dp: f64, l: &GblLayer) -> Result<()> {
        self.f
            .get_mut(&0)
            .unwrap()
            .draw(&mut self.p, self.io.dp_to_px, text, dp, l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn memory_serialization() {
        let d = tempdir().unwrap();
        let p = d.path().join("vis.json");
        let mut m = Memory::from_path(&p);
        m.wid.insert("test".to_owned(), WidgetMemory::default());
        m.exit().unwrap();
        let m = Memory::from_path(&p);
        assert!(m.wid.contains_key("test"));
    }
}
