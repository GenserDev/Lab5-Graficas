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

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4<f32>) -> vec4<f32> {
    return mod289_4(((x * 34.0) + 1.0) * x);
}

fn taylorInvSqrt(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn snoise(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);
    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);
    let x1 = x0 - i1 + C.xxx;
    let x2 = x0 - i2 + C.yyy;
    let x3 = x0 - D.yyy;
    i = mod289_3(i);
    let p = permute(permute(permute(
        i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
        + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0))
        + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));
    var n_ = 0.142857142857;
    let ns = n_ * D.wyz - D.xzx;
    let j = p - 49.0 * floor(p * ns.z * ns.z);
    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);
    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1.0 - abs(x) - abs(y);
    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);
    let s0 = floor(b0) * 2.0 + 1.0;
    let s1 = floor(b1) * 2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.0));
    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;
    var p0 = vec3<f32>(a0.xy, h.x);
    var p1 = vec3<f32>(a0.zw, h.y);
    var p2 = vec3<f32>(a1.xy, h.z);
    var p3 = vec3<f32>(a1.zw, h.w);
    let norm = taylorInvSqrt(vec4<f32>(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;
    let m = max(0.6 - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    let m2 = m * m;
    return 42.0 * dot(m2 * m2, vec4<f32>(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let time_scale = uniforms.time * 2.0;
    let pos = normal * 3.0 + vec3<f32>(time_scale, time_scale * 0.7, time_scale * 0.5);
    
    // Efectos de luces de fiesta
    let lights1 = sin(pos.x * 5.0 + time_scale * 2.0) * 0.5 + 0.5;
    let lights2 = sin(pos.y * 7.0 - time_scale * 3.0) * 0.5 + 0.5;
    let lights3 = sin(pos.z * 6.0 + time_scale * 2.5) * 0.5 + 0.5;
    
    // Colores de fiesta rotando
    let rainbow_pos = fract(time_scale * 0.1 + length(pos) * 0.2);
    var color = vec3<f32>(1.0, 0.0, 0.0);
    
    if (rainbow_pos < 0.16) {
        color = mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.5, 0.0), rainbow_pos / 0.16);
    } else if (rainbow_pos < 0.33) {
        color = mix(vec3<f32>(1.0, 0.5, 0.0), vec3<f32>(1.0, 1.0, 0.0), (rainbow_pos - 0.16) / 0.17);
    } else if (rainbow_pos < 0.50) {
        color = mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), (rainbow_pos - 0.33) / 0.17);
    } else if (rainbow_pos < 0.66) {
        color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.5, 1.0), (rainbow_pos - 0.50) / 0.16);
    } else if (rainbow_pos < 0.83) {
        color = mix(vec3<f32>(0.0, 0.5, 1.0), vec3<f32>(0.5, 0.0, 1.0), (rainbow_pos - 0.66) / 0.17);
    } else {
        color = mix(vec3<f32>(0.5, 0.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), (rainbow_pos - 0.83) / 0.17);
    }
    
    // Añadir luces parpadeantes
    color = color * (0.7 + lights1 * 0.3 + lights2 * 0.3 + lights3 * 0.3);
    
    // Brillo intenso
    let pulse = sin(uniforms.time * 4.0) * 0.2 + 1.2;
    color = color * pulse;
    
    // Corona brillante
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.0);
    color = color + color * fresnel * 1.0;
    
    return vec4<f32>(color, 1.0);
}