use crate::visual::types::{GblRt, TexPt, TexSz, TexUvRect};
use derive_more::Display;
use rgb::RGBA8;
use std::collections::hash_map::IterMut;
use std::collections::HashMap;

pub(super) type TexId = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextureLayer {
    pub r: GblRt,
    pub uv: TexUvRect,
    pub(super) tex: TexId,
}

impl TextureLayer {
    pub fn new(r: GblRt, uv: TexUvRect, tex: TexId) -> Self {
        Self { r, uv, tex }
    }

    pub fn with_rect(self, r: GblRt) -> Self {
        Self { r, ..self }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Display)]
#[display(fmt = "Tex[id:{}, sz:{}, dirty:{}]", id, sz, dirty)]
pub(super) struct Tex {
    pub id: TexId,
    pub sz: TexSz,
    pub data: Vec<RGBA8>,
    pub dirty: bool,
}

impl Tex {
    pub fn new(id: TexId, sz: TexSz) -> Self {
        let numpix: usize = sz.w as usize * sz.h as usize;
        let mut data = Vec::with_capacity(numpix);
        data.resize(numpix, RGBA8::new(0, 0, 0, 0));
        Self { id, sz, data, dirty: true }
    }

    pub fn write(&mut self, p: TexPt, c: RGBA8) {
        self.data[(self.sz.w * p.y + p.x) as usize] = c;
        self.dirty = true;
    }
}

#[derive(Debug)]
pub(super) struct TexStore {
    last_id: TexId,
    texs: HashMap<TexId, Tex>,
}

impl TexStore {
    pub fn new() -> Self {
        Self { last_id: 1usize, texs: HashMap::new() }
    }

    pub fn insert(&mut self, sz: TexSz) -> TexId {
        let id = self.last_id;
        self.last_id += 1;
        self.texs.insert(id, Tex::new(id, sz));
        id
    }

    pub fn get_mut(&mut self, id: TexId) -> &mut Tex {
        self.texs.get_mut(&id).expect("expected texture id")
    }

    pub fn contains(&mut self, id: TexId) -> bool {
        self.texs.contains_key(&id)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, TexId, Tex> {
        self.texs.iter_mut()
    }
}
