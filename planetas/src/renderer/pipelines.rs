use super::types::Vertex;

pub fn create_all_pipelines(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    color_format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::RenderPipeline, wgpu::RenderPipeline) {
    let sun_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sun Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sun.wgsl").into()),
    });

    let gas_planet_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Gas Planet Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gas_planet.wgsl").into()),
    });

    let rocky_planet_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Rocky Planet Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/rocky_planet.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let sun_pipeline = create_pipeline(device, &pipeline_layout, color_format, &sun_shader);
    let gas_planet_pipeline = create_pipeline(device, &pipeline_layout, color_format, &gas_planet_shader);
    let rocky_planet_pipeline = create_pipeline(device, &pipeline_layout, color_format, &rocky_planet_shader);

    (sun_pipeline, gas_planet_pipeline, rocky_planet_pipeline)
}

fn create_pipeline(
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}