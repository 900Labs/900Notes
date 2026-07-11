export class CloseCoordinator {
  private closing = false

  constructor(
    private readonly flush: () => Promise<void>,
    private readonly close: () => Promise<void>,
    private readonly onError: (error: unknown) => void,
  ) {}

  async requestClose(): Promise<boolean> {
    if (this.closing) return false
    this.closing = true
    try {
      await this.flush()
      await this.close()
      return true
    } catch (error) {
      this.closing = false
      this.onError(error)
      return false
    }
  }
}
