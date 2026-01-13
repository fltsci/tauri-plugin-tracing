/**
 * Event listeners for receiving logs from the Rust backend.
 * @module
 */

import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { LogLevel, type LoggerFn, type RecordPayload } from './types'
import { cleanMessage } from './utils'

/**
 * Attaches a custom listener for log events from the Rust backend.
 *
 * Use this to implement custom log handling, such as sending logs to
 * an external service or storing them locally.
 *
 * @param fn - Callback function called for each log entry
 * @returns A function to unsubscribe from log events
 *
 * @example
 * ```ts
 * const unlisten = await attachLogger(({ level, message }) => {
 *   if (level === LogLevel.Error) {
 *     sendToErrorTracking(message);
 *   }
 * });
 *
 * // Later, to stop listening:
 * unlisten();
 * ```
 */
export async function attachLogger(fn: LoggerFn): Promise<UnlistenFn> {
  return await listen('tracing://log', (event: Event<RecordPayload>) => {
    const { level } = event.payload
    const message = cleanMessage(event.payload.message)

    fn({ message, level })
  })
}

/**
 * Attaches a listener that forwards log events to the browser console.
 *
 * Maps each log level to the appropriate console method:
 * - Trace/Debug → `console.log`/`console.debug`
 * - Info → `console.info`
 * - Warn → `console.warn`
 * - Error → `console.error`
 *
 * @returns A function to unsubscribe from log events
 *
 * @example
 * ```ts
 * // Start forwarding logs to console
 * const unlisten = await attachConsole();
 *
 * // Logs from Rust will now appear in browser DevTools
 *
 * // To stop forwarding:
 * unlisten();
 * ```
 */
export async function attachConsole(): Promise<UnlistenFn> {
  return await attachLogger(({ level, message }: RecordPayload) => {
    switch (level) {
      case LogLevel.Trace:
        console.log(message)
        break
      case LogLevel.Debug:
        console.debug(message)
        break
      case LogLevel.Info:
        console.info(message)
        break
      case LogLevel.Warn:
        console.warn(message)
        break
      case LogLevel.Error:
        console.error(message)
        break
      default:
        throw new Error(`unknown log level ${level}`)
    }
  })
}
