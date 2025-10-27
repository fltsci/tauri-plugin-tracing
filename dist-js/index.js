import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

var LogLevel;
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
})(LogLevel || (LogLevel = {}));
const stripAnsi = (s) => {
    return String(s).replace(
    // TODO: Investigate security/detect-unsafe-regex
    // biome-ignore lint/suspicious/noControlCharactersInRegex: this is in the tauri log plugin
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
};
/**
 * Circular replacer for JSON.parse
 * @returns Circular replacer function
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse#description
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
const cleanUntypedValue = (value) => stripAnsi(JSON.stringify(value, getCircularReplacer()));
const cleanMessage = (message) => {
    const safeMessage = [];
    if (typeof message === 'string') {
        safeMessage.push(stripAnsi(message));
    }
    else if (Array.isArray(message)) {
        for (const msg of message) {
            safeMessage.push(stripAnsi(msg));
        }
    }
    else if (typeof message === 'object') {
        for (const [key, value] of Object.entries(message)) {
            safeMessage.push(`${stripAnsi(key)}: ${cleanUntypedValue(value)}`);
        }
    }
    else {
        error(`Unhandled type: message is not a string, array, or object, message is ${typeof message}`);
    }
    return safeMessage;
};
function log(level, ...msg) {
    const message = cleanMessage(msg);
    invoke('plugin:tracing|log', {
        level,
        message,
        callStack: new Error().stack
    }).catch(console.error);
}
function time(label) {
    invoke('plugin:tracing|time', {
        label,
        callStack: new Error().stack
    }).catch(console.error);
}
function timeEnd(label) {
    invoke('plugin:tracing|time_end', {
        label,
        callStack: new Error().stack
    }).catch(console.error);
}
/**
 * Logs a message at the error level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { error } from 'tauri-plugin-tracing';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 */
function error(...message) {
    log(LogLevel.Error, ...message);
}
/**
 * Logs a message at the warn level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { warn } from 'tauri-plugin-tracing';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 */
function warn(...message) {
    log(LogLevel.Warn, ...message);
}
/**
 * Logs a message at the info level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { info } from 'tauri-plugin-tracing';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 */
function info(...message) {
    log(LogLevel.Info, ...message);
}
/**
 * Logs a message at the debug level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { debug } from 'tauri-plugin-tracing';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 */
function debug(...message) {
    log(LogLevel.Debug, ...message);
}
/**
 * Logs a message at the trace level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { trace } from 'tauri-plugin-tracing';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 */
function trace(...message) {
    log(LogLevel.Trace, ...message);
}
/**
 * Attaches a listener for the log, and calls the passed function for each log entry.
 * @param fn
 *
 * @returns a function to cancel the listener.
 */
async function attachLogger(fn) {
    return await listen('tracing://log', (event) => {
        const { level } = event.payload;
        const message = cleanMessage(event.payload.message);
        fn({ message, level });
    });
}
/**
 * Attaches a listener that writes log entries to the console as they come in.
 *
 * @returns a function to cancel the listener.
 */
async function attachConsole() {
    return await attachLogger(({ level, message }) => {
        switch (level) {
            case LogLevel.Trace:
                console.log(message);
                break;
            case LogLevel.Debug:
                console.debug(message);
                break;
            case LogLevel.Info:
                console.info(message);
                break;
            case LogLevel.Warn:
                console.warn(message);
                break;
            case LogLevel.Error:
                console.error(message);
                break;
            default:
                throw new Error(`unknown log level ${level}`);
        }
    });
}

export { attachConsole, attachLogger, debug, error, info, time, timeEnd, trace, warn };
