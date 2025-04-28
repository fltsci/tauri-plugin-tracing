import { useEffect } from 'react'
import {
  attachLogger,
  debug,
  error,
  info,
  trace,
  warn,
  type LogMessage
} from 'tauri-plugin-tracing'
import { z } from 'zod'

const javascriptLogLevelEnumSchema = z.enum([
  'log',
  'debug',
  'info',
  'warn',
  'error',
  'trace'
])

type JavascriptLogLevel = z.infer<typeof javascriptLogLevelEnumSchema>

const forwardConsole = (
  fnName: JavascriptLogLevel,
  logger: (...message: LogMessage) => Promise<void>,
  includeOriginal = false
) => {
  const original = console[fnName]
  console[fnName] = (...message: LogMessage) => {
    if (includeOriginal) {
      original(...message)
    }
    if (!javascriptLogLevelEnumSchema.options.includes(fnName)) {
      warn(`No Rust equivalent for console.${fnName}(); using WebView logger`)
      return original
    }
    logger(...message)
  }
  return attachLogger(console[fnName])
}

const original: typeof console = { ...console }

const forwarded: {
  [L in JavascriptLogLevel]: (...message: LogMessage) => Promise<void>
} = {
  log: info,
  debug: debug,
  info: info,
  warn: warn,
  error: error,
  trace: trace
}

const useLogger = () => {
  useEffect(() => {
    for (const [key, value] of Object.entries(forwarded)) {
      forwardConsole(key as JavascriptLogLevel, value)
    }
    return () => {
      for (const [key, value] of Object.entries(original)) {
        forwardConsole(key as JavascriptLogLevel, value)
      }
    }
  }, [])
}

export { useLogger }
