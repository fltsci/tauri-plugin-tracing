/**
 * @module
 *
 * Tauri plugin for structured logging via the tracing crate.
 *
 * This module provides logging functions that bridge JavaScript logs to Rust's
 * tracing infrastructure, along with performance timing utilities.
 *
 * @example
 * ```ts
 * import { info, debug, error, time, timeEnd } from '@fltsci/tauri-plugin-tracing';
 *
 * info('Application started');
 * debug('Debug details', { user: 'alice' });
 * error('Something went wrong');
 *
 * time('operation');
 * // ... perform work ...
 * timeEnd('operation'); // Logs elapsed time
 * ```
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * A log message consisting of one or more values.
 *
 * Mirrors the variadic signature of `console.log`, allowing multiple
 * arguments to be passed and concatenated in the log output.
 */
export type LogMessage = [
  ...Parameters<typeof console.log>[0],
  ...Parameters<typeof console.log>
]

/**
 * Log severity levels.
 *
 * These levels correspond to the tracing crate's Level enum in Rust.
 * Lower values indicate more verbose (less severe) logs.
 */
enum LogLevel {
  /**
   * The "trace" level.
   *
   * Designates very low priority, often extremely verbose, information.
   */
  Trace = 1,
  /**
   * The "debug" level.
   *
   * Designates lower priority information.
   */
  Debug = 2,
  /**
   * The "info" level.
   *
   * Designates useful information.
   */
  Info = 3,
  /**
   * The "warn" level.
   *
   * Designates hazardous situations.
   */
  Warn = 4,
  /**
   * The "error" level.
   *
   * Designates very serious errors.
   */
  Error = 5
}

/**
 * Strips ANSI escape codes from a string.
 *
 * Used to sanitize log messages that may contain terminal color codes
 * before sending them to the Rust backend.
 *
 * @param s - The value to strip ANSI codes from
 * @returns The string with all ANSI escape sequences removed
 */
const stripAnsi = (s?: unknown): string => {
  return String(s).replace(
    // TODO: Investigate security/detect-unsafe-regex
    // biome-ignore lint/suspicious/noControlCharactersInRegex: this is in the tauri log plugin
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,
    ''
  )
}

/**
 * Creates a replacer function for JSON.stringify that handles circular references.
 *
 * When a circular reference is detected, it is replaced with the string "[Circular]"
 * instead of throwing an error.
 *
 * @returns A replacer function for use with JSON.stringify
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify#the_replacer_parameter
 */
function getCircularReplacer() {
  const ancestors: unknown[] = []
  return function (_key: unknown, value: unknown) {
    if (typeof value !== 'object' || value === null) {
      return value
    }
    // `this` is the object that value is contained in,
    // i.e., its direct parent.
    // @ts-expect-error -- this type is meant to be unknown, this is a debug container
    while (ancestors.length > 0 && ancestors.at(-1) !== this) {
      ancestors.pop()
    }
    if (ancestors.includes(value)) {
      return '[Circular]'
    }
    ancestors.push(value)
    return value
  }
}

/**
 * Converts an arbitrary value to a clean string representation.
 *
 * Handles circular references and strips ANSI codes from the result.
 *
 * @param value - Any value to convert to string
 * @returns A JSON string representation with ANSI codes removed
 */
const cleanUntypedValue = (value: unknown): string =>
  stripAnsi(JSON.stringify(value, getCircularReplacer()))

/**
 * Performs printf-style string formatting like console.log.
 *
 * Supports the following format specifiers:
 * - `%s` - String
 * - `%d`, `%i` - Integer
 * - `%f` - Float
 * - `%o`, `%O` - Object (JSON)
 * - `%%` - Literal percent sign
 *
 * @param format - The format string
 * @param args - Arguments to substitute
 * @returns The formatted string and any remaining arguments
 */
function formatPrintf(format: string, args: unknown[]): [string, unknown[]] {
  const remainingArgs = [...args]
  const result = format.replace(/%([sdifooO%])/g, (match, specifier) => {
    if (specifier === '%') return '%'
    if (remainingArgs.length === 0) return match

    const arg = remainingArgs.shift()
    switch (specifier) {
      case 's':
        return String(arg)
      case 'd':
      case 'i':
        return String(Math.floor(Number(arg)))
      case 'f':
        return String(Number(arg))
      case 'o':
      case 'O':
        return JSON.stringify(arg, getCircularReplacer())
      default:
        return match
    }
  })
  return [result, remainingArgs]
}

