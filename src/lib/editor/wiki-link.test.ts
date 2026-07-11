import { describe, expect, it } from 'vitest'
import { wikiLinkQueryFromText } from './index'

describe('wiki link autocomplete query', () => {
  it('tracks text after the latest open trigger', () => {
    expect(wikiLinkQueryFromText('See [[Target Pa', 1)).toEqual({
      from: 5,
      query: 'Target Pa',
    })
  })

  it('closes after a completed link or without a trigger', () => {
    expect(wikiLinkQueryFromText('See [[Target]]')).toBeNull()
    expect(wikiLinkQueryFromText('ordinary text')).toBeNull()
  })
})
