<script lang="ts">
  import { historyStore, pageStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'

  let {
    onPageRestored,
  }: {
    onPageRestored: (id: string) => void
  } = $props()

  let selectedRevisionId = $state<string | null>(null)

  async function handleRestore(revisionId: string) {
    if (!confirm($t('history.restoreConfirm'))) return
    const page = await historyStore.restoreRevision(revisionId)
    pageStore.currentPage = page
    onPageRestored(page.id)
  }

  async function handleDelete(revisionId: string) {
    await historyStore.deleteRevision(revisionId)
    if (selectedRevisionId === revisionId) {
      selectedRevisionId = null
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso)
    return d.toLocaleString()
  }

  function getPreview(content: string): string {
    try {
      const doc = JSON.parse(content)
      const text = extractText(doc)
      return text.slice(0, 200)
    } catch {
      return content.slice(0, 200)
    }
  }

  function extractText(node: any): string {
    if (node.text) return node.text
    if (node.content) return node.content.map(extractText).join(' ')
    return ''
  }
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-900">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-800">
    <h2 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">{$t('history.title')}</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-2 space-y-1">
    {#if historyStore.revisions.length === 0}
      <p class="text-xs text-gray-400 dark:text-gray-600 px-2 py-4">{$t('history.empty')}</p>
    {:else}
      {#each historyStore.revisions as revision (revision.id)}
        <div
          class="p-2 rounded-md border cursor-pointer transition-colors {selectedRevisionId === revision.id ? 'border-accent bg-accent/5' : 'border-gray-200 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800'}"
          onclick={() => (selectedRevisionId = revision.id)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedRevisionId = revision.id } }}
          role="button"
          tabindex="0"
        >
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs text-gray-500 dark:text-gray-400">{formatDate(revision.createdAt)}</span>
            <div class="flex gap-1">
              <button
                onclick={(e) => { e.stopPropagation(); handleRestore(revision.id) }}
                class="text-xs text-accent hover:underline"
              >{$t('history.restore')}</button>
              <button
                onclick={(e) => { e.stopPropagation(); handleDelete(revision.id) }}
                class="text-xs text-red-400 hover:text-red-600"
              >✕</button>
            </div>
          </div>
          <p class="text-sm font-medium truncate text-gray-700 dark:text-gray-300">{revision.title || $t('editor.titlePlaceholder')}</p>
          {#if selectedRevisionId === revision.id}
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-4">{getPreview(revision.content)}</p>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
