/**
 * Console interception to route browser console calls to Rust tracing.
 *
 * See the main module documentation for a guide on choosing between
 * `attachConsole()`, `interceptConsole()`, and `takeoverConsole()`.
 *
 * @module
 */
/**
 * Intercepts console calls and routes them to Rust tracing.
 *
 * @param options.preserveOriginal - If true, also calls the original console method (default: false)
 * @returns A function to restore the original console
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
 * Full console takeover: JS console → Rust tracing → browser console.
 *
 * **Routed through Rust:** `log`, `debug`, `info`, `warn`, `error`, `trace`,
 * `assert`, `dir`, `dirxml`, `table`
 *
 * **Native (zero IPC overhead):** `clear`, `count`, `countReset`, `group`,
 * `groupCollapsed`, `groupEnd`, `time`, `timeEnd`, `timeLog`
 *
 * @returns A cleanup function to restore normal console behavior
 */
export declare function takeoverConsole(): Promise<() => void>;
