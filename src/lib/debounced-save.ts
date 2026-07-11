export type SavePayload = { pageId: string; content: string }

/**
 * A page-bound debouncer. Each queued payload keeps the page id that produced
 * it, so navigation can never redirect an old edit into the newly opened page.
 */
export class DebouncedPageSave {
  private timer: ReturnType<typeof setTimeout> | null = null
  private pending: SavePayload | null = null
  private retry: SavePayload | null = null
  private drainPromise: Promise<void> | null = null

  constructor(
    private readonly save: (payload: SavePayload) => Promise<void>,
    private readonly onError: (error: unknown) => void,
    private readonly delayMs = 500,
  ) {}

  schedule(payload: SavePayload) {
    this.pending = { ...payload }
    if (this.timer) clearTimeout(this.timer)
    this.timer = setTimeout(() => {
      this.timer = null
      void this.drain().catch(() => {
        // The failed payload remains in `retry`. A later timer or explicit
        // flush will attempt it again, while onError reports this attempt.
      })
    }, this.delayMs)
  }

  private drain(): Promise<void> {
    if (this.drainPromise) return this.drainPromise

    const operation = this.drainQueuedPayloads()
    this.drainPromise = operation
    operation.then(
      () => {
        if (this.drainPromise === operation) this.drainPromise = null
      },
      () => {
        if (this.drainPromise === operation) this.drainPromise = null
      },
    )
    return operation
  }

  private async drainQueuedPayloads(): Promise<void> {
    while (this.retry || this.pending) {
      const payload = this.retry ?? this.pending!
      if (!this.retry) this.pending = null

      try {
        await this.save(payload)
        if (this.retry === payload) this.retry = null
      } catch (error) {
        this.retry = payload
        this.onError(error)
        throw error
      }
    }
  }

  async flush(): Promise<void> {
    if (this.timer) clearTimeout(this.timer)
    this.timer = null
    await this.drain()
  }

  cancel() {
    if (this.timer) clearTimeout(this.timer)
    this.timer = null
    this.pending = null
    this.retry = null
  }
}
