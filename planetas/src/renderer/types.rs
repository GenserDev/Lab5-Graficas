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
    pub sun_pipeline: wgpu::RenderPipeline,
    pub gas_planet_pipeline: wgpu::RenderPipeline,
    pub rocky_planet_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub uniform_buffers: Vec<wgpu::Buffer>,
    pub uniform_bind_groups: Vec<wgpu::BindGroup>,
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

        let (vertices, indices) = super::geometry::create_sphere(2.0, 64, 64);
        
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

        // Crear un buffer y bind group para cada planeta (5 planetas en total)
        let num_planets = 5;
        let mut uniform_buffers = Vec::new();
        let mut uniform_bind_groups = Vec::new();

        for i in 0..num_planets {
            let uniforms = Uniforms {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                model: glam::Mat4::IDENTITY.to_cols_array_2d(),
                time: 0.0,
                _padding: [0.0; 3],
            };

            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Uniform Buffer {}", i)),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some(&format!("uniform_bind_group {}", i)),
            });

            uniform_buffers.push(uniform_buffer);
            uniform_bind_groups.push(uniform_bind_group);
        }

        let (sun_pipeline, gas_planet_pipeline, rocky_planet_pipeline) = 
            super::pipelines::create_all_pipelines(&device, &uniform_bind_group_layout, config.format);

        Self {
            surface,
            device,
            queue,
            config,
            sun_pipeline,
            gas_planet_pipeline,
            rocky_planet_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffers,
            uniform_bind_groups,
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