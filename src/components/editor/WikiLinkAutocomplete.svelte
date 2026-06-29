<script lang="ts">
  import { t } from '../../i18n'

  let {
    titles,
    query,
    onSelect,
    onClose,
  }: {
    titles: { id: string; title: string }[]
    query: string
    onSelect: (title: string, pageId: string | null) => void
    onClose: () => void
  } = $props()

  let filtered = $derived(
    titles
      .filter((p) => p.title.toLowerCase().includes(query.toLowerCase()))
      .slice(0, 8),
  )

  let selected = $state(0)

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selected = Math.min(selected + 1, filtered.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selected = Math.max(selected - 1, 0)
    } else if (e.key === 'Enter') {
      e.preventDefault()
      if (filtered[selected]) {
        onSelect(filtered[selected].title, filtered[selected].id)
      }
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 min-w-[240px] max-h-64 overflow-y-auto"
  style="left: 50%; top: 40%; transform: translate(-50%, -50%)"
>
  {#if filtered.length === 0}
    <div class="px-3 py-2 text-sm text-gray-400">{$t('link.noMatch')}</div>
  {:else}
    {#each filtered as item, i}
      <button
        onclick={() => onSelect(item.title, item.id)}
        onmouseenter={() => (selected = i)}
        class="w-full text-left px-3 py-2 text-sm {i === selected ? 'bg-accent/10 text-accent' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
      >
        <span class="font-medium">{item.title}</span>
      </button>
    {/each}
  {/if}
</div>
