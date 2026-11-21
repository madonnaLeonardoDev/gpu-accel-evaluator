const NUM:u32 = 0u;
const VAR:u32 = 1u;
const MINUS:u32 = 2u;
const ABSOLUTE:u32 = 3u;
const COS:u32 = 4u;
const SIN:u32 = 5u;
const TAN:u32 = 6u;
const LN:u32 = 7u;
const LOG2:u32 = 8u;
const SQUAREROOT:u32 = 9u;
const ADD:u32 = 10u;
const SUBTRACT:u32 = 11u;
const MULTIPLY:u32 = 12u;
const DIVIDE:u32 = 13u;
const POWER:u32 = 14u;
const MOD:u32 = 15u;
const SMOOTHMIN:u32 = 16u;
const SMOOTHMAX:u32 = 17u;
const MIX:u32 = 18u;
const CLAMP:u32 = 19u;

const MAX_VARS: u32 = 32u;
const STACK_SIZE: u32 = 32u;
const MAX_STEPS: i32 = 200;
const STEP_SCALE: f32 = 0.7;
const MAX_DIST: f32 = 100.0;

// ---- Error flags (bitmask) ----
// Hard errors: the bytecode itself is broken (same result for every pixel)
const ERR_OVERFLOW:  u32 = 1u;     // stack overflow
const ERR_UNDERFLOW: u32 = 2u;     // op needed more operands than the stack had
const ERR_BAD_OP:    u32 = 4u;     // unknown opcode
const ERR_TRUNCATED: u32 = 8u;     // NUM/VAR at end of bytecode with no operand
const ERR_BAD_VAR:   u32 = 16u;    // VAR index out of range
const ERR_EMPTY:     u32 = 32u;    // program left nothing on the stack
const ERR_LEFTOVER:  u32 = 64u;    // program left more than one value on the stack
const ERR_NAN:       u32 = 128u;   // final result was NaN or inf
// Soft errors: numeric trouble that is often harmless away from the surface
const ERR_DIV_ZERO:  u32 = 256u;
const ERR_DOMAIN:    u32 = 512u;   // ln/log2 <= 0, sqrt < 0, pow(neg, fractional), k <= 0

const HARD_MASK: u32 = 255u;
// Set true to also visualize soft errors (will often light up large areas)
const SHOW_SOFT_ERRORS: bool = false;

var<private> err: u32 = 0u;

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    var_count: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> bytecode: array<u32>;
@group(0) @binding(2) var<storage, read> vars: array<f32>;
@group(0) @binding(3) var<storage, read> camera: array<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
}

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

// Bit-level NaN/inf test (exponent all ones). More reliable than x != x,
// which some compilers optimize away.
fn is_bad(x: f32) -> bool {
    return (bitcast<u32>(x) & 0x7f800000u) == 0x7f800000u;
}

fn safe_pow(x: f32, y: f32) -> f32 {
    let yr = round(y);
    if (abs(y - yr) < 1e-4) {
        let n = i32(yr);
        if (n == 0) { return 1.0; }
        if (n == 1) { return x; }
        if (n == 2) { return x * x; }
        if (n == 3) { return x * x * x; }
        let mag = pow(abs(x), f32(n));
        if (x < 0.0 && (n & 1) != 0) { return -mag; }
        return mag;
    }
    if (x < 0.0) { err = err | ERR_DOMAIN; }
    return pow(max(x, 0.0), y);
}

fn floor_mod(a: f32, b: f32) -> f32 {
    if (abs(b) < 1e-9) { err = err | ERR_DIV_ZERO; }
    let d = select(b, 1e-9, abs(b) < 1e-9);
    return a - d * floor(a / d);
}

fn safe_div(a: f32, b: f32) -> f32 {
    if (abs(b) < 1e-9) { err = err | ERR_DIV_ZERO; }
    let d = select(b, select(1e-9, -1e-9, b < 0.0), abs(b) < 1e-9);
    return a / d;
}

