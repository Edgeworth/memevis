use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;

use eyre::{eyre, Result};
use glium::index::PrimitiveType;
use glium::program::ProgramCreationInput;
use glium::texture::{RawImage2d, Texture2d};
use glium::uniforms::{AsUniformValue, UniformValue, Uniforms};
use glium::{
    implement_vertex, Blend, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer,
    Program, Surface, VertexBuffer,
};
use lyon::geom::LineSegment;
use lyon::lyon_tessellation::{FillVertex, StrokeVertex};
use lyon::math::Point;
use lyon::path::builder::BorderRadii;
use lyon::path::traits::PathBuilder;
use lyon::path::{Path, Polygon, Winding};
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertexConstructor, StrokeOptions,
    StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
};
use rgb::ComponentBytes;

use crate::errors::StringErrorConversion;
use crate::visual::render::painter::{PaintCtx, PaintOp, Painter};
use crate::visual::render::texture::{TexId, TexStore};
use crate::visual::types::{lsz, tpt, Col, GblZ, TexRt, TexSz};

const TOLERANCE: f32 = 0.1;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
struct Vertex {
    p: [f32; 2],
    c: [f32; 4],
    uv: [f32; 2],
}

implement_vertex!(Vertex, p, c, uv);

#[derive(Debug)]
struct VertexCtor {
    c: Col,
}

impl VertexCtor {
    pub fn new(c: Col) -> Self {
        Self { c }
    }

    fn build_vertex(&self, v: Point, attrs: &[f32]) -> Vertex {
        Vertex {
            p: [v.x, v.y],
            c: if attrs.len() == 4 { attrs.try_into().unwrap() } else { self.c.into() },
            uv: if attrs.len() == 2 { attrs.try_into().unwrap() } else { Default::default() },
        }
    }
}

impl FillVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, mut v: FillVertex<'_>) -> Vertex {
        self.build_vertex(v.position(), v.interpolated_attributes())
    }
}

impl StrokeVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, mut v: StrokeVertex<'_, '_>) -> Vertex {
        self.build_vertex(v.position(), v.interpolated_attributes())
    }
}

struct UniformMap<'a>(HashMap<String, Box<dyn 'a + AsUniformValue>>);

impl<'a> UniformMap<'a> {
    pub fn add_val<T: 'a + AsUniformValue + Copy>(&mut self, name: &str, val: T) {
        self.0.insert(name.into(), Box::new(val));
    }

    pub fn add_ref<T>(&mut self, name: &str, val: &'a T)
    where
        &'a T: AsUniformValue,
    {
        self.0.insert(name.into(), Box::new(val));
    }
}

impl Uniforms for UniformMap<'_> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        for (name, val) in self.0.iter() {
            f(name, val.as_uniform_value());
        }
    }
}

pub struct GliumRenderer {
    filler: FillTessellator,
    stroker: StrokeTessellator,
    texmap: HashMap<TexId, Texture2d>,
    prog: Program,
}

struct DrawContext<'a> {
    disp: &'a Display,
    frame: &'a mut Frame,
    draw_params: DrawParameters<'a>,
}

const VERTEX: &str = include_str!("../../assets/shader.vert");
const FRAG: &str = include_str!("../../assets/shader.frag");

