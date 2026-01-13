/**
 * Flamegraph and flamechart generation functions.
 *
 * These functions require the `flamegraph` feature to be enabled in the Rust plugin
 * and `with_flamegraph()` to be called on the Builder.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Generates a flamegraph SVG from recorded profiling data.
 *
 * Flamegraphs collapse identical stack frames and sort them, making them
 * ideal for identifying hot paths in long-running or multi-threaded applications.
 *
 * @returns The path to the generated SVG file
 * @throws If no profiling data is available or SVG generation fails
 *
 * @example
 * ```ts
 * import { generateFlamegraph } from '@fltsci/tauri-plugin-tracing';
 *
 * // After running your application with profiling enabled...
 * const svgPath = await generateFlamegraph();
 * console.log(`Flamegraph saved to: ${svgPath}`);
 * ```
 */
export async function generateFlamegraph(): Promise<string> {
  return await invoke<string>('plugin:tracing|generate_flamegraph')
}

/**
 * Generates a flamechart SVG from recorded profiling data.
 *
 * Unlike flamegraphs, flamecharts preserve the exact ordering of events
 * as they were recorded, making it easier to see when each span occurs
 * relative to others. This is useful for understanding the temporal flow
 * of execution.
 *
 * @returns The path to the generated SVG file
 * @throws If no profiling data is available or SVG generation fails
 *
 * @example
 * ```ts
 * import { generateFlamechart } from '@fltsci/tauri-plugin-tracing';
 *
 * // After running your application with profiling enabled...
 * const svgPath = await generateFlamechart();
 * console.log(`Flamechart saved to: ${svgPath}`);
 * ```
 */
export async function generateFlamechart(): Promise<string> {
  return await invoke<string>('plugin:tracing|generate_flamechart')
}
