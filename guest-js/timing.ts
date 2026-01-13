/**
 * Performance timing utilities.
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Starts a performance timer with the given label.
 *
 * Similar to `console.time()`. Use {@link timeEnd} with the same label
 * to stop the timer and log the elapsed time.
 *
 * @param label - A unique identifier for this timer
 *
 * @example
 * ```ts
 * time('database-query');
 * const results = await db.query('SELECT * FROM users');
 * timeEnd('database-query'); // Logs: "database-query: 42.123ms"
 * ```
 */
export function time(label: string): void {
  invoke<void>('plugin:tracing|time', {
    label,
    callStack: new Error().stack
  }).catch(console.error)
}

/**
 * Stops a performance timer and logs the elapsed time.
 *
 * Similar to `console.timeEnd()`. Must be called with a label that was
 * previously started with {@link time}. Logs a warning if no timer
 * with the given label exists.
 *
 * @param label - The identifier of the timer to stop
 *
 * @example
 * ```ts
 * time('fetch-data');
 * const data = await fetch('/api/data');
 * timeEnd('fetch-data'); // Logs: "fetch-data: 156.789ms"
 * ```
 */
export function timeEnd(label: string): void {
  invoke<void>('plugin:tracing|time_end', {
    label,
    callStack: new Error().stack
  }).catch(console.error)
}
