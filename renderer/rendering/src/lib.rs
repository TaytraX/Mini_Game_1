mod render_backend;
mod texture;
mod camera;
mod block_types;
mod chunk_renderer;
mod jni_interface;

use render_backend::State;

use std::sync::Arc;
use jni::JNIEnv;
use jni::objects::JClass;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};
use winit::dpi::PhysicalSize;
use winit::window::{CursorGrabMode, WindowId};
use jni::sys::jdouble;
use winit::keyboard::KeyCode;
use std::sync::{OnceLock, RwLock};

static GLOBAL_POSITION: OnceLock<RwLock<(f32, f32, f32)>> = OnceLock::new();

pub struct App {
    state: Option<State>,
    last_time: instant::Instant,
    would_block: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            last_time: instant::Instant::now(),
            would_block: false,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();
        window_attributes.inner_size = Some(
            PhysicalSize {
                width: 1200,
                height: 800,
            }
                .into(),
        );
        window_attributes.title = "Mini Game".to_string();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window.set_cursor_visible(self.would_block);

        if !self.would_block {
            let _ = window.set_cursor_position(winit::dpi::PhysicalPosition::new(
                window.inner_size().width / 2,
                window.inner_size().height / 2,
            ));
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        event.window.request_redraw();
        event.resize(
            event.window.inner_size().width,
            event.window.inner_size().height,
        );
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = instant::Instant::now();
                let pos = get_position();

                // Vérifier si le chunk doit être mis à jour
                if jni_interface::check_and_clear_update_flag() {
                    if let Err(e) = state.update_chunk_from_java() {
                        log::error!("Failed to update chunk: {}", e);
                    }
                }

                state.update(dt);
                state.update_instance(pos);

                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                state.camera_controller.handle_mouse_scroll(&delta);
            }
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (KeyCode::KeyR, true) => {
                    if !self.would_block {
                        self.would_block = true;
                        let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                    } else {
                        self.would_block = false;
                        let _ = state.window.set_cursor_grab(CursorGrabMode::Locked);
                    }
                }
                _ => state.handle_key(event_loop, code, key_state.is_pressed()),
            },
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            return;
        };

        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                state
                    .camera_controller
                    .handle_mouse(dx, dy, self.would_block);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

pub fn set_position(pos: (f32, f32, f32)) {
    let lock = GLOBAL_POSITION.get_or_init(|| RwLock::new((0.0, 0.0, 0.0)));
    if let Ok(mut position) = lock.write() {
        *position = pos;
        println!("Position set to: {:?}", pos);
    }
}

pub fn get_position() -> (f32, f32, f32) {
    let lock = GLOBAL_POSITION.get_or_init(|| RwLock::new((0.0, 0.0, 0.0)));
    lock.read().map(|p| *p).unwrap_or((0.0, 0.0, 0.0))
}

fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_Teste_render<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    run().unwrap()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_Teste_updateValue<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
    set_position((x as f32, y as f32, z as f32));
    println!("Position mise à jour avec succès");
}

// Ré-exporter la fonction JNI pour les chunks
pub use jni_interface::Java_Teste_updateChunk;