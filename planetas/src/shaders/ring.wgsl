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
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_position = uniforms.view_proj * world_pos;
    
    // Calcular UV basado en la distancia del centro
    let dist = length(input.position.xz);
    out.uv = vec2<f32>(dist, atan2(input.position.z, input.position.x));
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Anillos con bandas de colores brillantes
    let ring_pattern = sin(in.uv.x * 20.0 + uniforms.time) * 0.5 + 0.5;
    let rotation = in.uv.y + uniforms.time * 0.5;
    let spiral = sin(rotation * 10.0) * 0.5 + 0.5;
    
    // Colores neón para los anillos
    let color1 = vec3<f32>(1.0, 0.0, 0.8);  // Rosa neón
    let color2 = vec3<f32>(0.0, 1.0, 1.0);  // Cyan neón
    let color3 = vec3<f32>(0.5, 0.0, 1.0);  // Púrpura
    
    var color = mix(color1, color2, ring_pattern);
    color = mix(color, color3, spiral * 0.5);
    
    // Transparencia variable
    let alpha = ring_pattern * 0.6 + 0.3;
    
    // Brillo pulsante
    let pulse = sin(uniforms.time * 2.0) * 0.2 + 0.8;
    color = color * pulse;
    
    return vec4<f32>(color, alpha);
}