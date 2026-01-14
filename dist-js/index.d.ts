/**
 * Tauri plugin for structured logging via the tracing crate.
 *
 * ## Basic logging
 * ```ts
 * import { info, debug, error } from '@fltsci/tauri-plugin-tracing';
 *
 * info('Application started');
 * debug('Debug details', { user: 'alice' });
 * error('Something went wrong');
 * ```
 *
 * ## Console integration
 *
 * | Scenario                        | Function              |
 * |---------------------------------|-----------------------|
 * | See Rust logs in browser        | `attachConsole()`     |
 * | Send JS logs to Rust/files      | `interceptConsole()`  |
 * | Unified logging, see everything | `takeoverConsole()`   |
 *
 * - **`attachConsole()`** - Rust logs → browser console
 * - **`interceptConsole()`** - JS console → Rust tracing
 * - **`takeoverConsole()`** - Both directions: JS → Rust → browser console
 *
 * @module
 */
export { LogLevel, type LogMessage, type LoggerFn, type RecordPayload } from './types';
export { trace, debug, info, warn, error } from './log';
export { attachLogger, attachConsole } from './listener';
export { interceptConsole, restoreConsole, takeoverConsole } from './console';
export { formatPrintf, getCircularReplacer } from './utils';
export { generateFlamegraph, generateFlamechart } from './flamegraph';
