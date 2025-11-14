use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub time: f32,
    pub _padding: [f32; 3],
}

pub struct RendererState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub planet_pipelines: Vec<wgpu::RenderPipeline>,
    pub moon_pipeline: wgpu::RenderPipeline,
    pub ship_pipeline: wgpu::RenderPipeline,
    pub skybox_pipeline: wgpu::RenderPipeline,
    pub ring_pipeline: wgpu::RenderPipeline,
    pub orbit_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub ship_vertex_buffer: wgpu::Buffer,
    pub ship_index_buffer: wgpu::Buffer,
    pub ship_num_indices: u32,
    pub skybox_vertex_buffer: wgpu::Buffer,
    pub skybox_index_buffer: wgpu::Buffer,
    pub skybox_num_indices: u32,
    pub ring_vertex_buffer: wgpu::Buffer,
    pub ring_index_buffer: wgpu::Buffer,
    pub ring_num_indices: u32,
    pub orbit_vertex_buffer: wgpu::Buffer,
    pub orbit_index_buffer: wgpu::Buffer,
    pub planet_uniform_buffers: Vec<wgpu::Buffer>,
    pub planet_bind_groups: Vec<wgpu::BindGroup>,
    pub moon_uniform_buffers: Vec<wgpu::Buffer>,
    pub moon_bind_groups: Vec<wgpu::BindGroup>,
    pub ring_uniform_buffers: Vec<wgpu::Buffer>,
    pub ring_bind_groups: Vec<wgpu::BindGroup>,
    pub orbit_bind_groups: Vec<wgpu::BindGroup>,
    pub orbit_ranges: Vec<(u32, u32)>,
    pub ship_uniform_buffer: wgpu::Buffer,
    pub ship_bind_group: wgpu::BindGroup,
    pub skybox_uniform_buffer: wgpu::Buffer,
    pub skybox_bind_group: wgpu::BindGroup,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl RendererState {
    pub async fn new(window: Arc<winit::window::Window>, size: winit::dpi::PhysicalSize<u32>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: Default::default(),
        }, None).await.unwrap();

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

        let depth_texture = Self::create_depth_texture(&device, size);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Crear geometrías
        let (vertices, indices) = super::geometry::create_sphere(2.0, 64, 64);
        println!("[debug] sphere: verts={} indices={}", vertices.len(), indices.len());
        if !indices.is_empty() {
            println!("[debug] sphere_indices sample: {:?}", &indices[0..indices.len().min(20)]);
            if let Some(max_idx) = indices.iter().max() {
                println!("[debug] sphere_indices max value = {}", max_idx);
            }
        }
        let (ship_vertices, ship_indices) = super::ship::create_ship();
        println!("[debug] ship: verts={} indices={}", ship_vertices.len(), ship_indices.len());
        let (skybox_vertices, skybox_indices) = super::skybox::create_skybox();
        println!("[debug] skybox: verts={} indices={}", skybox_vertices.len(), skybox_indices.len());
        let (ring_vertices, ring_indices) = super::geometry::create_ring(1.0, 2.0, 64);
        println!("[debug] ring: verts={} indices={}", ring_vertices.len(), ring_indices.len());
        if !ring_indices.is_empty() {
            println!("[debug] ring_indices sample: {:?}", &ring_indices[0..ring_indices.len().min(20)]);
        }
        
        // Buffers de esfera
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        // Buffers de nave
        let ship_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ship Vertex Buffer"),
            contents: bytemuck::cast_slice(&ship_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ship_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ship Index Buffer"),
            contents: bytemuck::cast_slice(&ship_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let ship_num_indices = ship_indices.len() as u32;

        // Buffers de skybox
        let skybox_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skybox Vertex Buffer"),
            contents: bytemuck::cast_slice(&skybox_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let skybox_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skybox Index Buffer"),
            contents: bytemuck::cast_slice(&skybox_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let skybox_num_indices = skybox_indices.len() as u32;

        // Buffers de anillos
        let ring_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ring Vertex Buffer"),
            contents: bytemuck::cast_slice(&ring_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ring_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ring Index Buffer"),
            contents: bytemuck::cast_slice(&ring_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let ring_num_indices = ring_indices.len() as u32;

        // Crear órbitas para cada planeta
        let planets = super::planets::create_planet_system();
        let mut orbit_vertices = Vec::new();
        let mut orbit_indices = Vec::new();
        let mut orbit_ranges = Vec::new();

        for planet in &planets {
            if planet.orbit_radius > 0.0 {
                let (orbit_verts, orbit_inds) = super::geometry::create_orbit(
                    planet.orbit_radius,
                    100,
                    planet.orbit_inclination,
                );

                let base_vertex = orbit_vertices.len() as u32;
                let base_index = orbit_indices.len() as u32;

                orbit_vertices.extend(orbit_verts);

                // map vertex indices and push into index buffer
                for idx in orbit_inds.iter() {
                    orbit_indices.push(idx + base_vertex);
                }

                let count = (orbit_inds.len()) as u32;
                orbit_ranges.push((base_index, count));
            } else {
                // Para el sol, no agregar indices (no se renderiza órbita)
                orbit_ranges.push((0, 0));
            }
        }

        let orbit_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Orbit Vertex Buffer"),
            contents: bytemuck::cast_slice(&orbit_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let orbit_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Orbit Index Buffer"),
            contents: bytemuck::cast_slice(&orbit_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Debug prints to verify orbit buffers
        println!("[debug] orbit_vertices.len() = {}", orbit_vertices.len());
        println!("[debug] orbit_indices.len() = {}", orbit_indices.len());
        if !orbit_indices.is_empty() {
            println!("[debug] orbit_indices sample: {:?}", &orbit_indices[0..orbit_indices.len().min(20)]);
        }


        // Crear bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        // Crear pipelines
        let planet_pipelines = super::pipelines::create_all_planet_pipelines(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        let moon_pipeline = super::pipelines::create_moon_pipeline(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        let ship_pipeline = super::pipelines::create_ship_pipeline(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        let skybox_pipeline = super::pipelines::create_skybox_pipeline(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        let ring_pipeline = super::pipelines::create_ring_pipeline(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        let orbit_pipeline = super::pipelines::create_orbit_pipeline(
            &device,
            &uniform_bind_group_layout,
            config.format,
        );

        // Crear uniformes y bind groups para planetas
        let num_planets = planets.len();
        let mut planet_uniform_buffers = Vec::new();
        let mut planet_bind_groups = Vec::new();
        let mut moon_uniform_buffers = Vec::new();
        let mut moon_bind_groups = Vec::new();
        let mut ring_uniform_buffers = Vec::new();
        let mut ring_bind_groups = Vec::new();
        let mut orbit_bind_groups = Vec::new();

        for i in 0..num_planets {
            let uniforms = Uniforms {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                time: 0.0,
                _padding: [0.0; 3],
            };

            // Planet uniform
            let planet_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Planet Uniform Buffer {}", i)),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let planet_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: planet_uniform_buffer.as_entire_binding(),
                }],
                label: Some(&format!("planet_bind_group {}", i)),
            });

            planet_uniform_buffers.push(planet_uniform_buffer);
            planet_bind_groups.push(planet_bind_group);

            // Moon uniform
            let moon_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Moon Uniform Buffer {}", i)),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let moon_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: moon_uniform_buffer.as_entire_binding(),
                }],
                label: Some(&format!("moon_bind_group {}", i)),
            });

            moon_uniform_buffers.push(moon_uniform_buffer);
            moon_bind_groups.push(moon_bind_group);

            // Ring uniform
            let ring_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Ring Uniform Buffer {}", i)),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let ring_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ring_uniform_buffer.as_entire_binding(),
                }],
                label: Some(&format!("ring_bind_group {}", i)),
            });

            ring_uniform_buffers.push(ring_uniform_buffer);
            ring_bind_groups.push(ring_bind_group);

            // Orbit bind group (usa el mismo buffer que el planeta)
            let orbit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: planet_uniform_buffers[i].as_entire_binding(),
                }],
                label: Some(&format!("orbit_bind_group {}", i)),
            });

            orbit_bind_groups.push(orbit_bind_group);
        }

        // Uniformes de nave
        let ship_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ship Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                time: 0.0,
                _padding: [0.0; 3],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ship_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ship_uniform_buffer.as_entire_binding(),
            }],
            label: Some("ship_bind_group"),
        });

        // Uniformes de skybox
        let skybox_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skybox Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                time: 0.0,
                _padding: [0.0; 3],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let skybox_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: skybox_uniform_buffer.as_entire_binding(),
            }],
            label: Some("skybox_bind_group"),
        });

        Self {
            surface,
            device,
            queue,
            config,
            planet_pipelines,
            moon_pipeline,
            ship_pipeline,
            skybox_pipeline,
            ring_pipeline,
            orbit_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            ship_vertex_buffer,
            ship_index_buffer,
            ship_num_indices,
            skybox_vertex_buffer,
            skybox_index_buffer,
            skybox_num_indices,
            ring_vertex_buffer,
            ring_index_buffer,
            ring_num_indices,
            orbit_vertex_buffer,
            orbit_index_buffer,
            orbit_ranges,
            planet_uniform_buffers,
            planet_bind_groups,
            moon_uniform_buffers,
            moon_bind_groups,
            ring_uniform_buffers,
            ring_bind_groups,
            orbit_bind_groups,
            ship_uniform_buffer,
            ship_bind_group,
            skybox_uniform_buffer,
            skybox_bind_group,
            depth_texture,
            depth_view,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        
        self.depth_texture = Self::create_depth_texture(&self.device, new_size);
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }

    fn create_depth_texture(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}