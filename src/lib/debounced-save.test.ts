import { describe, expect, it, vi } from 'vitest'
import { DebouncedPageSave } from './debounced-save'

describe('DebouncedPageSave', () => {
  it('keeps the originating page id across navigation', async () => {
    vi.useFakeTimers()
    const saved: Array<{ pageId: string; content: string }> = []
    const saver = new DebouncedPageSave(async (payload) => { saved.push(payload) }, () => {})

    saver.schedule({ pageId: 'page-a', content: 'A edit' })
    await saver.flush()
    saver.schedule({ pageId: 'page-b', content: 'B edit' })
    await vi.runAllTimersAsync()

    expect(saved).toEqual([
      { pageId: 'page-a', content: 'A edit' },
      { pageId: 'page-b', content: 'B edit' },
    ])
    vi.useRealTimers()
  })

  it('coalesces edits without changing their page identity', async () => {
    vi.useFakeTimers()
    const save = vi.fn(async () => {})
    const saver = new DebouncedPageSave(save, () => {})
    saver.schedule({ pageId: 'page-a', content: 'first' })
    saver.schedule({ pageId: 'page-a', content: 'latest' })
    await vi.runAllTimersAsync()
    expect(save).toHaveBeenCalledOnce()
    expect(save).toHaveBeenCalledWith({ pageId: 'page-a', content: 'latest' })
    vi.useRealTimers()
  })

  it('reports and rejects failed saves so navigation can abort', async () => {
    const error = new Error('disk full')
    const onError = vi.fn()
    const saver = new DebouncedPageSave(async () => { throw error }, onError, 0)
    saver.schedule({ pageId: 'page-a', content: 'keep me' })
    await expect(saver.flush()).rejects.toBe(error)
    expect(onError).toHaveBeenCalledWith(error)
  })

  it('retries a timer-triggered failure on the next explicit flush', async () => {
    vi.useFakeTimers()
    const error = new Error('disk full')
    let failing = true
    const save = vi.fn(async () => {
      if (failing) throw error
    })
    const saver = new DebouncedPageSave(save, () => {}, 10)
    saver.schedule({ pageId: 'page-a', content: 'keep me' })
    await vi.runAllTimersAsync()
    expect(save).toHaveBeenCalledOnce()

    failing = false
    await expect(saver.flush()).resolves.toBeUndefined()
    expect(save).toHaveBeenCalledTimes(2)
    expect(save).toHaveBeenLastCalledWith({ pageId: 'page-a', content: 'keep me' })
    vi.useRealTimers()
  })

  it('retries a failed payload before saving newer pending content', async () => {
    const error = new Error('offline')
    let failing = true
    const attempts: Array<{ pageId: string; content: string }> = []
    const saver = new DebouncedPageSave(async (payload) => {
      attempts.push(payload)
      if (failing) throw error
    }, () => {}, 0)

    saver.schedule({ pageId: 'page-a', content: 'failed edit' })
    await expect(saver.flush()).rejects.toBe(error)
    saver.schedule({ pageId: 'page-b', content: 'newer edit' })

    failing = false
    await saver.flush()
    expect(attempts).toEqual([
      { pageId: 'page-a', content: 'failed edit' },
      { pageId: 'page-a', content: 'failed edit' },
      { pageId: 'page-b', content: 'newer edit' },
    ])
  })

  it('waits for an in-flight save when flushed again', async () => {
    let finish!: () => void
    const save = vi.fn(() => new Promise<void>((resolve) => { finish = resolve }))
    const saver = new DebouncedPageSave(save, () => {}, 0)
    saver.schedule({ pageId: 'page-a', content: 'latest' })
    const first = saver.flush()
    const second = saver.flush()
    await Promise.resolve()
    finish()
    await Promise.all([first, second])
    expect(save).toHaveBeenCalledOnce()
  })
})
