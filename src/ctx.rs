use std::num::NonZeroU32;
use std::sync::Arc;

use eyre::{Result, WrapErr};
use glium::Surface;
use glium::backend::glutin::Display;
use glium::glutin::config::{ConfigTemplateBuilder, GlConfig};
use glium::glutin::context::{ContextApi, ContextAttributesBuilder, Robustness};
use glium::glutin::display::GetGlDisplay;
use glium::glutin::prelude::*;
use glium::glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::visual::gui::ui::Ui;
use crate::visual::render::glium_renderer::GliumRenderer;
use crate::visual::vis::Vis;

pub struct App<F: FnMut(&mut Ui<'_>) -> Result<()>> {
    ctx: Option<Ctx>,
    f: F,
}

impl<F: FnMut(&mut Ui<'_>) -> Result<()>> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ctx.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("memevis")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_stencil_size(8)
            .with_multisampling(4);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) =
            display_builder
                .build(event_loop, template, |configs| {
                    configs
                        .reduce(|accum, config| {
                            if config.num_samples() > accum.num_samples() { config } else { accum }
                        })
                        .unwrap()
                })
                .expect("Failed to build window and GL config");

        let window = Arc::new(window.unwrap());
        let raw_window_handle = window.window_handle().unwrap().as_raw();
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .with_robustness(Robustness::RobustLoseContextOnReset)
            .build(Some(raw_window_handle));

        #[allow(unsafe_code)]
        let not_current_gl_context =
            unsafe { gl_display.create_context(&gl_config, &context_attributes) }
                .expect("Failed to create GL context");

        let (width, height): (u32, u32) = window.inner_size().into();
        let surface_attrs =
            SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(Some(true)).build(
                raw_window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );

        #[allow(unsafe_code)]
        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attrs) }
            .expect("Failed to create GL surface");

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .expect("Failed to set vsync");
        let disp = Display::from_context_surface(gl_context, gl_surface)
            .expect("Failed to create glium Display");

        let scale = window.scale_factor();
        let scr_sz = window.inner_size().to_logical::<f64>(scale).into();
        let vis = Vis::new(scale, scr_sz).unwrap();
        let rend = GliumRenderer::new(&disp).unwrap();
        self.ctx = Some(Ctx { disp, win: window, vis, rend });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ctx) = self.ctx.as_mut() else { return };

        ctx.vis.io_mut().process_event(&ctx.win, &event);
        match event {
            WindowEvent::CloseRequested => {
                if let Err(e) = ctx.vis.exit() {
                    log::error!("Error on exit: {e}");
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let mut ui = ctx.vis.begin();
                if let Err(e) = (self.f)(&mut ui) {
                    log::error!("Error in user function: {e}");
                }

                let mut t = ctx.disp.draw();
                t.clear_color_and_depth((0.6, 0.6, 0.6, 1.0), 1.0);
                ctx.vis.end();

                if let Err(e) = ctx.rend.draw(&ctx.disp, &ctx.win, &mut t, ctx.vis.paint_mut()) {
                    log::error!("Error drawing: {e}");
                }
                if let Err(e) = t.finish() {
                    log::error!("Error finishing frame: {e}");
                }
            }
            _ => {}
        }
        ctx.win.request_redraw();
    }
}

pub struct Ctx {
    disp: Display<WindowSurface>,
    win: Arc<Window>,
    vis: Vis,
    rend: GliumRenderer,
}

pub fn run(f: impl FnMut(&mut Ui<'_>) -> Result<()> + 'static) -> Result<()> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { ctx: None, f };
    event_loop.run_app(&mut app).wrap_err("blew up in event loop")
}
