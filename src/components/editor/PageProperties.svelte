<script lang="ts">
  import { propertyStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'

  let { pageId }: { pageId: string } = $props()

  let showAddForm = $state(false)
  let newKey = $state('')
  let newValue = $state('')
  let editingId = $state<string | null>(null)
  let editValue = $state('')

  let lastPageId = ''
  $effect(() => {
    if (pageId !== lastPageId) {
      lastPageId = pageId
      propertyStore.loadProperties(pageId)
    }
  })

  async function handleAdd() {
    const key = newKey.trim()
    const value = newValue.trim()
    if (!key) return
    await propertyStore.setProperty(pageId, key, value)
    newKey = ''
    newValue = ''
    showAddForm = false
  }

  async function handleSaveEdit(id: string) {
    if (editingId === id) {
      const prop = propertyStore.properties.find((p) => p.id === id)
      if (prop) {
        await propertyStore.setProperty(pageId, prop.key, editValue)
      }
    }
    editingId = null
  }

  async function handleDelete(id: string) {
    await propertyStore.deleteProperty(id, pageId)
  }

  function startEdit(id: string, currentValue: string) {
    editingId = id
    editValue = currentValue
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && showAddForm) {
      e.preventDefault()
      handleAdd()
    } else if (e.key === 'Escape') {
      showAddForm = false
      editingId = null
      newKey = ''
      newValue = ''
    }
  }
</script>

<div class="flex flex-col gap-1" onkeydown={handleKeydown} role="toolbar" aria-label="Page properties" tabindex="0">
  {#if propertyStore.properties.length > 0}
    <div class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center text-sm">
      {#each propertyStore.properties as prop (prop.id)}
        <span class="text-gray-500 font-medium text-xs py-1">{prop.key}</span>
        {#if editingId === prop.id}
          <input
            type="text"
            bind:value={editValue}
            onblur={() => handleSaveEdit(prop.id)}
            class="px-2 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent"
          />
        {:else}
          <button
            onclick={() => startEdit(prop.id, prop.value)}
            class="text-left px-2 py-1 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 truncate"
          >
            {prop.value || '—'}
          </button>
        {/if}
        <button
          onclick={() => handleDelete(prop.id)}
          class="text-gray-400 hover:text-red-500 p-1"
          title={$t('common.delete')}
          aria-label={$t('common.delete')}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/each}
    </div>
  {:else if !showAddForm}
    <p class="text-xs text-gray-400">{$t('properties.empty')}</p>
  {/if}

  {#if showAddForm}
    <div class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center">
      <input
        type="text"
        bind:value={newKey}
        placeholder={$t('properties.addKey')}
        class="px-2 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent"
      />
      <input
        type="text"
        bind:value={newValue}
        placeholder={$t('properties.addValue')}
        class="px-2 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent"
      />
      <button
        onclick={handleAdd}
        class="px-2 py-1 text-sm rounded bg-accent text-white hover:bg-accent-dark"
      >
        {$t('common.save')}
      </button>
    </div>
  {:else}
    <button
      onclick={() => (showAddForm = true)}
      class="text-xs text-gray-400 hover:text-accent self-start"
    >
      + {$t('properties.add')}
    </button>
  {/if}
</div>
