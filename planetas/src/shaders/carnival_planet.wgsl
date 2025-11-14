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
    
    // Patrones de confeti y serpentinas
    let phi = atan2(normal.z, normal.x);
    let theta = acos(normal.y);
    
    let confetti1 = sin(phi * 20.0 + time_scale * 2.0) * sin(theta * 15.0 - time_scale);
    let confetti2 = cos(phi * 25.0 - time_scale * 1.5) * cos(theta * 18.0 + time_scale * 2.0);
    let streamers = sin(phi * 10.0 + theta * 12.0 - time_scale * 3.0);
    
    // Colores vibrantes de carnaval
    let bright_yellow = vec3<f32>(1.0, 0.95, 0.0);
    let bright_red = vec3<f32>(1.0, 0.1, 0.2);
    let bright_blue = vec3<f32>(0.0, 0.6, 1.0);
    let bright_green = vec3<f32>(0.2, 1.0, 0.3);
    let bright_purple = vec3<f32>(0.9, 0.2, 1.0);
    let bright_orange = vec3<f32>(1.0, 0.6, 0.0);
    
    // Crear patrón de confeti multicolor
    var color = bright_yellow;
    
    if (confetti1 > 0.6) {
        color = bright_red;
    } else if (confetti1 > 0.2) {
        color = bright_blue;
    } else if (confetti1 > -0.2) {
        color = bright_green;
    } else if (confetti1 > -0.6) {
        color = bright_purple;
    } else {
        color = bright_orange;
    }
    
    // Añadir serpentinas
    if (abs(streamers) > 0.7) {
        color = mix(color, bright_yellow, 0.5);
    }
    
    // Efecto de brillo de confeti
    let sparkle = step(0.85, confetti2) * 0.5;
    color = color + vec3<f32>(1.0, 1.0, 1.0) * sparkle;
    
    // Animación de celebración
    let celebration = sin(time_scale * 4.0) * 0.15 + 0.85;
    color = color * celebration;
    
    // Iluminación festiva
    let light_dir = normalize(vec3<f32>(1.0, 0.7, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.5 + diff * 0.5);
    
    // Atmósfera de fiesta brillante
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.5);
    color = color + bright_yellow * fresnel * 0.3;
    
    return vec4<f32>(color, 1.0);
}