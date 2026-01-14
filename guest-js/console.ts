/**
 * Console interception to route browser console calls to Rust tracing.
 *
 * See the main module documentation for a guide on choosing between
 * `attachConsole()`, `interceptConsole()`, and `takeoverConsole()`.
 *
 * @module
 */

import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { trace, debug, info, warn, error } from './log'
import { LogLevel, type RecordPayload } from './types'
import { cleanMessage } from './utils'

type ConsoleFn = (...args: unknown[]) => void

/**
 * All console methods that we intercept.
 */
interface FullConsole {
  // Logging methods - route to tracing
  log: ConsoleFn
  debug: ConsoleFn
  info: ConsoleFn
  warn: ConsoleFn
  error: ConsoleFn
  trace: ConsoleFn
  assert: typeof console.assert
  dir: typeof console.dir
  dirxml: typeof console.dirxml
  table: typeof console.table
  // UI/utility methods - pass through to original
  clear: typeof console.clear
  count: typeof console.count
  countReset: typeof console.countReset
  group: typeof console.group
  groupCollapsed: typeof console.groupCollapsed
  groupEnd: typeof console.groupEnd
  time: typeof console.time
  timeEnd: typeof console.timeEnd
  timeLog: typeof console.timeLog
}

// For backward compatibility, the simple interface used by interceptConsole
interface OriginalConsole {
  log: ConsoleFn
  debug: ConsoleFn
  info: ConsoleFn
  warn: ConsoleFn
  error: ConsoleFn
  trace: ConsoleFn
}

let originalConsole: OriginalConsole | null = null
let fullOriginalConsole: FullConsole | null = null

/**
 * Intercepts console calls and routes them to Rust tracing.
 *
 * @param options.preserveOriginal - If true, also calls the original console method (default: false)
 * @returns A function to restore the original console
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
 * Restores all console methods after full takeover.
 */
function restoreFullConsole(): void {
  if (fullOriginalConsole) {
    console.log = fullOriginalConsole.log
    console.debug = fullOriginalConsole.debug
    console.info = fullOriginalConsole.info
    console.warn = fullOriginalConsole.warn
    console.error = fullOriginalConsole.error
    console.trace = fullOriginalConsole.trace
    console.assert = fullOriginalConsole.assert
    console.dir = fullOriginalConsole.dir
    console.dirxml = fullOriginalConsole.dirxml
    console.table = fullOriginalConsole.table
    console.clear = fullOriginalConsole.clear
    console.count = fullOriginalConsole.count
    console.countReset = fullOriginalConsole.countReset
    console.group = fullOriginalConsole.group
    console.groupCollapsed = fullOriginalConsole.groupCollapsed
    console.groupEnd = fullOriginalConsole.groupEnd
    console.time = fullOriginalConsole.time
    console.timeEnd = fullOriginalConsole.timeEnd
    console.timeLog = fullOriginalConsole.timeLog
    fullOriginalConsole = null
  }
}

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
export async function takeoverConsole(): Promise<() => void> {
  // Store ALL original console methods before interception
  fullOriginalConsole = {
    // Logging methods
    log: console.log.bind(console),
    debug: console.debug.bind(console),
    info: console.info.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    trace: console.trace.bind(console),
    assert: console.assert.bind(console),
    dir: console.dir.bind(console),
    dirxml: console.dirxml.bind(console),
    table: console.table.bind(console),
    // UI/utility methods
    clear: console.clear.bind(console),
    count: console.count.bind(console),
    countReset: console.countReset.bind(console),
    group: console.group.bind(console),
    groupCollapsed: console.groupCollapsed.bind(console),
    groupEnd: console.groupEnd.bind(console),
    time: console.time.bind(console),
    timeEnd: console.timeEnd.bind(console),
    timeLog: console.timeLog.bind(console)
  }

  const saved = fullOriginalConsole

  // Helper to wrap logging methods
  const wrapLog = (
    level: 'trace' | 'debug' | 'info' | 'warn' | 'error'
  ): ConsoleFn => {
    const logFn = { trace, debug, info, warn, error }[level]
    return (...args: unknown[]) => logFn(...args)
  }

  // Replace logging methods - route to tracing
  console.log = wrapLog('debug')
  console.debug = wrapLog('debug')
  console.info = wrapLog('info')
  console.warn = wrapLog('warn')
  console.error = wrapLog('error')
  console.trace = wrapLog('trace')

  // assert: log error only if condition is falsy
  console.assert = (condition?: boolean, ...data: unknown[]) => {
    if (!condition) {
      error('Assertion failed:', ...data)
    }
  }

  // Object inspection methods - map to debug level
  console.dir = (...args: unknown[]) => debug(...args)
  console.dirxml = (...args: unknown[]) => debug(...args)
  console.table = (...args: unknown[]) => {
    // Try to format table data nicely
    const [data, columns] = args
    if (columns) {
      debug('table:', data, 'columns:', columns)
    } else {
      debug('table:', data)
    }
  }

  // UI/utility methods (clear, count, countReset, group, groupCollapsed, groupEnd,
  // time, timeEnd, timeLog) are intentionally NOT replaced. They continue to call
  // the native console methods directly with zero IPC overhead. Only logging methods
  // that produce actual log output are routed through Rust tracing.

  // Listen for Rust tracing events and output using the ORIGINAL console methods
  const unlisten: UnlistenFn = await listen(
    'tracing://log',
    (event: Event<RecordPayload>) => {
      const { level } = event.payload
      const message = cleanMessage(event.payload.message)

      switch (level) {
        case LogLevel.Trace:
          saved.log(message)
          break
        case LogLevel.Debug:
          saved.debug(message)
          break
        case LogLevel.Info:
          saved.info(message)
          break
        case LogLevel.Warn:
          saved.warn(message)
          break
        case LogLevel.Error:
          saved.error(message)
          break
      }
    }
  )

  // Return cleanup function
  return () => {
    unlisten()
    restoreFullConsole()
  }
}
