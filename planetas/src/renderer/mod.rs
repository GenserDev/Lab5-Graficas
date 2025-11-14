mod types;
mod pipelines;
mod geometry;
mod planets;
mod ship;
mod skybox;

pub use types::Uniforms;
use types::RendererState;

use std::sync::Arc;
use winit::keyboard::KeyCode;
use glam::{Mat4, Vec3, Quat};

use crate::camera::Camera;
use planets::get_warp_points;
use planets::Planet;

pub struct Renderer {
    pub window: Arc<winit::window::Window>,
    state: RendererState,
    pub size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    time: f32,
    planets: Vec<Planet>,
    warp_points: Vec<planets::WarpPoint>,
    current_warp: usize,
    warp_progress: f32,
    is_warping: bool,
    warp_start_pos: Vec3,
    warp_start_yaw: f32,
    warp_start_pitch: f32,
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
        
        let planets = planets::create_planet_system();
        let warp_points = get_warp_points();

        Self {
            window,
            state,
            size,
            camera,
            time: 0.0,
            planets,
            warp_points,
            current_warp: 0,
            warp_progress: 0.0,
            is_warping: false,
            warp_start_pos: Vec3::ZERO,
            warp_start_yaw: 0.0,
            warp_start_pitch: 0.0,
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
            KeyCode::Digit1 if pressed => self.initiate_warp(0),
            KeyCode::Digit2 if pressed => self.initiate_warp(1),
            KeyCode::Digit3 if pressed => self.initiate_warp(2),
            KeyCode::Digit4 if pressed => self.initiate_warp(3),
            KeyCode::Digit5 if pressed => self.initiate_warp(4),
            KeyCode::Digit6 if pressed => self.initiate_warp(5),
            KeyCode::Digit7 if pressed => self.initiate_warp(6),
            _ => {}
        }
    }

    fn initiate_warp(&mut self, warp_index: usize) {
        if warp_index < self.warp_points.len() && !self.is_warping {
            self.current_warp = warp_index;
            self.is_warping = true;
            self.warp_progress = 0.0;
            self.warp_start_pos = self.camera.position;
            self.warp_start_yaw = self.camera.yaw;
            self.warp_start_pitch = self.camera.pitch;
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();
        self.time += dt;

        if self.is_warping {
            self.update_warp(dt);
        } else {
            self.update_camera(dt);
        }
        
        self.check_collisions();
        
        for planet in &mut self.planets {
            planet.update(self.time);
        }
    }

    fn update_warp(&mut self, dt: f32) {
        self.warp_progress += dt * 1.5;
        
        if self.warp_progress >= 1.0 {
            self.warp_progress = 1.0;
            self.is_warping = false;
            let target_point = &self.warp_points[self.current_warp];
            self.camera.position = target_point.position;
            let dir = (target_point.target - target_point.position).normalize();
            self.camera.yaw = dir.x.atan2(dir.z);
            self.camera.pitch = dir.y.asin();
        } else {
            let t = ease_in_out_cubic(self.warp_progress);
            let target_point = &self.warp_points[self.current_warp];
            
            self.camera.position = self.warp_start_pos.lerp(target_point.position, t);
            
            let start_dir = Vec3::new(
                self.warp_start_yaw.cos() * self.warp_start_pitch.cos(),
                self.warp_start_pitch.sin(),
                self.warp_start_yaw.sin() * self.warp_start_pitch.cos(),
            );
            let target_dir = (target_point.target - target_point.position).normalize();
            
            let start_quat = Quat::from_rotation_arc(Vec3::Z, start_dir);
            let target_quat = Quat::from_rotation_arc(Vec3::Z, target_dir);
            let current_quat = start_quat.slerp(target_quat, t);
            let current_dir = current_quat * Vec3::Z;
            
            self.camera.yaw = current_dir.x.atan2(current_dir.z);
            self.camera.pitch = current_dir.y.asin();
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

    fn check_collisions(&mut self) {
        let min_distance = 5.0;
        
        for planet in &self.planets {
            let planet_pos = planet.get_position();
            let distance = (self.camera.position - planet_pos).length();
            let collision_radius = planet.scale + min_distance;
            
            if distance < collision_radius {
                let push_dir = (self.camera.position - planet_pos).normalize();
                self.camera.position = planet_pos + push_dir * collision_radius;
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let view_proj = self.camera.build_view_projection_matrix();

        // Actualizar uniform del skybox: centrar skybox en la posición de la cámara
        let skybox_uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: Mat4::from_translation(self.camera.position).to_cols_array_2d(),
            time: self.time,
            _padding: [0.0; 3],
        };
        self.state.queue.write_buffer(
            &self.state.skybox_uniform_buffer,
            0,
            bytemuck::cast_slice(&[skybox_uniforms])
        );

        // Actualizar uniformes de planetas
        for (i, planet) in self.planets.iter().enumerate() {
            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: planet.get_model_matrix().to_cols_array_2d(),
                time: self.time,
                _padding: [0.0; 3],
            };
            self.state.queue.write_buffer(
                &self.state.planet_uniform_buffers[i],
                0,
                bytemuck::cast_slice(&[uniforms])
            );
        }

        // Actualizar uniformes de la nave
        let ship_model = Mat4::from_translation(self.camera.position + self.camera.get_forward() * 3.0 + Vec3::new(0.5, -0.5, 0.0))
            * Mat4::from_rotation_y(self.camera.yaw + std::f32::consts::PI)
            * Mat4::from_rotation_x(-self.camera.pitch)
            * Mat4::from_scale(Vec3::splat(0.5));
        
        let ship_uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: ship_model.to_cols_array_2d(),
            time: self.time,
            _padding: [0.0; 3],
        };
        self.state.queue.write_buffer(
            &self.state.ship_uniform_buffer,
            0,
            bytemuck::cast_slice(&[ship_uniforms])
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
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

            // Renderizar skybox primero
            render_pass.set_pipeline(&self.state.skybox_pipeline);
            render_pass.set_bind_group(0, &self.state.skybox_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.state.skybox_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.state.skybox_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.state.skybox_num_indices, 0, 0..1);

            // Renderizar órbitas
            render_pass.set_pipeline(&self.state.orbit_pipeline);
            render_pass.set_vertex_buffer(0, self.state.orbit_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.state.orbit_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            
            for (i, planet) in self.planets.iter().enumerate() {
                if planet.orbit_radius > 0.0 {
                    let (start, count) = self.state.orbit_ranges[i];
                    if count > 0 {
                        render_pass.set_bind_group(0, &self.state.orbit_bind_groups[i], &[]);
                        render_pass.draw_indexed(start..(start + count), 0, 0..1);
                    }
                }
            }

            // Renderizar planetas
            render_pass.set_vertex_buffer(0, self.state.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for (i, planet) in self.planets.iter().enumerate() {
                let pipeline = &self.state.planet_pipelines[planet.planet_type as usize];
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &self.state.planet_bind_groups[i], &[]);
                render_pass.draw_indexed(0..self.state.num_indices, 0, 0..1);
                
                // Renderizar luna si existe
                if planet.has_moon {
                    if let Some(moon_matrix) = planet.get_moon_model_matrix() {
                        let moon_uniforms = Uniforms {
                            view_proj: view_proj.to_cols_array_2d(),
                            model: moon_matrix.to_cols_array_2d(),
                            time: self.time,
                            _padding: [0.0; 3],
                        };
                        self.state.queue.write_buffer(
                            &self.state.moon_uniform_buffers[i],
                            0,
                            bytemuck::cast_slice(&[moon_uniforms])
                        );
                        
                        render_pass.set_pipeline(&self.state.moon_pipeline);
                        render_pass.set_bind_group(0, &self.state.moon_bind_groups[i], &[]);
                        render_pass.draw_indexed(0..self.state.num_indices, 0, 0..1);
                    }
                }
                
                // Renderizar anillos si existen
                if planet.has_rings {
                    if let Some(ring_matrix) = planet.get_rings_model_matrix() {
                        let ring_uniforms = Uniforms {
                            view_proj: view_proj.to_cols_array_2d(),
                            model: ring_matrix.to_cols_array_2d(),
                            time: self.time,
                            _padding: [0.0; 3],
                        };
                        self.state.queue.write_buffer(
                            &self.state.ring_uniform_buffers[i],
                            0,
                            bytemuck::cast_slice(&[ring_uniforms])
                        );
                        
                        render_pass.set_pipeline(&self.state.ring_pipeline);
                        render_pass.set_bind_group(0, &self.state.ring_bind_groups[i], &[]);
                        render_pass.set_vertex_buffer(0, self.state.ring_vertex_buffer.slice(..));
                        render_pass.set_index_buffer(self.state.ring_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..self.state.ring_num_indices, 0, 0..1);
                        // Restore planet vertex/index buffers for subsequent draws
                        render_pass.set_vertex_buffer(0, self.state.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(self.state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    }
                }
            }

            // Renderizar nave
            render_pass.set_pipeline(&self.state.ship_pipeline);
            render_pass.set_bind_group(0, &self.state.ship_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.state.ship_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.state.ship_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.state.ship_num_indices, 0, 0..1);
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}