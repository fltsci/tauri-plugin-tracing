/**
 * Type definitions for the tracing plugin.
 * @module
 */

/**
 * A log message consisting of one or more values.
 *
 * Mirrors the variadic signature of `console.log`, allowing multiple
 * arguments to be passed and concatenated in the log output.
 */
export type LogMessage = [
  ...Parameters<typeof console.log>[0],
  ...Parameters<typeof console.log>
]

/**
 * Log severity levels.
 *
 * These levels correspond to the tracing crate's Level enum in Rust.
 * Lower values indicate more verbose (less severe) logs.
 */
export enum LogLevel {
  /**
   * The "trace" level.
   *
   * Designates very low priority, often extremely verbose, information.
   */
  Trace = 1,
  /**
   * The "debug" level.
   *
   * Designates lower priority information.
   */
  Debug = 2,
  /**
   * The "info" level.
   *
   * Designates useful information.
   */
  Info = 3,
  /**
   * The "warn" level.
   *
   * Designates hazardous situations.
   */
  Warn = 4,
  /**
   * The "error" level.
   *
   * Designates very serious errors.
   */
  Error = 5
}

/**
 * Payload structure for log records emitted via events.
 *
 * Used when listening to log events from the Rust backend.
 */
export interface RecordPayload {
  /** The severity level of the log entry */
  level: LogLevel
  /** The log message content */
  message: LogMessage
}

/**
 * Callback function type for handling log records.
 *
 * @param payload - The log record containing level and message
 */
export type LoggerFn = (payload: RecordPayload) => void
