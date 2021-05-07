use crate::visual::gui::layer::GblLayer;
use crate::visual::gui::layouts::hint::Hint;
use crate::visual::gui::layouts::layout::{Layout, LayoutInfo};
use crate::visual::gui::layouts::resize_layout::{ResizeLayout, ResizeState};
use crate::visual::gui::ui::Ui;
use crate::visual::io::Io;
use crate::visual::render::font::Font;
use crate::visual::render::painter::Painter;
use crate::visual::types::{GblSz, LclSz};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type FontId = u32;

#[typetag::serde]
pub trait UserData: Any {
    fn get_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WidgetMemory {
    pub pos: ResizeState,
}

#[derive(Serialize, Deserialize)]
pub struct Memory {
    pub wid: HashMap<String, WidgetMemory>,
    pub debug: bool,
    user: HashMap<String, Box<dyn UserData>>,
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
        Self { path: p.as_ref().to_path_buf(), ..serde_json::from_reader(f).expect("json invalid") }
    }

    pub fn new(p: impl AsRef<Path>) -> Self {
        Self {
            wid: HashMap::new(),
            debug: false,
            user: HashMap::new(),
            path: p.as_ref().to_path_buf(),
        }
    }

    pub fn wid(&mut self, id: &str) -> &mut WidgetMemory {
        self.wid.entry(id.to_owned()).or_default()
    }

    pub fn user<T: UserData + Default + 'static>(&mut self, id: &str) -> Result<&mut T> {
        let any =
            self.user.entry(id.to_owned()).or_insert_with(|| Box::new(T::default())).get_any();
        let any =
            any.downcast_mut::<T>().ok_or_else(|| eyre!("user object type does not match"))?;
        Ok(any)
    }

    pub fn exit(&self) -> Result<()> {
        self.write_json()
    }

    fn write_json(&self) -> Result<()> {
        Ok(serde_json::to_writer_pretty(File::create(&self.path)?, self)?)
    }
}

pub struct Vis {
    p: Painter,
    io: Io,
    mem: Memory,
    f: HashMap<FontId, Font>,
}

impl Vis {
    pub fn new(dp_to_px: f64, scr_sz: GblSz) -> Result<Self> {
        const VIS_PATH: &str = "vis.json";
        let mut f = HashMap::new();
        f.insert(0, Font::new()?);
        let io = Io::new(dp_to_px, scr_sz);
        let mem = Memory::from_path(VIS_PATH);
        let p = Painter::new();
        Ok(Self { p, io, mem, f })
    }

    pub fn paint(&self) -> &Painter {
        &self.p
    }

    pub fn paint_mut(&mut self) -> &mut Painter {
        &mut self.p
    }

    pub fn io(&self) -> &Io {
        &self.io
    }

    pub fn io_mut(&mut self) -> &mut Io {
        &mut self.io
    }

    pub fn mem(&self) -> &Memory {
        &self.mem
    }

    pub fn mem_mut(&mut self) -> &mut Memory {
        &mut self.mem
    }

    pub fn begin(&mut self) -> Ui<'_> {
        self.paint_mut().begin();
        self.io_mut().begin();
        let scr_sz: LclSz = self.io().scr_sz.coerce();
        Ui::new(
            self,
            Layout::new(ResizeLayout::new(LayoutInfo::zero().hint(Hint::make_exact(scr_sz)))),
            "top",
        )
    }

    pub fn end(&mut self) {
        self.io_mut().end();
    }

    pub fn exit(&self) -> Result<()> {
        self.mem().exit()
    }

    pub fn font(&mut self, id: FontId) -> &mut Font {
        self.f.get_mut(&id).unwrap()
    }

    pub fn layout_text(&mut self, text: &str, dp: f64) -> Result<GblSz> {
        let f = self.f.get_mut(&0).unwrap();
        f.layout(self.io.dp_to_px, text, dp)
    }

    pub fn draw_text(&mut self, text: &str, dp: f64, l: &GblLayer) -> Result<()> {
        let f = self.f.get_mut(&0).unwrap();
        f.draw(&mut self.p, self.io.dp_to_px, text, dp, l)
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
