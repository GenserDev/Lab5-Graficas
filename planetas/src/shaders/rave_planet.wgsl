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
    let time_scale = uniforms.time * 3.0;
    
    // Ondas de neón pulsantes
    let wave1 = sin(normal.x * 10.0 + time_scale) * 0.5 + 0.5;
    let wave2 = sin(normal.y * 12.0 - time_scale * 1.3) * 0.5 + 0.5;
    let wave3 = sin(normal.z * 8.0 + time_scale * 0.8) * 0.5 + 0.5;
    
    // Colores neón brillantes
    let neon_pink = vec3<f32>(1.0, 0.1, 0.8);
    let neon_cyan = vec3<f32>(0.0, 1.0, 1.0);
    let neon_purple = vec3<f32>(0.8, 0.0, 1.0);
    let neon_green = vec3<f32>(0.2, 1.0, 0.2);
    
    // Mezclar colores basado en ondas
    var color = mix(neon_pink, neon_cyan, wave1);
    color = mix(color, neon_purple, wave2 * 0.5);
    color = mix(color, neon_green, wave3 * 0.3);
    
    // Estroboscopio rápido
    let strobe = step(0.5, fract(time_scale * 4.0));
    color = color * (0.7 + strobe * 0.3);
    
    // Efecto de brillo interno
    let glow = pow(wave1 * wave2 * wave3, 0.5);
    color = color * (0.8 + glow * 0.2);
    
    // Iluminación dramática
    let light_dir = normalize(vec3<f32>(1.0, 0.5, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.5 + diff * 0.5);
    
    // Atmósfera neón brillante
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 3.0);
    color = color + neon_cyan * fresnel * 0.6;
    
    return vec4<f32>(color, 1.0);
}