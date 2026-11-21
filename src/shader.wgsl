struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    var_count: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> vars: array<f32>;
@group(0) @binding(2) var<storage, read> camera: array<f32>;

// --- Render settings (were uniforms in the other version) ---
const MAX_STEPS: i32 = 100;
const STEP_SCALE: f32 = 0.7;
const MAX_DIST: f32 = 100.0;
const FOCAL: f32 = 1.5;   // larger = narrower field of view

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(positions[vid], 0.0, 1.0);
    return out;
}

// --- Math helpers used by generated expressions ---

fn fmod(a: f32, b: f32) -> f32 {
    let denom = select(b, select(1e-9, -1e-9, b < 0.0), abs(b) < 1e-9);
    return a - denom * floor(a / denom);
}

fn safe_pow(base: f32, exp: f32) -> f32 {
    let r = round(exp);
    if (abs(exp - r) < 1e-4) {
        let n = i32(r);
        if (n == 0) { return 1.0; }
        if (n == 1) { return base; }
        if (n == 2) { return base * base; }
        if (n == 3) { return base * base * base; }
        let mag = pow(abs(base), f32(n));
        return select(mag, -mag, base < 0.0 && (n & 1) != 0);
    }
    return pow(max(base, 0.0), exp);
}

fn smin(a: f32, b: f32, k_raw: f32) -> f32 {
    let k = max(k_raw, 1e-6);
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn smax(a: f32, b: f32, k_raw: f32) -> f32 {
    let k = max(k_raw, 1e-6);
    let h = clamp(0.5 + 0.5 * (a - b) / k, 0.0, 1.0);
    return mix(a, b, h) + k * h * (1.0 - h);
}

// {{EVAL_SDF}}

fn sdf_scene(p: vec3<f32>) -> f32 {
    return eval_sdf(p);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let h = 0.002;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * sdf_scene(p + k.xyy * h) +
        k.yyx * sdf_scene(p + k.yyx * h) +
        k.yxy * sdf_scene(p + k.yxy * h) +
        k.xxx * sdf_scene(p + k.xxx * h)
    );
}

// Safe camera read: the buffer may be shorter than expected (Rust pads empty to [0.0])
fn cam(i: u32, fallback: f32) -> f32 {
    return select(fallback, camera[i], arrayLength(&camera) > i);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // Pixel -> aspect-corrected coords, +Y up
    let uv = vec2<f32>(
        frag_coord.x * 2.0 - uniforms.resolution.x,
        uniforms.resolution.y - frag_coord.y * 2.0
    ) / uniforms.resolution.y;

    // camera = [x, y, z, yaw, pitch]; yaw/pitch default to 0 (looking down +Z)
    let ro = vec3<f32>(cam(0u, 0.0), cam(1u, 0.0), cam(2u, -3.0));
    let yaw = cam(3u, 0.0);
    let pitch = clamp(cam(4u, 0.0), -1.5, 1.5);

    let forward = vec3<f32>(
        sin(yaw) * cos(pitch),
        sin(pitch),
        cos(yaw) * cos(pitch)
    );
    let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), forward));
    let up = cross(forward, right);
    let rd = normalize(forward * FOCAL + right * uv.x + up * uv.y);

    var t = 0.0;
    var hit = false;

    for (var i = 0; i < MAX_STEPS; i++) {
        let p = ro + rd * t;
        let d = sdf_scene(p);
        let eps = 0.0005 * (1.0 + t);
        if (d < eps) {
            hit = true;
            break;
        }
        t += max(d * STEP_SCALE, 0.0005);
        if (t >= MAX_DIST) { break; }
    }

    if (!hit) {
        let sky_t = clamp(rd.y * 0.5 + 0.5, 0.0, 1.0);
        let bg = mix(vec3<f32>(0.05, 0.06, 0.09), vec3<f32>(0.14, 0.18, 0.26), sky_t);
        return vec4<f32>(bg, 1.0);
    }

    let hit_pos = ro + rd * t;
    let n = calc_normal(hit_pos);
    let light_dir = normalize(vec3<f32>(0.6, 0.8, -0.5));
    let diff = max(dot(n, light_dir), 0.0);
    let base_color = vec3<f32>(0.78, 0.82, 0.88);
    return vec4<f32>(base_color * (0.18 + diff * 0.82), 1.0);
}