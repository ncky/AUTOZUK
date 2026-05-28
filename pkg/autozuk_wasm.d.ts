declare namespace wasm_bindgen {
    /* tslint:disable */
    /* eslint-disable */

    export function configure_sim_context(input_json: string): string;

    export function exclude_tiles(input_json: string): string;

    export function simulate_tile(input_json: string): string;

    export function simulate_tiles(input_json: string): string;

    export function simulate_tiles_cached(tile_coords: Uint8Array, max_sims: number, quick_prune: boolean): string;

    export function simulate_tiles_top_cached(tile_coords: Uint8Array, max_sims: number, quick_prune: boolean, limit: number): string;

}
declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly configure_sim_context: (a: number, b: number) => [number, number];
    readonly exclude_tiles: (a: number, b: number) => [number, number];
    readonly simulate_tile: (a: number, b: number) => [number, number];
    readonly simulate_tiles: (a: number, b: number) => [number, number];
    readonly simulate_tiles_cached: (a: number, b: number, c: number, d: number) => [number, number];
    readonly simulate_tiles_top_cached: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
