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
    let time_scale = uniforms.time;
    
    // Líneas neón en espiral
    let phi = atan2(normal.z, normal.x);
    let theta = acos(normal.y);
    
    let spiral = sin((phi * 15.0 + theta * 20.0 - time_scale * 4.0));
    let grid = sin(phi * 30.0) * sin(theta * 30.0);
    
    // Tubos neón brillantes
    let neon_tubes = step(0.7, abs(spiral)) + step(0.8, abs(grid));
    
    // Paleta neón cyberpunk
    let electric_blue = vec3<f32>(0.0, 0.5, 1.0);
    let hot_pink = vec3<f32>(1.0, 0.0, 0.8);
    let laser_green = vec3<f32>(0.0, 1.0, 0.5);
    let purple_glow = vec3<f32>(0.7, 0.0, 1.0);
    
    // Ciclo de colores
    let color_cycle = fract(time_scale * 0.3 + length(normal) * 0.5);
    var base_color = mix(electric_blue, hot_pink, color_cycle);
    base_color = mix(base_color, laser_green, sin(color_cycle * 6.28) * 0.5 + 0.5);
    
    // Aplicar tubos neón
    var color = base_color * 0.3;
    color = mix(color, purple_glow, neon_tubes * 0.8);
    
    // Pulso de energía
    let energy_pulse = sin(time_scale * 5.0 + length(normal) * 10.0) * 0.2 + 0.8;
    color = color * energy_pulse;
    
    // Iluminación
    let light_dir = normalize(vec3<f32>(1.0, 0.5, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.4 + diff * 0.6);
    
    // Brillo neón intenso
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.0);
    color = color + hot_pink * fresnel * 0.7;
    
    // Aumentar brillo general
    color = color * 1.3;
    
    return vec4<f32>(color, 1.0);
}