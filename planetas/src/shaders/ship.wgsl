struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_position = uniforms.view_proj * world_pos;
    out.normal = normalize(input.position);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    
    // Color base metálico plateado
    let metal_color = vec3<f32>(0.8, 0.85, 0.9);
    
    // Luces de neón en la nave
    let neon_glow = sin(uniforms.time * 5.0) * 0.5 + 0.5;
    let accent_color = vec3<f32>(0.0, 0.8, 1.0) * neon_glow;
    
    // Iluminación
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    var color = metal_color * (0.3 + diff * 0.7);
    
    // Especular metálico
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    color = color + vec3<f32>(1.0, 1.0, 1.0) * spec * 0.5;
    
    // Añadir acentos de neón
    color = color + accent_color * 0.2;
    
    return vec4<f32>(color, 1.0);
}