/* tslint:disable */
/* eslint-disable */

export function compile_shader(source: string): void;

export function get_shader_memory(): any;

export function init_engine(): void;

/**
 * Called once after wasm-bindgen `init()`, passing `instance.exports` from this same wasm module.
 */
export function lpvm_init_exports(exports: any): void;

export function render_frame(width: number, height: number, time_q32: number, out_ptr: number): void;

export function shader_ready(): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly compile_shader: (a: number, b: number) => [number, number];
    readonly get_shader_memory: () => any;
    readonly init_engine: () => void;
    readonly lpvm_init_exports: (a: any) => void;
    readonly __lps_acos_q32: (a: number) => number;
    readonly __lps_acosh_q32: (a: number) => number;
    readonly __lps_asin_q32: (a: number) => number;
    readonly __lps_asinh_q32: (a: number) => number;
    readonly __lps_atan2_q32: (a: number, b: number) => number;
    readonly __lps_atan_q32: (a: number) => number;
    readonly __lps_atanh_q32: (a: number) => number;
    readonly __lps_cos_q32: (a: number) => number;
    readonly __lps_cosh_q32: (a: number) => number;
    readonly __lps_exp2_q32: (a: number) => number;
    readonly __lps_exp_q32: (a: number) => number;
    readonly __lps_fma_q32: (a: number, b: number, c: number) => number;
    readonly __lps_inversesqrt_q32: (a: number) => number;
    readonly __lps_ldexp_q32: (a: number, b: number) => number;
    readonly __lps_log2_q32: (a: number) => number;
    readonly __lps_log_q32: (a: number) => number;
    readonly __lps_mod_q32: (a: number, b: number) => number;
    readonly __lps_pow_q32: (a: number, b: number) => number;
    readonly __lps_round_q32: (a: number) => number;
    readonly __lps_sin_q32: (a: number) => number;
    readonly __lps_sinh_q32: (a: number) => number;
    readonly __lps_tan_q32: (a: number) => number;
    readonly __lps_tanh_q32: (a: number) => number;
    readonly __lp_lpir_fadd_q32: (a: number, b: number) => number;
    readonly __lp_lpir_fdiv_q32: (a: number, b: number) => number;
    readonly __lp_lpir_fmul_q32: (a: number, b: number) => number;
    readonly __lp_lpir_fnearest_q32: (a: number) => number;
    readonly __lp_lpir_fsqrt_q32: (a: number) => number;
    readonly __lp_lpir_fsub_q32: (a: number, b: number) => number;
    readonly __lp_lpfx_fbm2_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_fbm2_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_fbm3_f32: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly __lp_lpfx_fbm3_q32: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly __lp_lpfx_fbm3_tile_f32: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly __lp_lpfx_fbm3_tile_q32: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly __lp_lpfx_gnoise1_f32: (a: number, b: number) => number;
    readonly __lp_lpfx_gnoise1_q32: (a: number, b: number) => number;
    readonly __lp_lpfx_gnoise2_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_gnoise2_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_gnoise3_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_gnoise3_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_gnoise3_tile_f32: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly __lp_lpfx_gnoise3_tile_q32: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly __lp_lpfx_hash_1: (a: number, b: number) => number;
    readonly __lp_lpfx_hash_2: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_hash_3: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_hsv2rgb_f32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_hsv2rgb_q32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_hsv2rgb_vec4_f32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_hsv2rgb_vec4_q32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_hue2rgb_f32: (a: number, b: number) => void;
    readonly __lp_lpfx_hue2rgb_q32: (a: number, b: number) => void;
    readonly __lp_lpfx_psrdnoise2_f32: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly __lp_lpfx_psrdnoise2_q32: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly __lp_lpfx_psrdnoise3_f32: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
    readonly __lp_lpfx_psrdnoise3_q32: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
    readonly __lp_lpfx_random1_f32: (a: number, b: number) => number;
    readonly __lp_lpfx_random1_q32: (a: number, b: number) => number;
    readonly __lp_lpfx_random2_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_random2_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_random3_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_random3_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_rgb2hsv_f32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_rgb2hsv_q32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_rgb2hsv_vec4_f32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_rgb2hsv_vec4_q32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_saturate_f32: (a: number) => number;
    readonly __lp_lpfx_saturate_q32: (a: number) => number;
    readonly __lp_lpfx_saturate_vec3_f32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_saturate_vec3_q32: (a: number, b: number, c: number, d: number) => void;
    readonly __lp_lpfx_saturate_vec4_f32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_saturate_vec4_q32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_snoise1_f32: (a: number, b: number) => number;
    readonly __lp_lpfx_snoise1_q32: (a: number, b: number) => number;
    readonly __lp_lpfx_snoise2_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_snoise2_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_snoise3_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_snoise3_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_srandom1_f32: (a: number, b: number) => number;
    readonly __lp_lpfx_srandom1_q32: (a: number, b: number) => number;
    readonly __lp_lpfx_srandom2_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_srandom2_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_srandom3_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_srandom3_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_srandom3_tile_f32: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly __lp_lpfx_srandom3_tile_q32: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly __lp_lpfx_srandom3_vec_f32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_srandom3_vec_q32: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __lp_lpfx_worley2_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_worley2_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_worley2_value_f32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_worley2_value_q32: (a: number, b: number, c: number) => number;
    readonly __lp_lpfx_worley3_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_worley3_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_worley3_value_f32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_lpfx_worley3_value_q32: (a: number, b: number, c: number, d: number) => number;
    readonly __lp_vm_get_fuel_q32: (a: number) => number;
    readonly render_frame: (a: number, b: number, c: number, d: number) => [number, number];
    readonly shader_ready: () => number;
    readonly memcmp: (a: number, b: number, c: number) => number;
    readonly memcpy: (a: number, b: number, c: number) => number;
    readonly memset: (a: number, b: number, c: number) => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
