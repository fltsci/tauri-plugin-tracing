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
import { type UnlistenFn } from '@tauri-apps/api/event';
/**
 * A log message consisting of one or more values.
 *
 * Mirrors the variadic signature of `console.log`, allowing multiple
 * arguments to be passed and concatenated in the log output.
 */
export type LogMessage = [
    ...Parameters<typeof console.log>[0],
    ...Parameters<typeof console.log>
];
/**
 * Log severity levels.
 *
 * These levels correspond to the tracing crate's Level enum in Rust.
 * Lower values indicate more verbose (less severe) logs.
 */
declare enum LogLevel {
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
export declare function time(label: string): void;
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
export declare function timeEnd(label: string): void;
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
/**
 * Payload structure for log records emitted via events.
 *
 * Used when listening to log events from the Rust backend.
 */
interface RecordPayload {
    /** The severity level of the log entry */
    level: LogLevel;
    /** The log message content */
    message: LogMessage;
}
/**
 * Callback function type for handling log records.
 *
 * @param payload - The log record containing level and message
 */
type LoggerFn = (payload: RecordPayload) => void;
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
export declare function attachLogger(fn: LoggerFn): Promise<UnlistenFn>;
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
export declare function attachConsole(): Promise<UnlistenFn>;
export {};
