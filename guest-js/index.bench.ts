import { bench, describe } from 'vitest'
import { formatPrintf, getCircularReplacer } from './index'

describe('formatPrintf', () => {
  bench('no specifiers', () => {
    formatPrintf('Hello, world!', [])
  })

  bench('single %s', () => {
    formatPrintf('Hello, %s!', ['world'])
  })

  bench('multiple specifiers', () => {
    formatPrintf('%s has %d items at %f price', ['Cart', 5, 19.99])
  })

  bench('%o object (small)', () => {
    formatPrintf('Data: %o', [{ key: 'value' }])
  })

  bench('%o object (medium)', () => {
    formatPrintf('Data: %o', [
      {
        user: 'alice',
        id: 12345,
        roles: ['admin', 'user'],
        metadata: { created: '2024-01-15', active: true }
      }
    ])
  })

  bench('%o object (large)', () => {
    const largeObj = {
      users: Array.from({ length: 10 }, (_, i) => ({
        id: i,
        name: `user${i}`,
        email: `user${i}@example.com`,
        settings: { theme: 'dark', notifications: true }
      }))
    }
    formatPrintf('Data: %o', [largeObj])
  })

  bench('mixed format string', () => {
    formatPrintf('User %s (id=%d) performed %s at %f%% completion: %o', [
      'alice',
      42,
      'upload',
      87.5,
      { file: 'report.pdf', size: 1024 }
    ])
  })

  bench('escaped percent %%', () => {
    formatPrintf('Progress: %d%% complete', [75])
  })
})

describe('getCircularReplacer', () => {
  bench('simple object', () => {
    const obj = { a: 1, b: 'string', c: true }
    JSON.stringify(obj, getCircularReplacer())
  })

  bench('nested object', () => {
    const obj = {
      level1: {
        level2: {
          level3: {
            value: 'deep'
          }
        }
      }
    }
    JSON.stringify(obj, getCircularReplacer())
  })

  bench('array of objects', () => {
    const arr = Array.from({ length: 10 }, (_, i) => ({
      id: i,
      name: `item${i}`
    }))
    JSON.stringify(arr, getCircularReplacer())
  })

  bench('circular reference', () => {
    const obj: Record<string, unknown> = { name: 'test' }
    obj.self = obj
    JSON.stringify(obj, getCircularReplacer())
  })

  bench('deeply nested circular', () => {
    type NestedObj = { a: { b: { c: unknown } } }
    const obj: NestedObj = { a: { b: { c: {} } } }
    obj.a.b.c = obj
    JSON.stringify(obj, getCircularReplacer())
  })
})

// Compare with native JSON.stringify (baseline)
describe('baseline comparisons', () => {
  const simpleObj = { a: 1, b: 'string', c: true }
  const nestedObj = {
    level1: { level2: { level3: { value: 'deep' } } }
  }

  bench('JSON.stringify (simple)', () => {
    JSON.stringify(simpleObj)
  })

  bench('getCircularReplacer (simple)', () => {
    JSON.stringify(simpleObj, getCircularReplacer())
  })

  bench('JSON.stringify (nested)', () => {
    JSON.stringify(nestedObj)
  })

  bench('getCircularReplacer (nested)', () => {
    JSON.stringify(nestedObj, getCircularReplacer())
  })
})
