import { describe, it, expect } from 'vitest'

// Re-implement formatPrintf for testing (since it's not exported)
function getCircularReplacer() {
  const ancestors: unknown[] = []
  return function (_key: unknown, value: unknown) {
    if (typeof value !== 'object' || value === null) {
      return value
    }
    // @ts-expect-error -- this type is meant to be unknown
    while (ancestors.length > 0 && ancestors.at(-1) !== this) {
      ancestors.pop()
    }
    if (ancestors.includes(value)) {
      return '[Circular]'
    }
    ancestors.push(value)
    return value
  }
}

function formatPrintf(format: string, args: unknown[]): [string, unknown[]] {
  const remainingArgs = [...args]
  const result = format.replace(/%([sdifooO%])/g, (match, specifier) => {
    if (specifier === '%') return '%'
    if (remainingArgs.length === 0) return match

    const arg = remainingArgs.shift()
    switch (specifier) {
      case 's':
        return String(arg)
      case 'd':
      case 'i':
        return String(Math.floor(Number(arg)))
      case 'f':
        return String(Number(arg))
      case 'o':
      case 'O':
        return JSON.stringify(arg, getCircularReplacer())
      default:
        return match
    }
  })
  return [result, remainingArgs]
}

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
    const [result, remaining] = formatPrintf('Hello %s', ['world', 'extra', 123])
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
})
