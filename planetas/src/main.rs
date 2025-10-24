use winit::{
    event::*,
    event_loop::EventLoop,
    application::ApplicationHandler,
};

mod renderer;
mod camera;

use renderer::Renderer;

struct App {
    renderer: Option<Renderer>,
    last_render_time: std::time::Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.renderer.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Planetas Celestes")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            
            let window = event_loop.create_window(window_attributes).unwrap();
            let renderer = pollster::block_on(Renderer::new(window));
            self.renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                renderer.resize(physical_size);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state,
                    physical_key: winit::keyboard::PhysicalKey::Code(key),
                    ..
                },
                ..
            } => {
                renderer.input(key, state == ElementState::Pressed);
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;
                
                renderer.update(dt);
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        renderer.resize(renderer.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(wgpu::SurfaceError::Timeout) => eprintln!("Surface timeout"),
                }
                
                renderer.window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    
    let mut app = App {
        renderer: None,
        last_render_time: std::time::Instant::now(),
    };
    
    event_loop.run_app(&mut app).unwrap();
}