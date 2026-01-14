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
export { LogLevel, type LogMessage, type LoggerFn, type RecordPayload } from './types';
export { trace, debug, info, warn, error } from './log';
export { attachLogger, attachConsole } from './listener';
export { formatPrintf, getCircularReplacer } from './utils';
export { generateFlamegraph, generateFlamechart } from './flamegraph';
