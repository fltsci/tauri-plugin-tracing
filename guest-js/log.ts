/**
 * Logging functions for sending messages to the Rust backend.
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { LogLevel, type LogMessage } from './types'
import { cleanMessage } from './utils'

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
