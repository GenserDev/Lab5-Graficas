use wgpu::util::DeviceExt;
use std::sync::Arc;
use winit::keyboard::KeyCode;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    time: f32,
    _padding: [f32; 3],
}

pub struct Renderer {
    pub window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    sun_pipeline: wgpu::RenderPipeline,
    gas_planet_pipeline: wgpu::RenderPipeline,
    rocky_planet_pipeline: wgpu::RenderPipeline,
    
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    
    camera: Camera,
    time: f32,
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
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Crear esfera
        let (vertices, indices) = create_sphere(2.0, 64, 64);
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = indices.len() as u32;

        let camera = Camera::new(size.width, size.height);

        let uniforms = Uniforms {
            view_proj: camera.build_view_projection_matrix().to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
            time: 0.0,
            _padding: [0.0; 3],
        };

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // Shaders
        let sun_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sun Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sun.wgsl").into()),
        });

        let gas_planet_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gas Planet Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/gas_planet.wgsl").into()),
        });

        let rocky_planet_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rocky Planet Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rocky_planet.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sun_pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &sun_shader,
        );

        let gas_planet_pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &gas_planet_shader,
        );

        let rocky_planet_pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &rocky_planet_shader,
        );

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            sun_pipeline,
            gas_planet_pipeline,
            rocky_planet_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffer,
            uniform_bind_group,
            camera,
            time: 0.0,
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
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
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

        // Movimiento de la cámara
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

        // Rotación de la cámara
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
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Renderizar el Sol en el centro
            render_pass.set_pipeline(&self.sun_pipeline);
            let sun_uniforms = Uniforms {
                view_proj: self.camera.build_view_projection_matrix().to_cols_array_2d(),
                model: (Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)) 
                    * Mat4::from_rotation_y(self.time * 0.3)).to_cols_array_2d(),
                time: self.time,
                _padding: [0.0; 3],
            };
            self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[sun_uniforms]));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // Renderizar el Planeta Gaseoso
            render_pass.set_pipeline(&self.gas_planet_pipeline);
            let gas_uniforms = Uniforms {
                view_proj: self.camera.build_view_projection_matrix().to_cols_array_2d(),
                model: (Mat4::from_translation(Vec3::new(12.0, 0.0, 0.0)) 
                    * Mat4::from_rotation_y(self.time * 0.5) 
                    * Mat4::from_scale(Vec3::splat(0.8))).to_cols_array_2d(),
                time: self.time,
                _padding: [0.0; 3],
            };
            self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[gas_uniforms]));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // Renderizar el Planeta Rocoso
            render_pass.set_pipeline(&self.rocky_planet_pipeline);
            let rocky_uniforms = Uniforms {
                view_proj: self.camera.build_view_projection_matrix().to_cols_array_2d(),
                model: (Mat4::from_translation(Vec3::new(-12.0, 0.0, 0.0)) 
                    * Mat4::from_rotation_y(self.time * 0.7) 
                    * Mat4::from_scale(Vec3::splat(0.6))).to_cols_array_2d(),
                time: self.time,
                _padding: [0.0; 3],
            };
            self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[rocky_uniforms]));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    let stack_step = std::f32::consts::PI / stacks as f32;

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(Vertex {
                position: [x, y, z],
            });
        }
    }

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _ in 0..sectors {
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            if i != (stacks - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    (vertices, indices)
}