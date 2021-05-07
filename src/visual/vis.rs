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
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

#[derive(Clone)]
pub struct Vis {
    p: Arc<Mutex<Painter>>,
    io: Arc<Mutex<Io>>,
    mem: Arc<Mutex<Memory>>,
    f: Arc<Mutex<HashMap<FontId, Font>>>,
}

impl Vis {
    pub fn new(dp_to_px: f64, scr_sz: GblSz) -> Result<Self> {
        const VIS_PATH: &str = "vis.json";
        let f = Arc::new(Mutex::new(HashMap::new()));
        f.lock().insert(0, Font::new()?);
        let io = Arc::new(Mutex::new(Io::new(dp_to_px, scr_sz)));
        let mem = Arc::new(Mutex::new(Memory::from_path(VIS_PATH)));
        let p = Arc::new(Mutex::new(Painter::new()));
        Ok(Self { p, io, mem, f })
    }

    pub fn paint(&self) -> MutexGuard<'_, Painter> {
        self.p.try_lock().expect("BUG")
    }

    pub fn io(&self) -> MutexGuard<'_, Io> {
        self.io.try_lock().expect("BUG")
    }

    pub fn mem(&self) -> MutexGuard<'_, Memory> {
        self.mem.try_lock().expect("BUG")
    }

    pub fn begin(&self) -> Ui {
        self.paint().begin();
        self.io().begin();
        let scr_sz: LclSz = self.io().scr_sz.coerce();
        Ui::new(
            self.clone(),
            Layout::new(ResizeLayout::new(LayoutInfo::zero().hint(Hint::make_exact(scr_sz)))),
            "top",
        )
    }

    pub fn end(&self) {
        self.io().end();
    }

    pub fn exit(&self) -> Result<()> {
        self.mem().exit()
    }

    pub fn font(&self, id: FontId) -> MappedMutexGuard<'_, Font> {
        MutexGuard::map(self.f.try_lock().expect("BUG"), |v| v.get_mut(&id).unwrap())
    }

    pub fn layout_text(&self, text: &str, dp: f64) -> Result<GblSz> {
        let dp_to_px = self.io().dp_to_px;
        let mut f = self.font(0);
        f.layout(dp_to_px, text, dp)
    }

    pub fn draw_text(&self, text: &str, dp: f64, l: &GblLayer) -> Result<()> {
        let dp_to_px = self.io().dp_to_px;
        let mut paint = self.paint();
        let mut f = self.font(0);
        f.draw(&mut paint, dp_to_px, text, dp, l)
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
