import type { Page } from './types'

export async function mutateAfterCurrentPageFlush<T>(
  currentPageId: string | null | undefined,
  targetPageId: string,
  flush: (() => Promise<void>) | null,
  mutation: () => Promise<T>,
): Promise<T> {
  if (currentPageId === targetPageId) {
    await flush?.()
  }
  return mutation()
}

export async function mutateAfterFlush<T>(
  flush: (() => Promise<void>) | null,
  mutation: () => Promise<T>,
): Promise<T> {
  await flush?.()
  return mutation()
}

export async function restoreRevisionAfterFlush(
  revisionId: string,
  dependencies: {
    flush: (() => Promise<void>) | null
    restore: (revisionId: string) => Promise<Page>
    showRestoredPage: (page: Page) => void
    refresh: (pageId: string) => Promise<void>
  },
): Promise<Page> {
  await dependencies.flush?.()
  const page = await dependencies.restore(revisionId)
  dependencies.showRestoredPage(page)
  await dependencies.refresh(page.id)
  return page
}
