import { describe, expect, it, vi } from 'vitest'
import { CloseCoordinator } from './close-coordinator'

describe('CloseCoordinator', () => {
  it('waits for an in-flight save and closes exactly once', async () => {
    let finishSave!: () => void
    const flush = vi.fn(() => new Promise<void>((resolve) => { finishSave = resolve }))
    const close = vi.fn(async () => {})
    const coordinator = new CloseCoordinator(flush, close, () => {})

    const first = coordinator.requestClose()
    const second = coordinator.requestClose()
    expect(close).not.toHaveBeenCalled()
    finishSave()

    expect(await first).toBe(true)
    expect(await second).toBe(false)
    expect(flush).toHaveBeenCalledOnce()
    expect(close).toHaveBeenCalledOnce()
  })

  it('keeps the window open when saving fails and allows retry', async () => {
    const error = new Error('disk full')
    const flush = vi.fn()
      .mockRejectedValueOnce(error)
      .mockResolvedValueOnce(undefined)
    const close = vi.fn(async () => {})
    const onError = vi.fn()
    const coordinator = new CloseCoordinator(flush, close, onError)

    expect(await coordinator.requestClose()).toBe(false)
    expect(close).not.toHaveBeenCalled()
    expect(onError).toHaveBeenCalledWith(error)
    expect(await coordinator.requestClose()).toBe(true)
    expect(close).toHaveBeenCalledOnce()
  })
})
