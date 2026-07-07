<script lang="ts">
  import { tagStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'

  let { pageId }: { pageId: string } = $props()

  let showInput = $state(false)
  let searchQuery = $state('')
  let selectedTagIds = $state<string[]>([])

  let availableTags = $derived(
    tagStore.tags.filter(
      (tag) =>
        !selectedTagIds.includes(tag.id) &&
        tag.name.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  )

  // Load current page tags when pageId changes
  let lastPageId = ''
  $effect(() => {
    if (pageId !== lastPageId) {
      lastPageId = pageId
      tagStore.loadPageTags(pageId).then(() => {
        selectedTagIds = tagStore.pageTags.map((t) => t.id)
      })
    }
  })

  async function addTag(tagId: string) {
    selectedTagIds = [...selectedTagIds, tagId]
    await tagStore.setPageTags(pageId, selectedTagIds)
    showInput = false
    searchQuery = ''
  }

  async function removeTag(tagId: string) {
    selectedTagIds = selectedTagIds.filter((id) => id !== tagId)
    await tagStore.setPageTags(pageId, selectedTagIds)
  }

  async function createAndAddTag() {
    const name = searchQuery.trim()
    if (!name) return
    const tag = await tagStore.createTag(name)
    await addTag(tag.id)
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && showInput && searchQuery.trim() && availableTags.length === 0) {
      e.preventDefault()
      createAndAddTag()
    } else if (e.key === 'Enter' && showInput && availableTags.length > 0) {
      e.preventDefault()
      addTag(availableTags[0].id)
    } else if (e.key === 'Escape') {
      showInput = false
      searchQuery = ''
    }
  }
</script>

<div class="flex flex-wrap items-center gap-1.5" onkeydown={handleKeydown} role="toolbar" aria-label="{$t('editor.addTag')}" tabindex="0">
  {#each tagStore.pageTags as tag (tag.id)}
    <button
      onclick={() => removeTag(tag.id)}
      class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium hover:opacity-70"
      style="background: {tag.color || '#2f6fce'}20; color: {tag.color || '#2f6fce'}"
    >
      <span class="w-1.5 h-1.5 rounded-full" style="background: {tag.color || '#2f6fce'}"></span>
      {tag.name}
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/each}

  {#if showInput}
    <div class="relative">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={$t('editor.addTag')}
        class="px-2 py-0.5 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent w-32"
      />
      {#if availableTags.length > 0}
        <div class="absolute top-full left-0 mt-1 z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg py-1 min-w-[160px] max-h-40 overflow-y-auto">
          {#each availableTags as tag (tag.id)}
            <button
              onclick={() => addTag(tag.id)}
              class="w-full text-left px-2 py-1 text-xs hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-1.5"
            >
              <span class="w-2 h-2 rounded-full" style="background: {tag.color || '#2f6fce'}"></span>
              {tag.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <button
      onclick={() => (showInput = true)}
      class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700"
    >
      + {$t('editor.addTag')}
    </button>
  {/if}
</div>
