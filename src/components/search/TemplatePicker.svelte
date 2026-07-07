<script lang="ts">
  import { onMount } from 'svelte'
  import { templateStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { Template } from '../../lib/types'

  let {
    onSelect,
    onClose,
  }: {
    onSelect: (template: Template) => void
    onClose: () => void
  } = $props()

  let selected = $state(0)

  onMount(async () => {
    await templateStore.loadTemplates()
  })

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selected = Math.min(selected + 1, templateStore.templates.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selected = Math.max(selected - 1, 0)
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const tpl = templateStore.templates[selected]
      if (tpl) onSelect(tpl)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 bg-black/35 dark:bg-black/60 backdrop-blur-sm flex items-start justify-center pt-[12vh]"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose() }}
  role="presentation"
>
  <div
    class="bg-white dark:bg-gray-900 rounded-lg shadow-[0_24px_80px_rgba(0,0,0,0.22)] dark:shadow-[0_24px_90px_rgba(0,0,0,0.55)] w-full max-w-md overflow-hidden border border-gray-200 dark:border-gray-700"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <h3 class="text-sm font-semibold">{$t('template.newFrom')}</h3>
    </div>

    <div class="max-h-80 overflow-y-auto p-2">
      {#if templateStore.templates.length === 0}
        <p class="text-sm text-gray-400 px-3 py-6 text-center">{$t('template.empty')}</p>
      {:else}
        {#each templateStore.templates as tpl, i (tpl.id)}
          <button
            onclick={() => onSelect(tpl)}
            onmouseenter={() => (selected = i)}
            class="w-full text-left px-3 py-2.5 rounded-md flex items-center gap-3 {i === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
          >
            <span class="text-xl">{tpl.icon}</span>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">{tpl.name}</div>
              <div class="text-xs text-gray-400">
                {tpl.category === 'built-in' ? $t('template.builtIn') : $t('template.custom')}
              </div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</div>
