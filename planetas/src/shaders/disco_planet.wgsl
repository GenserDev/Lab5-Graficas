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
    
    // Crear patrón de bola de discoteca (espejos cuadrados)
    let phi = atan2(normal.z, normal.x);
    let theta = acos(normal.y);
    
    let mirror_size = 0.3;
    let grid_x = floor(phi * 8.0 / mirror_size);
    let grid_y = floor(theta * 8.0 / mirror_size);
    
    // Patrón de tablero de espejos
    let checker = fract((grid_x + grid_y) * 0.5) * 2.0;
    
    // Luces de colores rotando
    let light_angle = time_scale * 2.0 + grid_x * 0.5 + grid_y * 0.3;
    let light_color = vec3<f32>(
        sin(light_angle) * 0.5 + 0.5,
        sin(light_angle + 2.094) * 0.5 + 0.5,
        sin(light_angle + 4.189) * 0.5 + 0.5
    );
    
    // Base plateada/espejo
    let mirror_base = vec3<f32>(0.9, 0.9, 1.0);
    var color = mix(mirror_base * 0.5, mirror_base, checker);
    
    // Añadir reflejos de colores
    color = color * (0.6 + light_color * 0.4);
    
    // Iluminación
    let light_dir = normalize(vec3<f32>(1.0, 0.7, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.4 + diff * 0.6);
    
    // Especular brillante (efecto espejo)
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);
    color = color + vec3<f32>(1.0, 1.0, 1.0) * spec * 0.8;
    
    // Brillo pulsante
    let pulse = sin(time_scale * 3.0) * 0.1 + 0.9;
    color = color * pulse;
    
    return vec4<f32>(color, 1.0);
}