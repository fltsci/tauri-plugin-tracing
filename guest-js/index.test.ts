import { describe, it, expect } from 'vitest'
import { formatPrintf, getCircularReplacer } from './index'

describe('formatPrintf', () => {
  it('handles %s string substitution', () => {
    const [result, remaining] = formatPrintf('Hello %s!', ['world'])
    expect(result).toBe('Hello world!')
    expect(remaining).toEqual([])
  })

  it('handles %d integer substitution', () => {
    const [result, remaining] = formatPrintf('Count: %d', [42])
    expect(result).toBe('Count: 42')
    expect(remaining).toEqual([])
  })

  it('handles %d with float (floors it)', () => {
    const [result, remaining] = formatPrintf('Count: %d', [3.7])
    expect(result).toBe('Count: 3')
    expect(remaining).toEqual([])
  })

  it('handles %i integer substitution', () => {
    const [result, remaining] = formatPrintf('Value: %i', [99])
    expect(result).toBe('Value: 99')
    expect(remaining).toEqual([])
  })

  it('handles %f float substitution', () => {
    const [result, remaining] = formatPrintf('Pi: %f', [3.14159])
    expect(result).toBe('Pi: 3.14159')
    expect(remaining).toEqual([])
  })

  it('handles %o object substitution', () => {
    const [result, remaining] = formatPrintf('Data: %o', [{ key: 'value' }])
    expect(result).toBe('Data: {"key":"value"}')
    expect(remaining).toEqual([])
  })

  it('handles %O object substitution', () => {
    const [result, remaining] = formatPrintf('Data: %O', [{ a: 1 }])
    expect(result).toBe('Data: {"a":1}')
    expect(remaining).toEqual([])
  })

  it('handles %% escape', () => {
    const [result, remaining] = formatPrintf('100%% complete', [])
    expect(result).toBe('100% complete')
    expect(remaining).toEqual([])
  })

  it('handles multiple specifiers', () => {
    const [result, remaining] = formatPrintf('%s has %d items', ['Cart', 5])
    expect(result).toBe('Cart has 5 items')
    expect(remaining).toEqual([])
  })

  it('returns remaining args when more args than specifiers', () => {
    const [result, remaining] = formatPrintf('Hello %s', [
      'world',
      'extra',
      123
    ])
    expect(result).toBe('Hello world')
    expect(remaining).toEqual(['extra', 123])
  })

  it('preserves specifier when no args available', () => {
    const [result, remaining] = formatPrintf('Hello %s %s', ['world'])
    expect(result).toBe('Hello world %s')
    expect(remaining).toEqual([])
  })

  it('handles empty format string', () => {
    const [result, remaining] = formatPrintf('', ['unused'])
    expect(result).toBe('')
    expect(remaining).toEqual(['unused'])
  })

  it('handles no specifiers', () => {
    const [result, remaining] = formatPrintf('Plain text', ['unused'])
    expect(result).toBe('Plain text')
    expect(remaining).toEqual(['unused'])
  })

  it('handles circular references in objects', () => {
    const obj: Record<string, unknown> = { name: 'test' }
    obj.self = obj
    const [result, remaining] = formatPrintf('Circular: %o', [obj])
    expect(result).toBe('Circular: {"name":"test","self":"[Circular]"}')
    expect(remaining).toEqual([])
  })

  it('handles null and undefined', () => {
    const [result1] = formatPrintf('%s %s', [null, undefined])
    expect(result1).toBe('null undefined')

    const [result2] = formatPrintf('%o', [null])
    expect(result2).toBe('null')
  })

  it('handles %c CSS styling (strips styling)', () => {
    const [result, remaining] = formatPrintf('%cHello', ['color: red'])
    expect(result).toBe('Hello')
    expect(remaining).toEqual([])
  })

  it('handles multiple %c CSS specifiers', () => {
    const [result, remaining] = formatPrintf('%c[·] %cReact Scan', [
      'font-weight:bold;color:#7a68e8;font-size:20px;',
      'font-weight:bold;font-size:14px;'
    ])
    expect(result).toBe('[·] React Scan')
    expect(remaining).toEqual([])
  })

  it('handles %c mixed with other specifiers', () => {
    const [result, remaining] = formatPrintf('%cCount: %d', ['color: blue', 42])
    expect(result).toBe('Count: 42')
    expect(remaining).toEqual([])
  })
})

describe('getCircularReplacer', () => {
  it('handles non-circular objects', () => {
    const obj = { a: 1, b: { c: 2 } }
    const result = JSON.stringify(obj, getCircularReplacer())
    expect(result).toBe('{"a":1,"b":{"c":2}}')
  })

  it('replaces circular references with [Circular]', () => {
    const obj: Record<string, unknown> = { name: 'test' }
    obj.self = obj
    const result = JSON.stringify(obj, getCircularReplacer())
    expect(result).toBe('{"name":"test","self":"[Circular]"}')
  })

  it('handles deeply nested circular references', () => {
    type NestedObj = { a: { b: { c: unknown } } }
    const obj: NestedObj = { a: { b: { c: {} } } }
    obj.a.b.c = obj
    const result = JSON.stringify(obj, getCircularReplacer())
    expect(result).toBe('{"a":{"b":{"c":"[Circular]"}}}')
  })

  it('handles null values', () => {
    const obj = { a: null, b: 1 }
    const result = JSON.stringify(obj, getCircularReplacer())
    expect(result).toBe('{"a":null,"b":1}')
  })

  it('handles arrays', () => {
    const arr = [1, 2, { nested: true }]
    const result = JSON.stringify(arr, getCircularReplacer())
    expect(result).toBe('[1,2,{"nested":true}]')
  })
})
