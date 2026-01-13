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

// Re-export types
export {
  LogLevel,
  type LogMessage,
  type LoggerFn,
  type RecordPayload
} from './types'

// Re-export logging functions
export { trace, debug, info, warn, error } from './log'

// Re-export timing functions
export { time, timeEnd } from './timing'

// Re-export listener functions
export { attachLogger, attachConsole } from './listener'

// Re-export utilities (for testing and advanced usage)
export { formatPrintf, getCircularReplacer } from './utils'
