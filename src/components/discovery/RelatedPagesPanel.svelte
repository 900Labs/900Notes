<script lang="ts">
  import { discoveryStore, pageStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'

  let {
    onNavigate,
  }: {
    onNavigate: (id: string) => void
  } = $props()

  let currentPageId = $derived(pageStore.currentPage?.id)

  $effect(() => {
    if (currentPageId) {
      discoveryStore.loadRelatedPages(currentPageId, 10)
    }
  })
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-900">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-800">
    <h2 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">{$t('discovery.relatedPages')}</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-2 space-y-1">
    {#if discoveryStore.relatedPages.length === 0}
      <p class="text-xs text-gray-400 dark:text-gray-600 px-2 py-4">{$t('discovery.noRelated')}</p>
    {:else}
      {#each discoveryStore.relatedPages as page (page.id)}
        <button
          onclick={() => onNavigate(page.id)}
          class="w-full text-left p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          <div class="flex items-center gap-2">
            {#if page.icon}<span class="text-base">{page.icon}</span>{/if}
            <span class="text-sm font-medium truncate flex-1 text-gray-700 dark:text-gray-300">{page.title || $t('editor.titlePlaceholder')}</span>
            <span class="text-xs text-gray-400 dark:text-gray-600">{page.score}</span>
          </div>
          {#if page.reasons.length > 0}
            <div class="flex gap-1 mt-1 flex-wrap">
              {#each page.reasons as reason}
                <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-400">{reason}</span>
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    {/if}
  </div>
</div>
