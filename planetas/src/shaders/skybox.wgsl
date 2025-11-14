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
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_position = uniforms.view_proj * world_pos;
    return out;
}

fn hash(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_pos);
    
    // Color base del espacio
    let space_color = vec3<f32>(0.01, 0.01, 0.05);
    var color = space_color;
    
    // Generar estrellas usando hash
    let star_density = 400.0;
    let star_pos = floor(dir * star_density);
    let star_hash = hash(star_pos);
    
    // Crear estrellas
    if (star_hash > 0.996) {
        let star_brightness = star_hash * 2.0;
        let star_color = mix(
            vec3<f32>(1.0, 1.0, 1.0),
            vec3<f32>(0.8, 0.9, 1.0),
            star_hash
        );
        color = color + star_color * star_brightness;
    }
    
    // Estrellas parpadeantes
    let twinkle = sin(star_hash * 100.0 + uniforms.time * 3.0) * 0.5 + 0.5;
    if (star_hash > 0.996) {
        color = color * (0.7 + twinkle * 0.3);
    }
    
    // Nebulosa de colores sutiles
    let nebula = sin(dir.x * 2.0 + uniforms.time * 0.1) * 
                 sin(dir.y * 2.0 - uniforms.time * 0.15) * 
                 sin(dir.z * 2.0 + uniforms.time * 0.12);
    let nebula_color = vec3<f32>(0.3, 0.1, 0.4) * max(nebula, 0.0) * 0.15;
    color = color + nebula_color;
    
    return vec4<f32>(color, 1.0);
}