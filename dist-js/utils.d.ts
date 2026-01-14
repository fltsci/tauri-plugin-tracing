/**
 * Utility functions for message formatting and sanitization.
 * @module
 */
import type { LogMessage } from './types';
/**
 * Strips ANSI escape codes from a string.
 *
 * Used to sanitize log messages that may contain terminal color codes
 * before sending them to the Rust backend.
 *
 * @param s - The value to strip ANSI codes from
 * @returns The string with all ANSI escape sequences removed
 */
export declare const stripAnsi: (s?: unknown) => string;
/**
 * Creates a replacer function for JSON.stringify that handles circular references.
 *
 * When a circular reference is detected, it is replaced with the string "[Circular]"
 * instead of throwing an error.
 *
 * @returns A replacer function for use with JSON.stringify
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify#the_replacer_parameter
 */
export declare function getCircularReplacer(): (_key: unknown, value: unknown) => unknown;
/**
 * Converts an arbitrary value to a clean string representation.
 *
 * Handles circular references and strips ANSI codes from the result.
 *
 * @param value - Any value to convert to string
 * @returns A JSON string representation with ANSI codes removed
 */
export declare const cleanUntypedValue: (value: unknown) => string;
/**
 * Performs printf-style string formatting like console.log.
 *
 * Supports the following format specifiers:
 * - `%s` - String
 * - `%d`, `%i` - Integer
 * - `%f` - Float
 * - `%o`, `%O` - Object (JSON)
 * - `%c` - CSS styling (consumed but not rendered)
 * - `%%` - Literal percent sign
 *
 * @param format - The format string
 * @param args - Arguments to substitute
 * @returns The formatted string and any remaining arguments
 */
export declare function formatPrintf(format: string, args: unknown[]): [string, unknown[]];
/**
 * Sanitizes a log message for transmission to the Rust backend.
 *
 * Handles printf-style format strings (like console.log), strips ANSI codes,
 * and converts values to safe string representations.
 *
 * @param message - The log message to clean
 * @returns A sanitized LogMessage array
 */
export declare const cleanMessage: (message: LogMessage) => LogMessage;
