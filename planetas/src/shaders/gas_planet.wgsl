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

// Función de ruido simplex 3D (reutilizamos las funciones)
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

// Función de turbulencia
fn turbulence(p: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < octaves; i = i + 1) {
        value = value + abs(snoise(pos * frequency)) * amplitude;
        frequency = frequency * 2.0;
        amplitude = amplitude * 0.5;
    }
    
    return value;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    
    // Coordenadas esféricas para bandas horizontales
    let phi = atan2(normal.z, normal.x);
    let theta = acos(normal.y);
    
    // Posición para ruido
    var pos = vec3<f32>(phi * 3.0, theta * 5.0, uniforms.time * 0.1);
    
    // Crear bandas con diferentes velocidades
    let band1 = snoise(vec3<f32>(pos.x * 0.5 + uniforms.time * 0.3, pos.y * 8.0, 0.0));
    let band2 = snoise(vec3<f32>(pos.x * 0.7 - uniforms.time * 0.2, pos.y * 12.0, 50.0));
    let band3 = snoise(vec3<f32>(pos.x * 0.3 + uniforms.time * 0.4, pos.y * 6.0, 100.0));
    
    // Turbulencia para remolinos
    let turb = turbulence(vec3<f32>(pos.x, pos.y * 2.0, uniforms.time * 0.05), 4);
    
    // Combinar bandas
    var bands = (band1 + band2 * 0.7 + band3 * 0.5) * 0.5 + 0.5;
    bands = bands + turb * 0.2;
    
    // Gran mancha roja - tormenta característica
    let storm_center = vec2<f32>(3.14, 1.5);
    let storm_dist = distance(vec2<f32>(phi + 3.14, theta), storm_center);
    let storm = smoothstep(0.5, 0.2, storm_dist);
    let storm_detail = snoise(vec3<f32>(phi * 10.0, theta * 10.0, uniforms.time * 0.05));
    let red_spot = storm * (0.5 + storm_detail * 0.5);
    
    // Colores del planeta gaseoso (tipo Júpiter)
    let color1 = vec3<f32>(0.85, 0.75, 0.65); // Beige claro
    let color2 = vec3<f32>(0.65, 0.50, 0.35); // Marrón
    let color3 = vec3<f32>(0.90, 0.85, 0.75); // Crema
    let color4 = vec3<f32>(0.55, 0.40, 0.30); // Marrón oscuro
    let red_storm = vec3<f32>(0.85, 0.35, 0.25); // Rojo de la mancha
    
    // Mezclar colores basado en las bandas
    var color = mix(color1, color2, smoothstep(0.3, 0.5, bands));
    color = mix(color, color3, smoothstep(0.5, 0.7, bands));
    color = mix(color, color4, smoothstep(0.7, 0.9, bands));
    
    // Agregar la gran mancha roja
    color = mix(color, red_storm, red_spot);
    
    // Iluminación simple
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diff = max(dot(normal, light_dir), 0.2);
    color = color * diff;
    
    // Efecto atmosférico en los bordes
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.0);
    let atmosphere = vec3<f32>(0.7, 0.6, 0.5) * fresnel * 0.3;
    color = color + atmosphere;
    
    return vec4<f32>(color, 1.0);
}