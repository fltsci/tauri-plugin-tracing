'use strict';

var core = require('@tauri-apps/api/core');
var event = require('@tauri-apps/api/event');

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
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, "");
};
/**
 * Circular replacer for JSON.parse
 * @returns Circular replacer function
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse#description
 */
function getCircularReplacer() {
    const ancestors = [];
    return function (_key, value) {
        if (typeof value !== "object" || value === null) {
            return value;
        }
        // `this` is the object that value is contained in,
        // i.e., its direct parent.
        // @ts-expect-error -- this type is meant to be unknown, this is a debug container
        while (ancestors.length > 0 && ancestors.at(-1) !== this) {
            ancestors.pop();
        }
        if (ancestors.includes(value)) {
            return "[Circular]";
        }
        ancestors.push(value);
        return value;
    };
}
const cleanUntypedValue = (value) => stripAnsi(JSON.stringify(value, getCircularReplacer()));
const cleanMessage = (message) => {
    const safeMessage = [];
    if (typeof message === "string") {
        safeMessage.push(stripAnsi(message));
    }
    else if (Array.isArray(message)) {
        for (const msg of message) {
            safeMessage.push(stripAnsi(msg));
        }
    }
    else if (typeof message === "object") {
        for (const [key, value] of Object.entries(message)) {
            safeMessage.push(`${stripAnsi(key)}: ${cleanUntypedValue(value)}`);
        }
    }
    else {
        error(`Unhandled type: message is not a string, array, or object, message is ${typeof message}`);
    }
    // I normally avoid type assertions, but LogMessage is an alias for string[] when managed as above
    return safeMessage;
};
// function getCallerLocation(stack?: string) {
//   if (!stack) {
//     console.log("stack is undefined, returning");
//     return;
//   }
//   if (stack.startsWith("Error")) {
//     // Assume it's Chromium V8
//     //
//     // Error
//     //     at baz (filename.js:10:15)
//     //     at bar (filename.js:6:3)
//     //     at foo (filename.js:2:3)
//     //     at filename.js:13:1
//     const lines = stack.split("\n");
//     // Find the third line (caller's caller of the current location)
//     const callerLine = lines[2]?.trim();
//     if (!callerLine) {
//       return;
//     }
//     const regex =
//       /at\s+(?<functionName>.*?)\s+\((?<fileName>.*?):(?<lineNumber>\d+):(?<columnNumber>\d+)\)/;
//     const match = callerLine.match(regex);
//     if (!match) {
//       // Handle cases where the regex does not match (e.g., last line without function name)
//       const regexNoFunction =
//         /at\s+(?<fileName>.*?):(?<lineNumber>\d+):(?<columnNumber>\d+)/;
//       const matchNoFunction = callerLine.match(regexNoFunction);
//       if (matchNoFunction) {
//         const { fileName, lineNumber, columnNumber } =
//           matchNoFunction.groups as {
//             fileName: string;
//             lineNumber: string;
//             columnNumber: string;
//           };
//         return `<anonymous>@${fileName}:${lineNumber}:${columnNumber}`;
//       }
//     } else {
//       const { functionName, fileName, lineNumber, columnNumber } =
//         match.groups as {
//           functionName: string;
//           fileName: string;
//           lineNumber: string;
//           columnNumber: string;
//         };
//       return `${functionName}@${fileName}:${lineNumber}:${columnNumber}`;
//     }
//   } else {
//     // Assume it's Webkit JavaScriptCore, example:
//     //
//     // baz@filename.js:10:24
//     // bar@filename.js:6:6
//     // foo@filename.js:2:6
//     // global code@filename.js:13:4
//     const traces = stack.split("\n").map((line) => line.split("@"));
//     // console.log("stack does not start with Error; traces: ", traces);
//     // const filtered = traces.filter(([name, location]) => {
//     //   return name.length > 0 && location !== "[native code]";
//     // });
//     // console.log("filtered: ", filtered);
//     // Find the third line (caller's caller of the current location)
//     return traces[2]?.filter((v) => v.length > 0).join("@");
//   }
// }
async function log(level, 
// options?: LogOptions,
callStack, ...msg) {
    // const location = getCallerLocation(new Error().stack);
    const message = cleanMessage(msg);
    // const { file, line, keyValues } = options ?? {};
    return await core.invoke("plugin:tracing|log", {
        level,
        message,
        callStack,
        // file,
        // line,
        // keyValues,
    });
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
async function error(...message) {
    await log(LogLevel.Error, new Error().stack, ...message);
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
async function warn(...message) {
    await log(LogLevel.Warn, new Error().stack, ...message);
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
async function info(...message) {
    await log(LogLevel.Info, new Error().stack, ...message);
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
async function debug(...message) {
    await log(LogLevel.Debug, new Error().stack, ...message);
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
async function trace(...message) {
    await log(LogLevel.Trace, new Error().stack, ...message);
}
/**
 * Attaches a listener for the log, and calls the passed function for each log entry.
 * @param fn
 *
 * @returns a function to cancel the listener.
 */
async function attachLogger(fn) {
    return await event.listen("tracing://log", (event) => {
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
                // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
                throw new Error(`unknown log level ${level}`);
        }
    });
}

exports.attachConsole = attachConsole;
exports.attachLogger = attachLogger;
exports.debug = debug;
exports.error = error;
exports.info = info;
exports.trace = trace;
exports.warn = warn;
