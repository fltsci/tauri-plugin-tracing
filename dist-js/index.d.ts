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
export { LogLevel, type LogMessage, type LoggerFn, type RecordPayload } from './types';
export { trace, debug, info, warn, error } from './log';
export { time, timeEnd } from './timing';
export { attachLogger, attachConsole } from './listener';
export { formatPrintf, getCircularReplacer } from './utils';
export { generateFlamegraph, generateFlamechart } from './flamegraph';
