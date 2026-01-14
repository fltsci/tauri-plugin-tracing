'use strict';

var core = require('@tauri-apps/api/core');
var event = require('@tauri-apps/api/event');

/**
 * Type definitions for the tracing plugin.
 * @module
 */
/**
 * Log severity levels.
 *
 * These levels correspond to the tracing crate's Level enum in Rust.
 * Lower values indicate more verbose (less severe) logs.
 */
exports.LogLevel = void 0;
(function (LogLevel) {
    /**
     * The "trace" level.
     *
     * Designates very low priority, often extremely verbose, information.
     */
    LogLevel[LogLevel["Trace"] = 1] = "Trace";
    /**
     * The "debug" level.
     *
     * Designates lower priority information.
     */
    LogLevel[LogLevel["Debug"] = 2] = "Debug";
    /**
     * The "info" level.
     *
     * Designates useful information.
     */
    LogLevel[LogLevel["Info"] = 3] = "Info";
    /**
     * The "warn" level.
     *
     * Designates hazardous situations.
     */
    LogLevel[LogLevel["Warn"] = 4] = "Warn";
    /**
     * The "error" level.
     *
     * Designates very serious errors.
     */
    LogLevel[LogLevel["Error"] = 5] = "Error";
})(exports.LogLevel || (exports.LogLevel = {}));

/**
 * Utility functions for message formatting and sanitization.
 * @module
 */
/**
 * Strips ANSI escape codes from a string.
 *
 * Used to sanitize log messages that may contain terminal color codes
 * before sending them to the Rust backend.
 *
 * @param s - The value to strip ANSI codes from
 * @returns The string with all ANSI escape sequences removed
 */
const stripAnsi = (s) => {
    return String(s).replace(
    // TODO: Investigate security/detect-unsafe-regex
    // biome-ignore lint/suspicious/noControlCharactersInRegex: this is in the tauri log plugin
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
};
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
    const ancestors = [];
    return function (_key, value) {
        if (typeof value !== 'object' || value === null) {
            return value;
        }
        // `this` is the object that value is contained in,
        // i.e., its direct parent.
        // @ts-expect-error -- this type is meant to be unknown, this is a debug container
        while (ancestors.length > 0 && ancestors.at(-1) !== this) {
            ancestors.pop();
        }
        if (ancestors.includes(value)) {
            return '[Circular]';
        }
        ancestors.push(value);
        return value;
    };
}
/**
 * Converts an arbitrary value to a clean string representation.
 *
 * Handles circular references and strips ANSI codes from the result.
 *
 * @param value - Any value to convert to string
 * @returns A JSON string representation with ANSI codes removed
 */
