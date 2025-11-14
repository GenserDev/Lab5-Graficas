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
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = vec4<f32>(input.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Líneas de órbita sutiles
    let orbit_color = vec3<f32>(0.3, 0.6, 0.8);
    let alpha = 0.3;
    
    return vec4<f32>(orbit_color, alpha);
}