impl GliumRenderer {
    pub fn new(disp: &Display) -> Result<Self> {
        let prog = Program::new(
            disp,
            ProgramCreationInput::SourceCode {
                vertex_shader: VERTEX,
                fragment_shader: FRAG,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )?;
        Ok(Self {
            filler: FillTessellator::new(),
            stroker: StrokeTessellator::new(),
            texmap: HashMap::new(),
            prog,
        })
    }

    pub fn draw(&mut self, disp: &Display, frame: &mut Frame, p: &mut Painter) -> Result<()> {
        let mut dtx = DrawContext {
            disp,
            frame,
            draw_params: DrawParameters {
                depth: Depth { test: DepthTest::Overwrite, ..Default::default() },
                blend: Blend::alpha_blending(),
                ..Default::default()
            },
        };
        self.update_textures(&mut dtx, &mut p.ts)?;
        self.render(&mut dtx, &p.ops)?;
        disp.gl_window().window().set_cursor_icon(p.cursor);
        Ok(())
    }

    fn update_textures(&mut self, dtx: &'_ mut DrawContext<'_>, ts: &mut TexStore) -> Result<()> {
        // Load new textures.
        for (tid, tex) in ts.iter_mut() {
            if tex.dirty {
                let image = RawImage2d::from_raw_rgba(
                    tex.data.as_slice().as_bytes().to_vec(),
                    tex.sz.into(),
                );
                match self.texmap.get(tid) {
                    None => {
                        self.texmap.insert(*tid, Texture2d::new(dtx.disp, image)?);
                    }
                    Some(gltex) => {
                        gltex.write(TexRt::ptsz(tpt(0, 0), tex.sz).into(), image);
                    }
                }
                tex.dirty = false;
            }
        }
        // Get rid of old textures.
        self.texmap.retain(|tid, _| ts.contains(*tid));
        Ok(())
    }

    fn render(&mut self, dtx: &'_ mut DrawContext<'_>, ops: &[(PaintCtx, PaintOp)]) -> Result<()> {
        let mut geom_map: BTreeMap<(GblZ, Option<TexId>), VertexBuffers<Vertex, u16>> =
            BTreeMap::new();

        let fopt = FillOptions::tolerance(TOLERANCE);
        for (pctx, op) in ops.iter() {
            let tf = pctx.tf;
            let z = tf.z(pctx.z);

            let mut geom = geom_map.entry((z, None)).or_default();
            if let PaintOp::Texture { tex } = op {
                geom = geom_map.entry((z, Some(tex.tex))).or_default();
            }
            let mut buf = BuffersBuilder::new(&mut geom, VertexCtor::new(pctx.col));

            let line_width_px = pctx.line_width * dtx.disp.gl_window().window().scale_factor();
            let sopt = StrokeOptions::tolerance(TOLERANCE).with_line_width(line_width_px as f32);

            let mut b = Path::builder();
            match *op {
                PaintOp::FillPath { ref p } => {
                    self.filler.tessellate_path(&tf.path(p.clone()), &fopt, &mut buf).serr()?;
                }
                PaintOp::FillCirc { center, radius } => {
                    // TODO: Size conversion of radius may be different in different axes.
                    self.filler
                        .tessellate_circle(tf.pt(center).into(), radius as f32, &fopt, &mut buf)
                        .serr()?;
                }
                PaintOp::FillPoly { ref pts } => {
                    let points = &pts.iter().map(|v| tf.pt(*v).into()).collect::<Vec<_>>();
                    self.filler
                        .tessellate_polygon(Polygon { points, closed: true }, &fopt, &mut buf)
                        .serr()?;
                }
                PaintOp::FillQuad { v } => {
                    b.add_polygon(Polygon {
                        points: &[
                            tf.pt(v[0]).into(),
                            tf.pt(v[1]).into(),
                            tf.pt(v[2]).into(),
                            tf.pt(v[3]).into(),
                        ],
                        closed: true,
                    });
                    self.filler.tessellate_path(&b.build(), &fopt, &mut buf).serr()?;
                }
                PaintOp::FillRt { r } => {
                    self.filler.tessellate_rectangle(&tf.rt(r).into(), &fopt, &mut buf).serr()?;
                }
                PaintOp::FillRRt { r, radius } => {
                    // TODO: Ignores scaling with different x and y scales.
                    let radii = tf.sz(lsz(radius, radius));
                    let radii = &BorderRadii {
                        top_left: radii.w as f32,
                        top_right: radii.w as f32,
                        bottom_left: radii.w as f32,
                        bottom_right: radii.w as f32,
                    };
                    b.add_rounded_rectangle(&tf.rt(r).into(), radii, Winding::Positive);
                    self.filler.tessellate_path(&b.build(), &fopt, &mut buf).serr()?;
                }
                PaintOp::StrokeLine { st, en } => {
                    b.add_line_segment(&LineSegment {
                        from: tf.pt(st).into(),
                        to: tf.pt(en).into(),
                    });
                    self.stroker.tessellate_path(&b.build(), &sopt, &mut buf).serr()?;
                }

                PaintOp::StrokePath { ref p } => {
                    self.stroker.tessellate_path(&tf.path(p.clone()), &sopt, &mut buf).serr()?;
                }
                PaintOp::StrokeCirc { center, radius } => {
                    self.stroker
                        .tessellate_circle(tf.pt(center).into(), radius as f32, &sopt, &mut buf)
                        .serr()?;
                }
                PaintOp::StrokeEllipse { center, radii, rot } => {
                    b.add_ellipse(
                        tf.pt(center).into(),
                        tf.sz(radii).into(),
                        rot,
                        Winding::Positive,
                    );
                    self.stroker.tessellate_path(&b.build(), &sopt, &mut buf).serr()?;
                }
                PaintOp::StrokePoly { ref pts, is_closed } => {
                    let points = &pts.iter().map(|v| tf.pt(*v).into()).collect::<Vec<_>>();
                    self.stroker
                        .tessellate_polygon(Polygon { points, closed: is_closed }, &sopt, &mut buf)
                        .serr()?;
                }
                PaintOp::StrokeQuad { v } => {
                    b.add_polygon(Polygon {
                        points: &[
                            tf.pt(v[0]).into(),
                            tf.pt(v[1]).into(),
                            tf.pt(v[2]).into(),
                            tf.pt(v[3]).into(),
                        ],
                        closed: true,
                    });
                    self.stroker.tessellate_path(&b.build(), &sopt, &mut buf).serr()?;
                }
                PaintOp::StrokeRt { r } => {
                    self.stroker.tessellate_rectangle(&tf.rt(r).into(), &sopt, &mut buf).serr()?;
                }
                PaintOp::StrokeRRt { r, radius } => {
                    // TODO: Ignores scaling with different x and y scales.
                    let radii = tf.sz(lsz(radius, radius));
                    let radii = &BorderRadii {
                        top_left: radii.w as f32,
                        top_right: radii.w as f32,
                        bottom_left: radii.w as f32,
                        bottom_right: radii.w as f32,
                    };
                    b.add_rounded_rectangle(&tf.rt(r).into(), radii, Winding::Positive);
                    self.stroker.tessellate_path(&b.build(), &sopt, &mut buf).serr()?;
                }
                PaintOp::StrokeTri { v } => {
                    self.stroker
                        .tessellate_polygon(
                            Polygon {
                                points: &[
                                    tf.pt(v[0]).into(),
                                    tf.pt(v[1]).into(),
                                    tf.pt(v[2]).into(),
                                ],
                                closed: true,
                            },
                            &sopt,
                            &mut buf,
                        )
                        .serr()?;
                }
                PaintOp::Texture { tex } => {
                    let mut b = Path::builder_with_attributes(2);
                    b.begin(tex.r.bl().into(), &tex.uv.tl().to_f32().to_arr());
                    b.line_to(tex.r.br().into(), &tex.uv.tr().to_f32().to_arr());
                    b.line_to(tex.r.tr().into(), &tex.uv.br().to_f32().to_arr());
                    b.line_to(tex.r.tl().into(), &tex.uv.bl().to_f32().to_arr());
                    b.end(true);
                    self.filler.tessellate_path(&b.build(), &fopt, &mut buf).serr()?;
                }
            }
        }

        let gl = dtx.disp.gl_window();
        let sz = TexSz::from(gl.window().inner_size()).to_f64();
        let sf = gl.window().scale_factor() as f64;
        let dp = sz / sf;
        let mut uni = UniformMap(HashMap::new());
        uni.add_val::<(f32, f32)>("screen_dp", (dp.w as f32, dp.h as f32));

        for ((_, tex), geom) in geom_map.iter() {
            let vertices = VertexBuffer::new(dtx.disp, &geom.vertices)?;
            let indices = IndexBuffer::new(dtx.disp, PrimitiveType::TrianglesList, &geom.indices)?;
            uni.add_val("use_tex", tex.is_some());

            if let Some(tid) = tex {
                uni.add_ref(
                    "sampler",
                    self.texmap.get(&tid).ok_or_else(|| eyre!("unknown texture id"))?,
                );
            }
            dtx.frame.draw(&vertices, &indices, &self.prog, &uni, &dtx.draw_params)?;
        }
        Ok(())
    }
}