const cleanUntypedValue = (value) => stripAnsi(JSON.stringify(value, getCircularReplacer()));
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
function formatPrintf(format, args) {
    const remainingArgs = [...args];
    const result = format.replace(/%([sdifooO%])/g, (match, specifier) => {
        if (specifier === '%')
            return '%';
        if (remainingArgs.length === 0)
            return match;
        const arg = remainingArgs.shift();
        switch (specifier) {
            case 's':
                return String(arg);
            case 'd':
            case 'i':
                return String(Math.floor(Number(arg)));
            case 'f':
                return String(Number(arg));
            case 'o':
            case 'O':
                return JSON.stringify(arg, getCircularReplacer());
            default:
                return match;
        }
    });
    return [result, remainingArgs];
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
const cleanMessage = (message) => {
    const safeMessage = [];
    if (typeof message === 'string') {
        safeMessage.push(stripAnsi(message));
    }
    else if (Array.isArray(message)) {
        // Check if first argument is a string that might be a format string
        if (message.length > 1
            && typeof message[0] === 'string'
            && message[0].includes('%')) {
            const [formatted, remaining] = formatPrintf(message[0], message.slice(1));
            safeMessage.push(stripAnsi(formatted));
            for (const arg of remaining) {
                safeMessage.push(stripAnsi(arg));
            }
        }
        else {
            for (const msg of message) {
                safeMessage.push(stripAnsi(msg));
            }
        }
    }
    else if (typeof message === 'object') {
        for (const [key, value] of Object.entries(message)) {
            safeMessage.push(`${stripAnsi(key)}: ${cleanUntypedValue(value)}`);
        }
    }
    else {
        // Import would cause circular dependency, log directly
        console.error(`Unhandled type: message is not a string, array, or object, message is ${typeof message}`);
    }
    return safeMessage;
};

/**
 * Logging functions for sending messages to the Rust backend.
 * @module
 */
/**
 * Internal function to send a log message to the Rust backend.
 *
 * Captures the current call stack for source location information
 * and invokes the tracing plugin command.
 *
 * @param level - The severity level of the log
 * @param msg - The message parts to log
 */
function log(level, ...msg) {
    const message = cleanMessage(msg);
    core.invoke('plugin:tracing|log', {
        level,
        message,
        callStack: new Error().stack
    }).catch(console.error);
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
function error(...message) {
    log(exports.LogLevel.Error, ...message);
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
function warn(...message) {
    log(exports.LogLevel.Warn, ...message);
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
function info(...message) {
    log(exports.LogLevel.Info, ...message);
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
function debug(...message) {
    log(exports.LogLevel.Debug, ...message);
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
function trace(...message) {
    log(exports.LogLevel.Trace, ...message);
}

/**
 * Event listeners for receiving logs from the Rust backend.
 * @module
 */
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
async function attachLogger(fn) {
    return await event.listen('tracing://log', (event) => {
        const { level } = event.payload;
        const message = cleanMessage(event.payload.message);
        fn({ message, level });
    });
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
async function attachConsole() {
    return await attachLogger(({ level, message }) => {
        switch (level) {
            case exports.LogLevel.Trace:
                console.log(message);
                break;
            case exports.LogLevel.Debug:
                console.debug(message);
                break;
            case exports.LogLevel.Info:
                console.info(message);
                break;
            case exports.LogLevel.Warn:
                console.warn(message);
                break;
            case exports.LogLevel.Error:
                console.error(message);
                break;
            default:
                throw new Error(`unknown log level ${level}`);
        }
    });
}

/**
 * Console interception to route browser console calls to Rust tracing.
 * @module
 */
let originalConsole = null;
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
function interceptConsole(options = {}) {
    const { preserveOriginal = false } = options;
    // Store original console methods
    originalConsole = {
        log: console.log.bind(console),
        debug: console.debug.bind(console),
        info: console.info.bind(console),
        warn: console.warn.bind(console),
        error: console.error.bind(console),
        trace: console.trace.bind(console)
    };
    const wrap = (level, original) => {
        const logFn = { trace, debug, info, warn, error }[level];
        return (...args) => {
            logFn(...args);
            if (preserveOriginal) {
                original(...args);
            }
        };
    };
    console.log = wrap('debug', originalConsole.log);
    console.debug = wrap('debug', originalConsole.debug);
    console.info = wrap('info', originalConsole.info);
    console.warn = wrap('warn', originalConsole.warn);
    console.error = wrap('error', originalConsole.error);
    console.trace = wrap('trace', originalConsole.trace);
    return restoreConsole;
}
/**
 * Restores the original console methods after interception.
 *
 * This is automatically returned by interceptConsole(), but can also be
 * called directly if needed.
 */
function restoreConsole() {
    if (originalConsole) {
        console.log = originalConsole.log;
        console.debug = originalConsole.debug;
        console.info = originalConsole.info;
        console.warn = originalConsole.warn;
        console.error = originalConsole.error;
        console.trace = originalConsole.trace;
        originalConsole = null;
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
async function takeoverConsole() {
    // Store original console methods before interception
    const savedConsole = {
        log: console.log.bind(console),
        debug: console.debug.bind(console),
        info: console.info.bind(console),
        warn: console.warn.bind(console),
        error: console.error.bind(console),
        trace: console.trace.bind(console)
    };
    // Intercept console calls and route to Rust (don't preserve original - we'll handle output ourselves)
    interceptConsole({ preserveOriginal: false });
    // Listen for Rust tracing events and output using the ORIGINAL console methods
    // This avoids infinite loops since we use savedConsole, not the intercepted console
    const unlisten = await event.listen('tracing://log', (event) => {
        const { level } = event.payload;
        const message = cleanMessage(event.payload.message);
        switch (level) {
            case exports.LogLevel.Trace:
                savedConsole.log(message);
                break;
            case exports.LogLevel.Debug:
                savedConsole.debug(message);
                break;
            case exports.LogLevel.Info:
                savedConsole.info(message);
                break;
            case exports.LogLevel.Warn:
                savedConsole.warn(message);
                break;
            case exports.LogLevel.Error:
                savedConsole.error(message);
                break;
        }
    });
    // Return cleanup function
    return () => {
        unlisten();
        restoreConsole();
    };
}

/**
 * Flamegraph and flamechart generation functions.
 *
 * These functions require the `flamegraph` feature to be enabled in the Rust plugin
 * and `with_flamegraph()` to be called on the Builder.
 *
 * @module
 */
/**
 * Generates a flamegraph SVG from recorded profiling data.
 *
 * Flamegraphs collapse identical stack frames and sort them, making them
 * ideal for identifying hot paths in long-running or multi-threaded applications.
 *
 * @returns The path to the generated SVG file
 * @throws If no profiling data is available or SVG generation fails
 *
 * @example
 * ```ts
 * import { generateFlamegraph } from '@fltsci/tauri-plugin-tracing';
 *
 * // After running your application with profiling enabled...
 * const svgPath = await generateFlamegraph();
 * console.log(`Flamegraph saved to: ${svgPath}`);
 * ```
 */
async function generateFlamegraph() {
    return await core.invoke('plugin:tracing|generate_flamegraph');
}
/**
 * Generates a flamechart SVG from recorded profiling data.
 *
 * Unlike flamegraphs, flamecharts preserve the exact ordering of events
 * as they were recorded, making it easier to see when each span occurs
 * relative to others. This is useful for understanding the temporal flow
 * of execution.
 *
 * @returns The path to the generated SVG file
 * @throws If no profiling data is available or SVG generation fails
 *
 * @example
 * ```ts
 * import { generateFlamechart } from '@fltsci/tauri-plugin-tracing';
 *
 * // After running your application with profiling enabled...
 * const svgPath = await generateFlamechart();
 * console.log(`Flamechart saved to: ${svgPath}`);
 * ```
 */
async function generateFlamechart() {
    return await core.invoke('plugin:tracing|generate_flamechart');
}

exports.attachConsole = attachConsole;
exports.attachLogger = attachLogger;
exports.debug = debug;
exports.error = error;
exports.formatPrintf = formatPrintf;
exports.generateFlamechart = generateFlamechart;
exports.generateFlamegraph = generateFlamegraph;
exports.getCircularReplacer = getCircularReplacer;
exports.info = info;
exports.interceptConsole = interceptConsole;
exports.restoreConsole = restoreConsole;
exports.takeoverConsole = takeoverConsole;
exports.trace = trace;
exports.warn = warn;
