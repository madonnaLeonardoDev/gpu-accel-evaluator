const STACK_CAP: i32 = 64;
const MAX_VARS: u32 = 64u;

const NUM: u32 = 0u;
const VAR: u32 = 1u;
const MINUS: u32 = 2u;
const ABSOLUTE: u32 = 3u;
const COS: u32 = 4u;
const SIN: u32 = 5u;
const TAN: u32 = 6u;
const LN: u32 = 7u;
const LOG2: u32 = 8u;
const SQUAREROOT: u32 = 9u;
const ADD: u32 = 10u;
const SUBTRACT: u32 = 11u;
const MULTIPLY: u32 = 12u;
const DIVIDE: u32 = 13u;
const POWER: u32 = 14u;
const MOD: u32 = 15u;
const SMOOTHMIN: u32 = 16u;
const SMOOTHMAX: u32 = 17u;
const MIX: u32 = 18u;
const CLAMP: u32 = 19u;

struct Params {
    code_len: u32,
    vars_len: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> bytecode: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read_write> out_val: f32;
@group(0) @binding(3) var<storage, read> vars_buf: array<f32>;

var<private> g_stack: array<f32, STACK_CAP>;
var<private> g_sp: i32 = 0;
var<private> g_vars: array<f32, MAX_VARS>;
var<private> g_vars_len: u32 = 0u;

fn push_val(v: f32) {
    g_stack[g_sp] = v;
    g_sp += 1;
}

fn pop_val() -> f32 {
    g_sp -= 1;
    return g_stack[g_sp];
}

fn read_var(slot: u32) -> f32 {
    if (slot < g_vars_len && slot < MAX_VARS) {
        return g_vars[slot];
    }
    return 0.0;
}

fn execute_bytecode(code_len: u32, vars_len: u32) -> f32 {
    g_sp = 0;
    g_vars_len = min(vars_len, MAX_VARS);

    var v_idx: u32 = 0u;
    loop {
        if (v_idx >= g_vars_len) {
            break;
        }
        g_vars[v_idx] = vars_buf[v_idx];
        v_idx += 1u;
    }

    var i: u32 = 0u;
    loop {
        if (i >= code_len) {
            break;
        }

        let op: u32 = bytecode[i];
        switch (op) {
            case NUM: {
                i += 1u;
                push_val(bitcast<f32>(bytecode[i]));
            }
            case VAR: {
                i += 1u;
                let slot: u32 = bytecode[i];
                push_val(read_var(slot));
            }
            case MINUS: {
                let num: f32 = pop_val();
                push_val(-num);
            }
            case ABSOLUTE: {
                let num: f32 = pop_val();
                push_val(abs(num));
            }
            case COS: {
                let num: f32 = pop_val();
                push_val(cos(num));
            }
            case SIN: {
                let num: f32 = pop_val();
                push_val(sin(num));
            }
            case TAN: {
                let num: f32 = pop_val();
                push_val(tan(num));
            }
            case LN: {
                let num: f32 = pop_val();
                push_val(log(num));
            }
            case LOG2: {
                let num: f32 = pop_val();
                push_val(log2(num));
            }
            case SQUAREROOT: {
                let num: f32 = pop_val();
                push_val(sqrt(num));
            }
            case ADD: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(left + right);
            }
            case SUBTRACT: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(left - right);
            }
            case MULTIPLY: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(left * right);
            }
            case DIVIDE: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(left / right);
            }
            case POWER: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(pow(left, right));
            }
            case MOD: {
                let right: f32 = pop_val();
                let left: f32 = pop_val();
                push_val(left % right);
            }
            case SMOOTHMIN: {
                let k: f32 = pop_val();
                let b: f32 = pop_val();
                let a: f32 = pop_val();
                let h: f32 = max(k - abs(a - b), 0.0) / k;
                push_val(min(a, b) - h * h * k * 0.25);
            }
            case SMOOTHMAX: {
                let k: f32 = pop_val();
                let b: f32 = pop_val();
                let a: f32 = pop_val();
                let h: f32 = max(k - abs(a - b), 0.0) / k;
                push_val(max(a, b) + h * h * k * 0.25);
            }
            case MIX: {
                let t: f32 = pop_val();
                let y: f32 = pop_val();
                let x: f32 = pop_val();
                push_val(mix(x, y, t));
            }
            case CLAMP: {
                let max_val: f32 = pop_val();
                let min_val: f32 = pop_val();
                let x: f32 = pop_val();
                push_val(clamp(x, min_val, max_val));
            }
            default: {
                push_val(0.0);
            }
        }

        i += 1u;
    }

    if (g_sp <= 0) {
        return 0.0;
    }
    return g_stack[g_sp - 1];
}

// Helper for shaders that mutate local `g_vars` (e.g. per-pixel/ray coords) before running.
fn set_var(slot: u32, val: f32) {
    if (slot < MAX_VARS) {
        g_vars[slot] = val;
        if (slot >= g_vars_len) {
            g_vars_len = slot + 1u;
        }
    }
}

@compute @workgroup_size(1)
fn main() {
    out_val = execute_bytecode(params.code_len, params.vars_len);
}