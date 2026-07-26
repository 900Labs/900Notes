import { describe, expect, it } from 'vitest'
import { sanitizeHref } from './schema'

describe('sanitizeHref', () => {
  it('preserves safe schemes, fragments, and relative urls', () => {
    expect(sanitizeHref('https://example.com/a')).toBe('https://example.com/a')
    expect(sanitizeHref('http://example.com')).toBe('http://example.com')
    expect(sanitizeHref('mailto:user@example.com')).toBe('mailto:user@example.com')
    expect(sanitizeHref('tel:+18005550100')).toBe('tel:+18005550100')
    expect(sanitizeHref('#section')).toBe('#section')
    expect(sanitizeHref('/relative/path')).toBe('/relative/path')
  })

  it('strips dangerous schemes regardless of case or surrounding whitespace', () => {
    expect(sanitizeHref('javascript:alert(1)')).toBe('')
    expect(sanitizeHref('JaVaScRiPt:alert(1)')).toBe('')
    expect(sanitizeHref('  javascript:alert(1)  ')).toBe('')
    expect(sanitizeHref('data:text/html,<script>')).toBe('')
    expect(sanitizeHref('vbscript:msgbox')).toBe('')
  })

  it('treats colon-free text and empty input as safe/empty', () => {
    expect(sanitizeHref('')).toBe('')
    expect(sanitizeHref('plain text')).toBe('plain text')
  })
})