fn eval_sdf(p: vec3<f32>) -> f32 {
    var v: array<f32, 32>;
    let var_len = min(arrayLength(&vars), MAX_VARS);
    for (var j: u32 = 0u; j < var_len; j = j + 1u) {
        v[j] = vars[j];
    }
    v[0] = p.x;
    v[1] = p.y;
    v[2] = p.z;

    var stack: array<f32, 32>;
    var sp: u32 = 0u;
    var i: u32 = 0u;
    let len: u32 = arrayLength(&bytecode);

    while (i < len) {
        let op = bytecode[i];

        switch op {
            case NUM: {
                if (i + 1u >= len)    { err = err | ERR_TRUNCATED; return 1e9; }
                if (sp >= STACK_SIZE) { err = err | ERR_OVERFLOW;  return 1e9; }
                i = i + 1u;
                stack[sp] = bitcast<f32>(bytecode[i]);
                sp = sp + 1u;
            }
            case VAR: {
                if (i + 1u >= len)    { err = err | ERR_TRUNCATED; return 1e9; }
                if (sp >= STACK_SIZE) { err = err | ERR_OVERFLOW;  return 1e9; }
                i = i + 1u;
                let idx = bytecode[i];
                if (idx >= MAX_VARS || (idx >= var_len && idx >= 3u)) {
                    err = err | ERR_BAD_VAR;
                    return 1e9;
                }
                stack[sp] = v[idx];
                sp = sp + 1u;
            }
            case MINUS: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                stack[sp] = -stack[sp];
                sp = sp + 1u;
            }
            case ABSOLUTE: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                stack[sp] = abs(stack[sp]);
                sp = sp + 1u;
            }
            case COS: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                stack[sp] = cos(stack[sp]);
                sp = sp + 1u;
            }
            case SIN: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                stack[sp] = sin(stack[sp]);
                sp = sp + 1u;
            }
            case TAN: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                stack[sp] = tan(stack[sp]);
                sp = sp + 1u;
            }
            case LN: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                if (stack[sp] <= 0.0) { err = err | ERR_DOMAIN; }
                stack[sp] = log(max(stack[sp], 1e-20));
                sp = sp + 1u;
            }
            case LOG2: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                if (stack[sp] <= 0.0) { err = err | ERR_DOMAIN; }
                stack[sp] = log2(max(stack[sp], 1e-20));
                sp = sp + 1u;
            }
            case SQUAREROOT: {
                if (sp < 1u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u;
                if (stack[sp] < 0.0) { err = err | ERR_DOMAIN; }
                stack[sp] = sqrt(max(stack[sp], 0.0));
                sp = sp + 1u;
            }
            case ADD: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = left + right;
                sp = sp + 1u;
            }
            case SUBTRACT: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = left - right;
                sp = sp + 1u;
            }
            case MULTIPLY: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = left * right;
                sp = sp + 1u;
            }
            case DIVIDE: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = safe_div(left, right);
                sp = sp + 1u;
            }
            case POWER: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = safe_pow(left, right);
                sp = sp + 1u;
            }
            case MOD: {
                if (sp < 2u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let right = stack[sp];
                sp = sp - 1u; let left  = stack[sp];
                stack[sp] = floor_mod(left, right);
                sp = sp + 1u;
            }
            case SMOOTHMIN: {
                if (sp < 3u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let k_raw = stack[sp];
                if (k_raw <= 0.0) { err = err | ERR_DOMAIN; }
                let k = max(k_raw, 1e-6);
                sp = sp - 1u; let b = stack[sp];
                sp = sp - 1u; let a = stack[sp];
                let h = max(k - abs(a - b), 0.0) / k;
                stack[sp] = min(a, b) - h * h * k * 0.25;
                sp = sp + 1u;
            }
            case SMOOTHMAX: {
                if (sp < 3u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let k_raw = stack[sp];
                if (k_raw <= 0.0) { err = err | ERR_DOMAIN; }
                let k = max(k_raw, 1e-6);
                sp = sp - 1u; let b = stack[sp];
                sp = sp - 1u; let a = stack[sp];
                let h = max(k - abs(a - b), 0.0) / k;
                stack[sp] = max(a, b) + h * h * k * 0.25;
                sp = sp + 1u;
            }
            case MIX: {
                if (sp < 3u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let t = stack[sp];
                sp = sp - 1u; let y = stack[sp];
                sp = sp - 1u; let x = stack[sp];
                stack[sp] = x + (y - x) * t;
                sp = sp + 1u;
            }
            case CLAMP: {
                if (sp < 3u) { err = err | ERR_UNDERFLOW; return 1e9; }
                sp = sp - 1u; let hi = stack[sp];
                sp = sp - 1u; let lo = stack[sp];
                sp = sp - 1u; let x  = stack[sp];
                stack[sp] = min(max(x, lo), hi);
                sp = sp + 1u;
            }
            default: {
                err = err | ERR_BAD_OP;
                return 1e9;
            }
        }

        i = i + 1u;
    }

    if (sp == 0u) {
        err = err | ERR_EMPTY;
        return 1e9;
    }
    if (sp != 1u) {
        err = err | ERR_LEFTOVER;   // malformed program, but still use the top value
    }
    let result = stack[sp - 1u];
    if (is_bad(result)) {
        err = err | ERR_NAN;
        return 1e9;
    }
    return result;
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let h = 0.0015;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * eval_sdf(p + k.xyy * h) +
        k.yyx * eval_sdf(p + k.yyx * h) +
        k.yxy * eval_sdf(p + k.yxy * h) +
        k.xxx * eval_sdf(p + k.xxx * h)
    );
}

