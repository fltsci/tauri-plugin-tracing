/**
 * Utility functions for message formatting and sanitization.
 * @module
 */

import type { LogMessage } from './types'

/**
 * Strips ANSI escape codes from a string.
 *
 * Used to sanitize log messages that may contain terminal color codes
 * before sending them to the Rust backend.
 *
 * @param s - The value to strip ANSI codes from
 * @returns The string with all ANSI escape sequences removed
 */
export const stripAnsi = (s?: unknown): string => {
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
export function getCircularReplacer() {
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
export const cleanUntypedValue = (value: unknown): string =>
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
export function formatPrintf(
  format: string,
  args: unknown[]
): [string, unknown[]] {
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
export const cleanMessage = (message: LogMessage): LogMessage => {
  const safeMessage: string[] = []
  if (typeof message === 'string') {
    safeMessage.push(stripAnsi(message))
  } else if (Array.isArray(message)) {
    // Check if first argument is a string that might be a format string
    if (
      message.length > 1
      && typeof message[0] === 'string'
      && message[0].includes('%')
    ) {
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
    // Import would cause circular dependency, log directly
    console.error(
      `Unhandled type: message is not a string, array, or object, message is ${typeof message}`
    )
  }
  return safeMessage as LogMessage
}
