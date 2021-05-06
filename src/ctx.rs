use crate::visual::gui::layouts::resize_layout::ResizeLayout;
use crate::visual::gui::ui::Ui;
use crate::visual::render::glium_renderer::GliumRenderer;
use crate::visual::vis::Vis;
use eyre::Result;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::{GlProfile, Robustness};
use glium::{glutin, Display, Surface};

pub struct Ctx {
    disp: Display,
    vis: Vis,
    rend: GliumRenderer,
}

impl Ctx {
    pub async fn new(ev: &EventLoop<()>) -> Result<Self> {
        let gl = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_srgb(false) // We do our own correction in shaders.
            .with_multisampling(4)
            .with_gl_profile(GlProfile::Core)
            .with_gl_robustness(Robustness::TryRobustLoseContextOnReset);
        let win = WindowBuilder::new()
            .with_title("hodlr")
            .with_inner_size(LogicalSize::new(1024f64, 768f64));
        let disp = Display::new(win, gl, &ev).expect("Failed to initialize display");
        let scale = disp.gl_window().window().scale_factor();
        let scr_sz = disp
            .gl_window()
            .window()
            .inner_size()
            .to_logical::<f64>(scale)
            .into();
        let vis = Vis::new(scale as f64, scr_sz)?;
        let rend = GliumRenderer::new(&disp)?;
        Ok(Self { disp, vis, rend })
    }

    pub fn run(
        mut self,
        ev: EventLoop<()>,
        mut f: impl FnMut(&mut Ui<'_, ResizeLayout>) -> Result<()> + 'static,
    ) -> Result<()> {
        ev.run(move |e, _, flow| self.event_loop(e, flow, &mut f).unwrap())
    }

    fn event_loop(
        &mut self,
        e: Event<'_, ()>,
        flow: &mut ControlFlow,
        f: &mut impl FnMut(&mut Ui<'_, ResizeLayout>) -> Result<()>,
    ) -> Result<()> {
        *flow = ControlFlow::Wait;
        match e {
            Event::NewEvents(_) => {}
            Event::MainEventsCleared => {
                self.disp.gl_window().window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let mut ui = self.vis.begin()?;
                f(&mut ui)?;

                let mut t = self.disp.draw();
                t.clear_color_and_depth((0.6, 0.6, 0.6, 1.0), 1.0);
                self.vis.end();

                self.rend.draw(&self.disp, &mut t, &mut self.vis.p).unwrap();
                t.finish()?;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                self.vis.exit()?;
                *flow = ControlFlow::Exit
            }
            Event::WindowEvent { event, .. } => self
                .vis
                .io
                .process_event(self.disp.gl_window().window(), event),
            _ => {}
        }
        Ok(())
    }
}
