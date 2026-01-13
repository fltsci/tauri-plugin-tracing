/**
 * Logging functions for sending messages to the Rust backend.
 * @module
 */
import { type LogMessage } from './types';
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
export declare function error(...message: LogMessage): void;
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
export declare function warn(...message: LogMessage): void;
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
export declare function info(...message: LogMessage): void;
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
export declare function debug(...message: LogMessage): void;
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
export declare function trace(...message: LogMessage): void;