// Maps the highest-priority error to a color, with diagonal stripes
// so it can't be mistaken for a normal render.
fn error_color(e: u32, frag_xy: vec2<f32>) -> vec4<f32> {
    var c = vec3<f32>(1.0, 1.0, 1.0);
    if      ((e & ERR_OVERFLOW)  != 0u) { c = vec3<f32>(1.0, 0.0, 0.0); }   // red
    else if ((e & ERR_UNDERFLOW) != 0u) { c = vec3<f32>(1.0, 0.5, 0.0); }   // orange
    else if ((e & ERR_BAD_OP)    != 0u) { c = vec3<f32>(1.0, 1.0, 0.0); }   // yellow
    else if ((e & ERR_TRUNCATED) != 0u) { c = vec3<f32>(0.5, 1.0, 0.0); }   // lime
    else if ((e & ERR_BAD_VAR)   != 0u) { c = vec3<f32>(0.0, 1.0, 1.0); }   // cyan
    else if ((e & ERR_EMPTY)     != 0u) { c = vec3<f32>(0.0, 0.3, 1.0); }   // blue
    else if ((e & ERR_LEFTOVER)  != 0u) { c = vec3<f32>(0.6, 0.2, 1.0); }   // purple
    else if ((e & ERR_NAN)       != 0u) { c = vec3<f32>(1.0, 0.0, 1.0); }   // magenta
    else if ((e & ERR_DIV_ZERO)  != 0u) { c = vec3<f32>(1.0, 0.6, 0.7); }   // pink
    else if ((e & ERR_DOMAIN)    != 0u) { c = vec3<f32>(1.0, 1.0, 1.0); }   // white
    let stripe = step(0.5, fract((frag_xy.x + frag_xy.y) / 16.0));
    return vec4<f32>(c * mix(1.0, 0.45, stripe), 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    err = 0u;

    let uv = vec2<f32>(
        frag_coord.x * 2.0 - uniforms.resolution.x,
        uniforms.resolution.y - frag_coord.y * 2.0
    ) / uniforms.resolution.y;

    let cx = select(0.0, camera[0], arrayLength(&camera) > 0u);
    let cy = select(0.0, camera[1], arrayLength(&camera) > 1u);
    let cz = select(-3.0, camera[2], arrayLength(&camera) > 2u);

    let ro = vec3<f32>(cx, cy, cz);
    let rd = normalize(vec3<f32>(uv, 1.5));

    var color = vec4<f32>(0.06, 0.07, 0.10, 1.0);

    var t = 0.0;
    for (var i = 0; i < MAX_STEPS; i++) {
        let p = ro + rd * t;
        let d = eval_sdf(p);

        // Bail out early on structural errors; no point marching a broken program
        if ((err & HARD_MASK) != 0u) { break; }

        let eps = 0.0005 * (1.0 + t);
        if (d < eps) {
            let n = calc_normal(p);
            let light_dir = normalize(vec3<f32>(0.6, 0.8, -0.5));
            let diff = max(dot(n, light_dir), 0.15);
            color = vec4<f32>(vec3<f32>(0.25, 0.75, 1.0) * diff, 1.0);
            break;
        }
        t += max(d * STEP_SCALE, 0.0005);
        if (t > MAX_DIST) { break; }
    }

    let shown = select(err & HARD_MASK, err, SHOW_SOFT_ERRORS);
    if (shown != 0u) {
        return error_color(shown, frag_coord.xy);
    }
    return color;
}