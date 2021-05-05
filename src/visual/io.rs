use crate::visual::types::{pt, GblPt, GblSz, GblZ, Pt};
use glium::glutin::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use glium::glutin::window::Window;
use num_traits::Zero;
use std::time::Instant;

#[derive(Debug)]
pub struct Io {
    // Sizes:
    pub dp_to_px: f64,
    pub scr_sz: GblSz,

    // Keyboard:
    pub kbd_captured: Option<String>,
    pub has_mouse: Option<String>,
    pub has_kbd: Option<String>,
    kbd_req: Option<(GblZ, String, bool)>,

    // Mouse:
    pub mouse_pt: GblPt,
    pub prev_mouse_pt: GblPt,
    pub mouse_delta: GblPt,
    pub mouse_captured: Option<String>,
    pub is_mouse_pressed: bool,
    pub mouse_pressed_pt: GblPt,
    pub mouse_just_released: bool,
    pub mouse_just_captured: bool,
    pub mouse_scroll: Pt,
    mouse_req: Option<(GblZ, String)>,
    mouse_capture_req: Option<(GblZ, String)>,

    // Frames:
    pub begin_frame_time: Instant,
    pub prev_begin_frame_time: Instant,
    pub prev_end_frame_time: Instant,
    pub frame_num: u64,
}

impl Io {
    pub fn new(dp_to_px: f64, scr_sz: GblSz) -> Self {
        Self {
            mouse_pt: Default::default(),
            prev_mouse_pt: Default::default(),
            mouse_delta: Default::default(),
            dp_to_px,
            scr_sz,
            mouse_captured: None,
            kbd_captured: None,
            has_mouse: None,
            has_kbd: None,
            is_mouse_pressed: false,
            mouse_pressed_pt: Default::default(),
            mouse_just_released: false,
            mouse_just_captured: false,
            begin_frame_time: Instant::now(),
            prev_begin_frame_time: Instant::now(),
            prev_end_frame_time: Instant::now(),
            frame_num: 0,
            mouse_req: None,
            mouse_capture_req: None,
            kbd_req: None,
            mouse_scroll: Pt::zero(),
        }
    }

    pub fn begin(&mut self) {
        self.begin_frame_time = Instant::now();
        self.frame_num += 1;
        self.mouse_delta = self.mouse_pt - self.prev_mouse_pt;

        // Try capture first, it takes precedence.
        if self.mouse_captured != self.has_mouse {
            self.mouse_just_captured = true;
        }
        self.mouse_captured = self.mouse_capture_req.take().map(|f| f.1);
        self.has_mouse = self.mouse_captured.clone();

        // Check regular requests.
        let mouse_req = self.mouse_req.take().map(|f| f.1);
        self.has_mouse = self.has_mouse.take().or(mouse_req);
    }

    pub fn end(&mut self) {
        self.mouse_just_captured = false;
        self.mouse_just_released = false;
        self.mouse_scroll = Pt::zero();
        self.prev_mouse_pt = self.mouse_pt;
        self.prev_end_frame_time = Instant::now();
        self.prev_begin_frame_time = self.begin_frame_time;
    }

    pub fn mouse_req(&mut self, z: GblZ, id: &str) {
        let id = id.to_owned();
        let req = &mut self.mouse_req;
        if req.is_some() && z >= req.as_ref().unwrap().0 || req.is_none() {
            *req = Some((z, id));
        }
    }

    pub fn mouse_capture(&mut self, z: GblZ, id: &str) {
        // Update capture request if we don't have one, or it's a higher z-order
        // and won't stop a capture being prolonged, or it's the current
        // capture.
        let id = id.to_owned();
        if let Some(req) = &self.mouse_capture_req {
            let capture_prolonged = Some(&req.1) == self.mouse_captured.as_ref();
            if !capture_prolonged && (z >= req.0 || self.mouse_captured.contains(&id)) {
                self.mouse_capture_req = Some((z, id));
            }
        } else {
            self.mouse_capture_req = Some((z, id));
        }
    }

    pub fn process_event(&mut self, w: &Window, e: WindowEvent<'_>) {
        match e {
            WindowEvent::Resized(ps) => self.scr_sz = ps.to_logical::<f64>(w.scale_factor()).into(),
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {}
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::ReceivedCharacter(_) => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput { .. } => {}
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pt = position.to_logical::<f64>(w.scale_factor()).into();
            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => self.mouse_scroll += pt(x, -y),
                MouseScrollDelta::PixelDelta(_) => {}
            },
            WindowEvent::MouseInput { button, state, .. } => {
                if button == MouseButton::Left {
                    self.mouse_just_released =
                        state == ElementState::Released && self.is_mouse_pressed;
                    self.is_mouse_pressed = state == ElementState::Pressed;
                    self.mouse_pressed_pt = self.mouse_pt;
                }
            }
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
                self.dp_to_px = scale_factor as f64;
                self.scr_sz = new_inner_size.to_logical::<f64>(w.scale_factor()).into()
            }
            WindowEvent::ThemeChanged(_) => {}
        }
    }
}
