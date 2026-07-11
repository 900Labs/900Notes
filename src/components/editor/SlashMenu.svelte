<script lang="ts">
  import { t } from '../../i18n'

  let {
    x,
    y,
    onSelect,
    onClose,
  }: {
    x: number
    y: number
    onSelect: (action: string) => void
    onClose: () => void
  } = $props()

  const items = [
    { action: 'h1', label: 'slash.heading1', icon: 'H1' },
    { action: 'h2', label: 'slash.heading2', icon: 'H2' },
    { action: 'h3', label: 'slash.heading3', icon: 'H3' },
    { action: 'paragraph', label: 'slash.paragraph', icon: '¶' },
    { action: 'bulletList', label: 'slash.bulletList', icon: '•' },
    { action: 'orderedList', label: 'slash.orderedList', icon: '1.' },
    { action: 'todo', label: 'slash.todo', icon: '☐' },
    { action: 'codeBlock', label: 'slash.code', icon: '{ }' },
    { action: 'quote', label: 'slash.quote', icon: '"' },
    { action: 'divider', label: 'slash.divider', icon: '-' },
    { action: 'audio', label: 'slash.audio', icon: '🎙' },
  ]

  let selected = $state(0)

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selected = (selected + 1) % items.length
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selected = (selected - 1 + items.length) % items.length
    } else if (e.key === 'Enter') {
      e.preventDefault()
      onSelect(items[selected].action)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 min-w-[200px]"
  style="left: {x}px; top: {y}px"
>
  {#each items as item, i}
    <button
      onclick={() => onSelect(item.action)}
      onmouseenter={() => (selected = i)}
      class="w-full flex items-center gap-3 px-3 py-2 text-sm {i === selected ? 'bg-accent/10 text-accent' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
    >
      <span class="w-6 text-center font-mono text-xs text-gray-400">{item.icon}</span>
      <span>{$t(item.label)}</span>
    </button>
  {/each}
</div>
