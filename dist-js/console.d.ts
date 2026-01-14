/**
 * Console interception to route browser console calls to Rust tracing.
 * @module
 */
/**
 * Intercepts browser console calls and routes them to the Rust tracing system.
 *
 * After calling this function, all console.log, console.debug, console.info,
 * console.warn, and console.error calls will be forwarded to the Rust backend
 * via the tracing plugin.
 *
 * @param options - Configuration options
 * @param options.preserveOriginal - If true, also calls the original console method (default: false)
 * @returns A function to restore the original console
 *
 * @example
 * ```ts
 * import { interceptConsole } from '@fltsci/tauri-plugin-tracing';
 *
 * // All console calls now go to Rust tracing
 * const restore = interceptConsole();
 *
 * console.log('This goes to Rust');
 * console.error('Errors too');
 *
 * // Restore original console
 * restore();
 * ```
 */
export declare function interceptConsole(options?: {
    preserveOriginal?: boolean;
}): () => void;
/**
 * Restores the original console methods after interception.
 *
 * This is automatically returned by interceptConsole(), but can also be
 * called directly if needed.
 */
export declare function restoreConsole(): void;
/**
 * Completely takes over the webview console, routing all logs through Rust tracing.
 *
 * This function:
 * 1. Intercepts all console calls (log, debug, info, warn, error, trace) and
 *    sends them to the Rust tracing backend
 * 2. Listens for Rust tracing events and outputs them to the original console
 *
 * This creates a unified logging pipeline where all logs (both JS and Rust)
 * flow through Rust's tracing infrastructure, then appear in the browser console.
 *
 * @returns A promise that resolves to a cleanup function to restore normal console behavior
 *
 * @example
 * ```ts
 * import { takeoverConsole } from '@fltsci/tauri-plugin-tracing';
 *
 * // Take over console - all logs now flow through Rust tracing
 * const restore = await takeoverConsole();
 *
 * console.log('This goes to Rust, then back to console');
 * console.error('Errors too');
 *
 * // Restore original console behavior
 * restore();
 * ```
 */
export declare function takeoverConsole(): Promise<() => void>;
