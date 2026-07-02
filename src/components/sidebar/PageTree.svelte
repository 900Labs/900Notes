<script lang="ts">
  import type { PageTreeNodeMeta } from '../../lib/types'
  import { t } from '../../i18n'
  import PageTree from './PageTree.svelte'

  let {
    tree,
    onSelectPage,
    onNewPage,
    onDeletePage,
    onDuplicatePage,
    onToggleFavorite,
    currentPageId,
    favoriteIds = new Set<string>(),
    depth = 0,
  }: {
    tree: PageTreeNodeMeta[]
    onSelectPage: (id: string) => void
    onNewPage: (parentId: string | null) => void
    onDeletePage: (id: string) => void
    onDuplicatePage: (id: string) => void
    onToggleFavorite: (id: string) => void
    currentPageId?: string
    favoriteIds?: Set<string>
    depth?: number
  } = $props()

  let expanded = $state<Record<string, boolean>>({})
  let menuOpen = $state<string | null>(null)

  function toggleExpand(id: string) {
    expanded[id] = !expanded[id]
  }
</script>

<div class="space-y-0.5">
  {#each tree as node (node.page.id)}
    <div>
      <div
        class="flex items-center group rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 {currentPageId === node.page.id ? 'bg-accent/10 text-accent' : ''}"
        style="padding-left: {depth * 12 + 4}px"
      >
        {#if node.children.length > 0}
          <button
            onclick={() => toggleExpand(node.page.id)}
            aria-label="Toggle expand"
            class="p-0.5 rounded hover:bg-gray-300 dark:hover:bg-gray-600 shrink-0"
          >
            <svg
              class="w-3 h-3 transition-transform {expanded[node.page.id] ? 'rotate-90' : ''}"
              fill="none" stroke="currentColor" viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        {:else}
          <span class="w-4 shrink-0"></span>
        {/if}

        <button
          onclick={() => onSelectPage(node.page.id)}
          oncontextmenu={(e) => { e.preventDefault(); menuOpen = menuOpen === node.page.id ? null : node.page.id }}
          class="flex-1 text-left px-1.5 py-1 truncate flex items-center gap-1"
        >
          {#if node.page.icon}<span>{node.page.icon}</span>{/if}
          <span class="truncate">{node.page.title || $t('editor.titlePlaceholder')}</span>
        </button>

        <div class="relative">
          <button
            onclick={() => (menuOpen = menuOpen === node.page.id ? null : node.page.id)}
            aria-label="Page menu"
            class="p-0.5 rounded hover:bg-gray-300 dark:hover:bg-gray-600 opacity-0 group-hover:opacity-100 shrink-0"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01" />
            </svg>
          </button>
          {#if menuOpen === node.page.id}
            <div class="absolute right-0 top-6 z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg py-1 min-w-[140px]">
              <button
                onclick={() => { onNewPage(node.page.id); expanded[node.page.id] = true; menuOpen = null }}
                class="w-full text-left px-3 py-1.5 text-xs hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                + {$t('sidebar.newPage')}
              </button>
              <button
                onclick={() => { onDuplicatePage(node.page.id); menuOpen = null }}
                class="w-full text-left px-3 py-1.5 text-xs hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                {$t('editor.duplicate')}
              </button>
              <button
                onclick={() => { onToggleFavorite(node.page.id); menuOpen = null }}
                class="w-full text-left px-3 py-1.5 text-xs hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                {favoriteIds.has(node.page.id) ? $t('favorites.remove') : $t('favorites.add')}
              </button>
              <button
                onclick={() => { onDeletePage(node.page.id); menuOpen = null }}
                class="w-full text-left px-3 py-1.5 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
              >
                {$t('editor.delete')}
              </button>
            </div>
          {/if}
        </div>
      </div>

      {#if expanded[node.page.id] && node.children.length > 0}
        <PageTree
          tree={node.children}
          {onSelectPage}
          {onNewPage}
          {onDeletePage}
          {onDuplicatePage}
          {onToggleFavorite}
          {currentPageId}
          {favoriteIds}
          depth={depth + 1}
        />
      {/if}
    </div>
  {/each}
</div>
