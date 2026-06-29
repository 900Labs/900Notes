<script lang="ts">
  import { backlinkStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'

  let {
    pageId,
    onNavigate,
  }: {
    pageId: string
    onNavigate: (pageId: string) => void
  } = $props()

  let lastPageId = ''
  $effect(() => {
    if (pageId !== lastPageId) {
      lastPageId = pageId
      backlinkStore.loadBacklinks(pageId)
    }
  })
</script>

<aside class="w-64 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 flex flex-col overflow-hidden shrink-0">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-800">
    <h3 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">{$t('editor.backlinks')}</h3>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if backlinkStore.backlinks.length === 0}
      <p class="text-xs text-gray-400 dark:text-gray-600 px-2 py-4">{$t('editor.noBacklinks')}</p>
    {:else}
      <div class="space-y-1">
        {#each backlinkStore.backlinks as link (link.id)}
          <button
            onclick={() => onNavigate(link.sourcePageId)}
            class="w-full text-left p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 group transition-colors"
          >
            <div class="flex items-center gap-1.5">
              {#if link.sourcePageIcon}<span>{link.sourcePageIcon}</span>{/if}
              <span class="text-sm font-medium truncate text-gray-700 dark:text-gray-300">{link.sourcePageTitle}</span>
            </div>
            <p class="text-xs text-gray-400 dark:text-gray-600 mt-0.5 truncate">{link.linkText}</p>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</aside>
