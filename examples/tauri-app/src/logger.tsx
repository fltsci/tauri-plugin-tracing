import { useEffect } from 'react'
import {
  attachLogger,
  debug,
  error,
  info,
  time,
  timeEnd,
  trace,
  warn,
  type LogMessage
} from '@fltsci/tauri-plugin-tracing'
import type { UnlistenFn } from '@tauri-apps/api/event'
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
  logger: (...message: LogMessage) => void,
  includeOriginal = false
) => {
  const original = console[fnName]
  console[fnName] = (...message: LogMessage) => {
    if (includeOriginal) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      original(...message)
    }
    if (!javascriptLogLevelEnumSchema.options.includes(fnName)) {
      warn(`No Rust equivalent for console.${fnName}(); using WebView logger`)
      return original
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    logger(...message)
  }
  return attachLogger(console[fnName])
}

const original = {
  log: console.log,
  debug: console.debug,
  info: console.info,
  warn: console.warn,
  error: console.error,
  time: console.time,
  timeEnd: console.timeEnd,
  trace: console.trace
}

const forwarded = {
  log: info,
  debug: debug,
  info: info,
  warn: warn,
  error: error,
  trace: trace
}

const unlisten = (fn: UnlistenFn | undefined) => {
  fn?.()
}

const unlistenAll = (res: (UnlistenFn | undefined)[]) => {
  for (const fn of res) {
    fn?.()
  }
}

const useLogger = () => {
  useEffect(() => {
    // const unlistenFns: ReturnType<typeof forwardConsole>[] = []
    // console.time = time
    // console.timeEnd = timeEnd
    // for (const [key, value] of Object.entries(forwarded)) {
    //   unlistenFns.push(forwardConsole(key as JavascriptLogLevel, value))
    // }
    // return () => {
    //   console.time = original.time
    //   console.timeEnd = original.timeEnd
    //   for (const [key, value] of Object.entries(original)) {
    //     forwardConsole(key as JavascriptLogLevel, value)
    //       .then(unlisten)
    //       .catch(console.error)
    //   }
    //   Promise.all(unlistenFns).then(unlistenAll).catch(console.error)
    // }
  }, [])
}

export { useLogger }
