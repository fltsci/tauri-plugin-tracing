/**
 * Console interception to route browser console calls to Rust tracing.
 * @module
 */

import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { trace, debug, info, warn, error } from './log'
import { LogLevel, type RecordPayload } from './types'
import { cleanMessage } from './utils'

type ConsoleFn = (...args: unknown[]) => void

interface OriginalConsole {
  log: ConsoleFn
  debug: ConsoleFn
  info: ConsoleFn
  warn: ConsoleFn
  error: ConsoleFn
  trace: ConsoleFn
}

let originalConsole: OriginalConsole | null = null

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
export function interceptConsole(
  options: { preserveOriginal?: boolean } = {}
): () => void {
  const { preserveOriginal = false } = options

  // Store original console methods
  originalConsole = {
    log: console.log.bind(console),
    debug: console.debug.bind(console),
    info: console.info.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    trace: console.trace.bind(console)
  }

  const wrap = (
    level: 'trace' | 'debug' | 'info' | 'warn' | 'error',
    original: ConsoleFn
  ): ConsoleFn => {
    const logFn = { trace, debug, info, warn, error }[level]
    return (...args: unknown[]) => {
      logFn(...args)
      if (preserveOriginal) {
        original(...args)
      }
    }
  }

  console.log = wrap('debug', originalConsole.log)
  console.debug = wrap('debug', originalConsole.debug)
  console.info = wrap('info', originalConsole.info)
  console.warn = wrap('warn', originalConsole.warn)
  console.error = wrap('error', originalConsole.error)
  console.trace = wrap('trace', originalConsole.trace)

  return restoreConsole
}

/**
 * Restores the original console methods after interception.
 *
 * This is automatically returned by interceptConsole(), but can also be
 * called directly if needed.
 */
export function restoreConsole(): void {
  if (originalConsole) {
    console.log = originalConsole.log
    console.debug = originalConsole.debug
    console.info = originalConsole.info
    console.warn = originalConsole.warn
    console.error = originalConsole.error
    console.trace = originalConsole.trace
    originalConsole = null
  }
}

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
export async function takeoverConsole(): Promise<() => void> {
  // Store original console methods before interception
  const savedConsole: OriginalConsole = {
    log: console.log.bind(console),
    debug: console.debug.bind(console),
    info: console.info.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    trace: console.trace.bind(console)
  }

  // Intercept console calls and route to Rust (don't preserve original - we'll handle output ourselves)
  interceptConsole({ preserveOriginal: false })

  // Listen for Rust tracing events and output using the ORIGINAL console methods
  // This avoids infinite loops since we use savedConsole, not the intercepted console
  const unlisten: UnlistenFn = await listen(
    'tracing://log',
    (event: Event<RecordPayload>) => {
      const { level } = event.payload
      const message = cleanMessage(event.payload.message)

      switch (level) {
        case LogLevel.Trace:
          savedConsole.log(message)
          break
        case LogLevel.Debug:
          savedConsole.debug(message)
          break
        case LogLevel.Info:
          savedConsole.info(message)
          break
        case LogLevel.Warn:
          savedConsole.warn(message)
          break
        case LogLevel.Error:
          savedConsole.error(message)
          break
      }
    }
  )

  // Return cleanup function
  return () => {
    unlisten()
    restoreConsole()
  }
}