/**
 * Sanitizes a log message for transmission to the Rust backend.
 *
 * Handles printf-style format strings (like console.log), strips ANSI codes,
 * and converts values to safe string representations.
 *
 * @param message - The log message to clean
 * @returns A sanitized LogMessage array
 */
const cleanMessage = (message: LogMessage): LogMessage => {
  const safeMessage: string[] = []
  if (typeof message === 'string') {
    safeMessage.push(stripAnsi(message))
  } else if (Array.isArray(message)) {
    // Check if first argument is a string that might be a format string
    if (message.length > 1 && typeof message[0] === 'string' && message[0].includes('%')) {
      const [formatted, remaining] = formatPrintf(message[0], message.slice(1))
      safeMessage.push(stripAnsi(formatted))
      for (const arg of remaining) {
        safeMessage.push(stripAnsi(arg))
      }
    } else {
      for (const msg of message) {
        safeMessage.push(stripAnsi(msg))
      }
    }
  } else if (typeof message === 'object') {
    for (const [key, value] of Object.entries(message)) {
      safeMessage.push(`${stripAnsi(key)}: ${cleanUntypedValue(value)}`)
    }
  } else {
    error(
      `Unhandled type: message is not a string, array, or object, message is ${typeof message}`
    )
  }
  return safeMessage as LogMessage
}

/**
 * Internal function to send a log message to the Rust backend.
 *
 * Captures the current call stack for source location information
 * and invokes the tracing plugin command.
 *
 * @param level - The severity level of the log
 * @param msg - The message parts to log
 */
function log(level: LogLevel, ...msg: LogMessage) {
  const message = cleanMessage(msg)
  invoke<void>('plugin:tracing|log', {
    level,
    message,
    callStack: new Error().stack
  }).catch(console.error)
}

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

/**
 * Logs a message at the error level.
 *
 * Use for serious errors that require immediate attention.
 *
 * @param message - One or more values to log
 *
 * @example
 * ```ts
 * import { error } from '@fltsci/tauri-plugin-tracing';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * error('Multiple', 'arguments', { also: 'work' });
 * ```
 */
export function error(...message: LogMessage): void {
  log(LogLevel.Error, ...message)
}

/**
 * Logs a message at the warn level.
 *
 * Use for potentially hazardous situations that don't prevent operation.
 *
 * @param message - One or more values to log
 *
 * @example
 * ```ts
 * import { warn } from '@fltsci/tauri-plugin-tracing';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! ${warn_description}!`);
 * ```
 */
export function warn(...message: LogMessage): void {
  log(LogLevel.Warn, ...message)
}

/**
 * Logs a message at the info level.
 *
 * Use for general informational messages about application state.
 *
 * @param message - One or more values to log
 *
 * @example
 * ```ts
 * import { info } from '@fltsci/tauri-plugin-tracing';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port ${conn_info.port} at ${conn_info.speed} Mb/s`);
 * ```
 */
export function info(...message: LogMessage): void {
  log(LogLevel.Info, ...message)
}

/**
 * Logs a message at the debug level.
 *
 * Use for detailed information useful during development and debugging.
 *
 * @param message - One or more values to log
 *
 * @example
 * ```ts
 * import { debug } from '@fltsci/tauri-plugin-tracing';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: ${pos.x}, y: ${pos.y}`);
 * ```
 */
export function debug(...message: LogMessage): void {
  log(LogLevel.Debug, ...message)
}

/**
 * Logs a message at the trace level.
 *
 * Use for very verbose, low-priority information. Often filtered out
 * in production builds.
 *
 * @param message - One or more values to log
 *
 * @example
 * ```ts
 * import { trace } from '@fltsci/tauri-plugin-tracing';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: ${pos.x}, y: ${pos.y}`);
 * ```
 */
export function trace(...message: LogMessage): void {
  log(LogLevel.Trace, ...message)
}

/**
 * Payload structure for log records emitted via events.
 *
 * Used when listening to log events from the Rust backend.
 */
interface RecordPayload {
  /** The severity level of the log entry */
  level: LogLevel
  /** The log message content */
  message: LogMessage
}

/**
 * Callback function type for handling log records.
 *
 * @param payload - The log record containing level and message
 */
type LoggerFn = (payload: RecordPayload) => void

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
