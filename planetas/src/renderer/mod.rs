mod types;
mod pipelines;
mod geometry;
mod planets;

pub use types::{Vertex, Uniforms};
use types::RendererState;

use wgpu::util::DeviceExt;
use std::sync::Arc;
use winit::keyboard::KeyCode;
use glam::Mat4;

use crate::camera::Camera;
use planets::Planet;

pub struct Renderer {
    pub window: Arc<winit::window::Window>,
    state: RendererState,
    pub size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    time: f32,
    planets: Vec<Planet>,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    rotate_left: bool,
    rotate_right: bool,
    rotate_up: bool,
    rotate_down: bool,
}

impl Renderer {
    pub async fn new(window: winit::window::Window) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();
        
        let state = RendererState::new(window.clone(), size).await;
        let camera = Camera::new(size.width, size.height);
        
        // Crear planetas con posiciones iniciales diferentes
        let planets = planets::create_planet_system();

        Self {
            window,
            state,
            size,
            camera,
            time: 0.0,
            planets,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            rotate_left: false,
            rotate_right: false,
            rotate_up: false,
            rotate_down: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.state.resize(new_size);
            self.camera.resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KeyW => self.move_forward = pressed,
            KeyCode::KeyS => self.move_backward = pressed,
            KeyCode::KeyA => self.move_left = pressed,
            KeyCode::KeyD => self.move_right = pressed,
            KeyCode::Space => self.move_up = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.move_down = pressed,
            KeyCode::ArrowLeft => self.rotate_left = pressed,
            KeyCode::ArrowRight => self.rotate_right = pressed,
            KeyCode::ArrowUp => self.rotate_up = pressed,
            KeyCode::ArrowDown => self.rotate_down = pressed,
            _ => {}
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();
        self.time += dt;

        // Actualizar cámara
        self.update_camera(dt);
        
        // Actualizar planetas
        for planet in &mut self.planets {
            planet.update(self.time);
        }
    }

    fn update_camera(&mut self, dt: f32) {
        if self.move_forward {
            self.camera.move_forward(dt);
        }
        if self.move_backward {
            self.camera.move_backward(dt);
        }
        if self.move_left {
            self.camera.move_left(dt);
        }
        if self.move_right {
            self.camera.move_right(dt);
        }
        if self.move_up {
            self.camera.move_up(dt);
        }
        if self.move_down {
            self.camera.move_down(dt);
        }

        let mut delta_yaw = 0.0;
        let mut delta_pitch = 0.0;
        
        if self.rotate_left {
            delta_yaw -= dt * 2.0;
        }
        if self.rotate_right {
            delta_yaw += dt * 2.0;
        }
        if self.rotate_up {
            delta_pitch += dt * 2.0;
        }
        if self.rotate_down {
            delta_pitch -= dt * 2.0;
        }
        
        if delta_yaw != 0.0 || delta_pitch != 0.0 {
            self.camera.rotate(delta_yaw, delta_pitch);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let view_proj = self.camera.build_view_projection_matrix();

        // Actualizar uniformes ANTES del render pass para cada planeta
        for (i, planet) in self.planets.iter().enumerate() {
            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: planet.get_model_matrix().to_cols_array_2d(),
                time: self.time,
                _padding: [0.0; 3],
            };

            // Escribir a un offset específico para cada planeta
            self.state.queue.write_buffer(
                &self.state.uniform_buffers[i],
                0,
                bytemuck::cast_slice(&[uniforms])
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.state.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_vertex_buffer(0, self.state.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // Renderizar cada planeta con su propio bind group
            for (i, planet) in self.planets.iter().enumerate() {
                let pipeline = match planet.planet_type {
                    planets::PlanetType::Sun => &self.state.sun_pipeline,
                    planets::PlanetType::Rocky => &self.state.rocky_planet_pipeline,
                    planets::PlanetType::Gas => &self.state.gas_planet_pipeline,
                };

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &self.state.uniform_bind_groups[i], &[]);
                render_pass.draw_indexed(0..self.state.num_indices, 0, 0..1);
            }
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}