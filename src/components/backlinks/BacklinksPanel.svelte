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

<aside class="w-64 h-full bg-gray-50 dark:bg-gray-800/50 border-l border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('editor.backlinks')}</h3>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if backlinkStore.backlinks.length === 0}
      <p class="text-xs text-gray-400 px-2 py-4">{$t('editor.noBacklinks')}</p>
    {:else}
      <div class="space-y-1">
        {#each backlinkStore.backlinks as link (link.id)}
          <button
            onclick={() => onNavigate(link.sourcePageId)}
            class="w-full text-left p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 group"
          >
            <div class="flex items-center gap-1.5">
              {#if link.sourcePageIcon}<span>{link.sourcePageIcon}</span>{/if}
              <span class="text-sm font-medium truncate">{link.sourcePageTitle}</span>
            </div>
            <p class="text-xs text-gray-400 mt-0.5 truncate">{link.linkText}</p>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</aside>
