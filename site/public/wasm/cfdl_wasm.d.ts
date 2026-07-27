/* tslint:disable */
/* eslint-disable */

/**
 * Compile an in-memory file map to IR. See module docs for the JSON shapes.
 */
export function compile(files_json: string, root_file: string): string;

/**
 * One-shot compile + run from sources (convenience for the playground).
 */
export function compile_and_run(files_json: string, root_file: string, config_json?: string | null, pack?: string | null): string;

/**
 * Run compiled IR. `config_json` is an optional run-config; `pack` optionally
 * applies that pack's domain metrics from the embedded registry.
 */
export function run(ir_json: string, config_json?: string | null, pack?: string | null): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly compile: (a: number, b: number, c: number, d: number) => [number, number];
    readonly run: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly compile_and_run: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
