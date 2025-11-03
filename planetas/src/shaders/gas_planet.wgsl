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

// Función de ruido simplex 3D
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

// Función de turbulencia mejorada
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

// FBM para detalles suaves
fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < octaves; i = i + 1) {
        value = value + snoise(pos * frequency) * amplitude;
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
    
    // Posición para ruido con mayor escala temporal
    let time_scale = uniforms.time * 0.15;
    var pos = vec3<f32>(phi * 4.0, theta * 6.0, time_scale);
    
    // Crear múltiples capas de bandas con diferentes velocidades y escalas
    let band1 = snoise(vec3<f32>(pos.x * 0.4 + time_scale * 0.5, pos.y * 10.0, 0.0));
    let band2 = snoise(vec3<f32>(pos.x * 0.6 - time_scale * 0.3, pos.y * 14.0, 50.0));
    let band3 = snoise(vec3<f32>(pos.x * 0.3 + time_scale * 0.7, pos.y * 8.0, 100.0));
    let band4 = snoise(vec3<f32>(pos.x * 0.8 - time_scale * 0.4, pos.y * 18.0, 150.0));
    
    // Detalles finos para textura dentro de las bandas
    let fine_detail = fbm(vec3<f32>(pos.x * 2.0, pos.y * 20.0, time_scale * 0.2), 3);
    
    // Turbulencia para remolinos y estructuras caóticas
    let turb1 = turbulence(vec3<f32>(pos.x * 1.5, pos.y * 3.0, time_scale * 0.1), 5);
    let turb2 = turbulence(vec3<f32>(pos.x * 0.8 + 100.0, pos.y * 2.5, time_scale * 0.15), 4);
    
    // Combinar todas las bandas con pesos diferentes
    var bands = band1 * 0.35 + band2 * 0.25 + band3 * 0.2 + band4 * 0.2;
    bands = bands * 0.5 + 0.5;
    
    // Añadir turbulencia para crear zonas de tormenta
    bands = bands + (turb1 * 0.25 - 0.125);
    bands = bands + fine_detail * 0.15;
    
    // Gran mancha roja - tormenta característica (más grande y prominente)
    let storm_center = vec2<f32>(2.8, 1.4);
    let storm_dist = distance(vec2<f32>(phi + 3.14, theta), storm_center);
    
    // Forma ovalada de la tormenta
    let storm_oval = vec2<f32>(storm_dist, storm_dist * 1.5);
    let storm = smoothstep(0.6, 0.1, length(storm_oval));
    
    // Detalles internos de la tormenta
    let storm_detail = snoise(vec3<f32>(phi * 15.0, theta * 15.0, time_scale * 0.08));
    let storm_swirl = turbulence(vec3<f32>(
        (phi - storm_center.x) * 8.0, 
        (theta - storm_center.y) * 8.0, 
        time_scale * 0.1
    ), 4);
    
    let red_spot = storm * (0.6 + storm_detail * 0.2 + storm_swirl * 0.2);
    
    // Paleta de colores más rica para planeta gaseoso tipo Júpiter
    let color1 = vec3<f32>(0.92, 0.82, 0.70);  // Beige muy claro
    let color2 = vec3<f32>(0.70, 0.55, 0.40);  // Marrón claro
    let color3 = vec3<f32>(0.95, 0.88, 0.78);  // Crema brillante
    let color4 = vec3<f32>(0.58, 0.42, 0.30);  // Marrón medio
    let color5 = vec3<f32>(0.45, 0.32, 0.22);  // Marrón oscuro
    let red_storm = vec3<f32>(0.88, 0.38, 0.28); // Rojo-naranja de la mancha
    let white_zones = vec3<f32>(0.98, 0.95, 0.90); // Zonas blancas brillantes
    
    // Mezclar colores basado en las bandas con transiciones más suaves
    var color = mix(color1, color2, smoothstep(0.25, 0.45, bands));
    color = mix(color, color3, smoothstep(0.45, 0.55, bands));
    color = mix(color, color4, smoothstep(0.55, 0.70, bands));
    color = mix(color, color5, smoothstep(0.70, 0.85, bands));
    
    // Añadir zonas blancas donde hay alta turbulencia
    color = mix(color, white_zones, smoothstep(0.6, 0.9, turb2) * 0.3);
    
    // Agregar la gran mancha roja
    color = mix(color, red_storm, red_spot);
    
    // Añadir pequeñas tormentas secundarias
    let small_storm1_center = vec2<f32>(1.5, 2.0);
    let small_storm1_dist = distance(vec2<f32>(phi + 3.14, theta), small_storm1_center);
    let small_storm1 = smoothstep(0.25, 0.1, small_storm1_dist);
    color = mix(color, vec3<f32>(0.75, 0.65, 0.55), small_storm1 * 0.5);
    
    // Iluminación más dramática
    let light_dir = normalize(vec3<f32>(1.0, 0.7, 1.0));
    let diff = max(dot(normal, light_dir), 0.0);
    let ambient = 0.25;
    let lighting = ambient + diff * 0.75;
    
    color = color * lighting;
    
    // Efecto de dispersión subsuperficial (luz difusa en los bordes)
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let ndotv = abs(dot(normal, view_dir));
    let subsurface = pow(1.0 - ndotv, 3.0) * diff;
    color = color + vec3<f32>(0.6, 0.5, 0.4) * subsurface * 0.2;
    
    // Atmósfera brillante y espesa en los bordes
    let fresnel = pow(1.0 - ndotv, 2.5);
    let atmosphere = vec3<f32>(0.75, 0.65, 0.55) * fresnel * 0.5;
    
    // Añadir brillo extra en la atmósfera donde hay luz
    let atmosphere_glow = fresnel * diff * vec3<f32>(0.9, 0.8, 0.7) * 0.3;
    
    color = color + atmosphere + atmosphere_glow;
    
    // Saturación adicional para hacer los colores más vivos
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    color = mix(vec3<f32>(luminance), color, 1.15);
    
    return vec4<f32>(color, 1.0);
}