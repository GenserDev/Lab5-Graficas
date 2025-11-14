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
    let time_scale = uniforms.time * 0.5;
    
    // Patrones tropicales ondulantes
    let phi = atan2(normal.z, normal.x);
    let theta = acos(normal.y);
    
    let palm_pattern = sin(phi * 8.0 + time_scale) * sin(theta * 6.0) * 0.5 + 0.5;
    let wave_pattern = sin(phi * 12.0 - time_scale * 2.0) * 0.5 + 0.5;
    
    // Colores tropicales vibrantes
    let sunset_orange = vec3<f32>(1.0, 0.5, 0.2);
    let tropical_pink = vec3<f32>(1.0, 0.3, 0.6);
    let ocean_blue = vec3<f32>(0.0, 0.7, 1.0);
    let palm_green = vec3<f32>(0.2, 0.9, 0.4);
    let sand_yellow = vec3<f32>(1.0, 0.9, 0.3);
    
    // Combinar colores tropicales
    var color = mix(ocean_blue, tropical_pink, palm_pattern);
    color = mix(color, sunset_orange, wave_pattern * 0.5);
    color = mix(color, palm_green, sin(theta * 10.0 + time_scale) * 0.25 + 0.25);
    color = mix(color, sand_yellow, cos(phi * 6.0 - time_scale) * 0.15 + 0.15);
    
    // Iluminación suave y cálida
    let light_dir = normalize(vec3<f32>(1.0, 0.8, 0.5));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.5 + diff * 0.5);
    
    // Brillo de atardecer tropical
    let sunset_glow = sin(time_scale * 1.5) * 0.1 + 0.9;
    color = color * sunset_glow;
    
    // Atmósfera cálida
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.5);
    color = color + sunset_orange * fresnel * 0.4;
    
    return vec4<f32>(color, 1.0);
}