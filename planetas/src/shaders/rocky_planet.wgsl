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

// Funciones de ruido
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

// FBM para terreno
fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < octaves; i = i + 1) {
        value = value + amplitude * snoise(pos * frequency);
        frequency = frequency * 2.0;
        amplitude = amplitude * 0.5;
    }
    
    return value;
}

// Función de Voronoi para cráteres
fn voronoi(p: vec3<f32>) -> vec2<f32> {
    let n = floor(p);
    let f = fract(p);
    
    var min_dist = 8.0;
    var min_dist2 = 8.0;
    
    for (var k = -1; k <= 1; k = k + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            for (var i = -1; i <= 1; i = i + 1) {
                let g = vec3<f32>(f32(i), f32(j), f32(k));
                let o = vec3<f32>(
                    snoise((n + g) * 0.5) * 0.5 + 0.5,
                    snoise((n + g) * 0.5 + vec3<f32>(43.0, 17.0, 31.0)) * 0.5 + 0.5,
                    snoise((n + g) * 0.5 + vec3<f32>(13.0, 27.0, 53.0)) * 0.5 + 0.5
                );
                let r = g + o - f;
                let d = dot(r, r);
                
                if (d < min_dist) {
                    min_dist2 = min_dist;
                    min_dist = d;
                } else if (d < min_dist2) {
                    min_dist2 = d;
                }
            }
        }
    }
    
    return vec2<f32>(sqrt(min_dist), sqrt(min_dist2));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let pos = normal * 5.0;
    
    // Generar terreno con FBM
    let terrain = fbm(pos * 2.0, 6) * 0.5 + 0.5;
    
    // Cráteres usando Voronoi
    let craters = voronoi(pos * 3.0);
    let crater_depth = smoothstep(0.1, 0.0, craters.x) * 0.5;
    let crater_rim = smoothstep(0.15, 0.1, craters.x) * smoothstep(0.1, 0.12, craters.x);
    
    // Detalle de superficie rocosa
    let detail1 = fbm(pos * 10.0, 3) * 0.5 + 0.5;
    let detail2 = fbm(pos * 25.0, 2) * 0.5 + 0.5;
    
    // Combinar características del terreno
    var height = terrain * 0.6 + detail1 * 0.3 + detail2 * 0.1;
    height = height - crater_depth;
    height = height + crater_rim * 0.3;
    
    // Colores rocosos (tipo Marte/Luna)
    let rock_dark = vec3<f32>(0.35, 0.25, 0.20);
    let rock_mid = vec3<f32>(0.55, 0.40, 0.30);
    let rock_light = vec3<f32>(0.75, 0.60, 0.45);
    let crater_color = vec3<f32>(0.25, 0.20, 0.18);
    let dust = vec3<f32>(0.65, 0.50, 0.35);
    
    // Mezclar colores basado en altura y características
    var color = mix(rock_dark, rock_mid, smoothstep(0.3, 0.6, height));
    color = mix(color, rock_light, smoothstep(0.6, 0.8, height));
    color = mix(color, dust, detail2 * 0.4);
    
    // Oscurecer cráteres
    color = mix(crater_color, color, smoothstep(0.5, 0.0, crater_depth));
    
    // Iluminar bordes de cráteres
    color = color + vec3<f32>(0.3, 0.25, 0.20) * crater_rim;
    
    // Agregar variación de color mineral
    let mineral = snoise(pos * 15.0) * 0.5 + 0.5;
    let mineral_color = vec3<f32>(0.6, 0.35, 0.25);
    color = mix(color, mineral_color, mineral * 0.2);
    
    // Iluminación
    let light_dir = normalize(vec3<f32>(1.0, 0.5, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;
    let lighting = ambient + diff * 0.7;
    
    color = color * lighting;
    
    // Especular sutil para minerales
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    color = color + vec3<f32>(0.3, 0.3, 0.3) * spec * mineral * 0.3;
    
    // Atmósfera muy tenue
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 4.0);
    let atmosphere = vec3<f32>(0.6, 0.45, 0.35) * fresnel * 0.15;
    color = color + atmosphere;
    
    return vec4<f32>(color, 1.0);
}