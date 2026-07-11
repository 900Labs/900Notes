import { describe, expect, it, vi } from 'vitest'
import type { Page } from './types'
import {
  mutateAfterCurrentPageFlush,
  restoreRevisionAfterFlush,
} from './page-lifecycle'

const page = { id: 'page-a', title: 'Restored', content: 'revision' } as Page

describe('page lifecycle coordination', () => {
  it('flushes the visible page before duplicating or deleting it', async () => {
    const order: string[] = []
    const result = await mutateAfterCurrentPageFlush(
      'page-a',
      'page-a',
      async () => { order.push('flush') },
      async () => { order.push('mutation'); return 'done' },
    )

    expect(result).toBe('done')
    expect(order).toEqual(['flush', 'mutation'])
  })

  it('aborts a page mutation when flushing fails', async () => {
    const mutation = vi.fn(async () => {})
    await expect(mutateAfterCurrentPageFlush(
      'page-a',
      'page-a',
      async () => { throw new Error('save failed') },
      mutation,
    )).rejects.toThrow('save failed')
    expect(mutation).not.toHaveBeenCalled()
  })

  it('flushes before restore and refreshes the same page afterward', async () => {
    const order: string[] = []
    const shown: Page[] = []

    await restoreRevisionAfterFlush('revision-1', {
      flush: async () => { order.push('flush') },
      restore: async () => { order.push('restore'); return page },
      showRestoredPage: (restored) => { order.push('show'); shown.push(restored) },
      refresh: async (pageId) => { order.push(`refresh:${pageId}`) },
    })

    expect(order).toEqual(['flush', 'restore', 'show', 'refresh:page-a'])
    expect(shown).toEqual([page])
  })

  it('does not restore when the pending editor save fails', async () => {
    const restore = vi.fn(async () => page)
    await expect(restoreRevisionAfterFlush('revision-1', {
      flush: async () => { throw new Error('disk full') },
      restore,
      showRestoredPage: () => {},
      refresh: async () => {},
    })).rejects.toThrow('disk full')
    expect(restore).not.toHaveBeenCalled()
  })
})
