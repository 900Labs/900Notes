<script lang="ts">
  import type { PageMetadata, PageTreeNodeMeta } from '../lib/types'

  let {
    pageTree,
    recentPages,
    onAction,
    onPageSelect,
  }: {
    pageTree: PageTreeNodeMeta[]
    recentPages: PageMetadata[]
    onAction: (action: string) => void
    onPageSelect: (pageId: string) => void
  } = $props()

  function countPages(nodes: PageTreeNodeMeta[]): number {
    return nodes.reduce((count, node) => count + 1 + countPages(node.children), 0)
  }

  function countPinned(nodes: PageTreeNodeMeta[]): number {
    return nodes.reduce((count, node) => count + (node.page.pinned ? 1 : 0) + countPinned(node.children), 0)
  }

  let totalPages = $derived(countPages(pageTree))
  let pinnedPages = $derived(countPinned(pageTree))
  let today = $derived(new Date().toLocaleDateString(undefined, {
    weekday: 'long',
    month: 'short',
    day: 'numeric',
  }))

  let primaryActions = [
    { label: 'New page', detail: 'Blank note or parent page', action: 'newPage' },
    { label: 'Daily note', detail: 'Open or create today', action: 'dailyNote' },
    { label: 'Template', detail: 'Start from a repeatable structure', action: 'newFromTemplate' },
    { label: 'Import', detail: 'Notion, Obsidian, Evernote, Roam', action: 'openDataSettings' },
  ]

  let workspaceActions = [
    { label: 'Knowledge graph', action: 'toggleGraph' },
    { label: 'Smart folders', action: 'toggleSmartFolders' },
    { label: 'Favorites', action: 'toggleFavorites' },
    { label: 'Local sync', action: 'openSyncSettings' },
    { label: 'Security', action: 'openSecuritySettings' },
    { label: 'Plugins', action: 'openPluginsSettings' },
  ]
</script>

<div class="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
  <div class="max-w-5xl mx-auto px-8 py-8 space-y-8">
    <section class="flex flex-col gap-5 md:flex-row md:items-end md:justify-between">
      <div>
        <p class="text-sm text-gray-500 dark:text-gray-400">{today}</p>
        <h1 class="mt-1 text-2xl font-semibold tracking-normal">Workspace</h1>
      </div>
      <div class="grid grid-cols-3 gap-2 w-full md:w-auto">
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">Pages</div>
          <div class="text-xl font-semibold">{totalPages}</div>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">Pinned</div>
          <div class="text-xl font-semibold">{pinnedPages}</div>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">Recent</div>
          <div class="text-xl font-semibold">{recentPages.length}</div>
        </div>
      </div>
    </section>

    <section>
      <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {#each primaryActions as action}
          <button
            onclick={() => onAction(action.action)}
            class="min-h-28 rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-4 text-left hover:border-accent/50 hover:bg-accent/5 transition-colors"
          >
            <div class="text-sm font-semibold">{action.label}</div>
            <div class="mt-2 text-sm text-gray-500 dark:text-gray-400 leading-snug">{action.detail}</div>
          </button>
        {/each}
      </div>
    </section>

    <section class="grid gap-6 lg:grid-cols-[minmax(0,1.5fr)_minmax(280px,1fr)]">
      <div>
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-sm font-semibold">Recent pages</h2>
          <button
            onclick={() => onAction('openCommandPalette')}
            class="text-xs text-accent hover:underline"
          >Search all</button>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 divide-y divide-gray-100 dark:divide-gray-800 overflow-hidden">
          {#if recentPages.length === 0}
            <div class="px-4 py-8 text-sm text-gray-500 dark:text-gray-400">No pages yet.</div>
          {:else}
            {#each recentPages as page (page.id)}
              <button
                onclick={() => onPageSelect(page.id)}
                class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                <span class="w-6 text-center">{page.icon || '#'}</span>
                <span class="min-w-0 flex-1">
                  <span class="block text-sm font-medium truncate">{page.title || 'Untitled'}</span>
                  <span class="block text-xs text-gray-500 dark:text-gray-400">{new Date(page.updatedAt).toLocaleString()}</span>
                </span>
              </button>
            {/each}
          {/if}
        </div>
      </div>

      <div>
        <h2 class="text-sm font-semibold mb-2">Workspace tools</h2>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-2 grid grid-cols-2 gap-1">
          {#each workspaceActions as action}
            <button
              onclick={() => onAction(action.action)}
              class="px-3 py-2 rounded-md text-sm text-left hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              {action.label}
            </button>
          {/each}
        </div>
      </div>
    </section>
  </div>
</div>
