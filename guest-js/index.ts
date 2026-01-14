/**
 * @module
 *
 * Tauri plugin for structured logging via the tracing crate.
 *
 * This module provides logging functions that bridge JavaScript logs to Rust's
 * tracing infrastructure.
 *
 * @example
 * ```ts
 * import { info, debug, error } from '@fltsci/tauri-plugin-tracing';
 *
 * info('Application started');
 * debug('Debug details', { user: 'alice' });
 * error('Something went wrong');
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

// Re-export listener functions
export { attachLogger, attachConsole } from './listener'

// Re-export console interception
export { interceptConsole, restoreConsole, takeoverConsole } from './console'

// Re-export utilities (for testing and advanced usage)
export { formatPrintf, getCircularReplacer } from './utils'

// Re-export flamegraph functions
export { generateFlamegraph, generateFlamechart } from './flamegraph'
