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
    
    // Superficie de la luna con colores pastel de fiesta
    let base_color = vec3<f32>(0.85, 0.75, 0.95);  // Lavanda suave
    
    // Cráteres con patrón simple
    let crater_pattern = sin(normal.x * 15.0) * sin(normal.y * 15.0) * sin(normal.z * 15.0);
    let crater_color = vec3<f32>(0.65, 0.55, 0.75);
    
    var color = mix(crater_color, base_color, crater_pattern * 0.5 + 0.5);
    
    // Brillo mágico pulsante
    let magic_glow = sin(uniforms.time * 2.0) * 0.1 + 0.9;
    color = color * magic_glow;
    
    // Iluminación
    let light_dir = normalize(vec3<f32>(1.0, 0.5, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    color = color * (0.4 + diff * 0.6);
    
    // Brillo sutil en los bordes
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 3.0);
    color = color + vec3<f32>(0.9, 0.8, 1.0) * fresnel * 0.3;
    
    return vec4<f32>(color, 1.0);
}