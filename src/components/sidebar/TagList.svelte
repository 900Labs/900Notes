<script lang="ts">
  import { tagStore, discoveryStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import * as api from '../../lib/api'

  let {
    onFilter,
    activeFilter,
  }: {
    onFilter: (tagId: string | null) => void
    activeFilter: string | null
  } = $props()

  let newTagName = $state('')
  let showInput = $state(false)
  let showCreateGroup = $state(false)
  let newGroupName = $state('')
  let expandedGroups = $state<Set<string>>(new Set())

  async function handleCreate() {
    const name = newTagName.trim()
    if (!name) return
    await tagStore.createTag(name)
    newTagName = ''
    showInput = false
    await discoveryStore.loadTagGroups()
    await discoveryStore.loadUngroupedTags()
  }

  async function handleCreateGroup() {
    const name = newGroupName.trim()
    if (!name) return
    await discoveryStore.createTagGroup(name, null)
    newGroupName = ''
    showCreateGroup = false
  }

  async function handleDeleteGroup(id: string) {
    await discoveryStore.deleteTagGroup(id)
    await discoveryStore.loadUngroupedTags()
  }

  function toggleGroup(id: string) {
    if (expandedGroups.has(id)) {
      expandedGroups.delete(id)
    } else {
      expandedGroups.add(id)
    }
    expandedGroups = new Set(expandedGroups)
  }

  $effect(() => {
    discoveryStore.loadTagGroups()
    discoveryStore.loadUngroupedTags()
  })
</script>

<div class="space-y-0.5">
  <button
    onclick={() => onFilter(null)}
    class="w-full text-left px-2 py-1.5 rounded text-sm {!activeFilter ? 'bg-accent/10 text-accent' : 'hover:bg-gray-200 dark:hover:bg-gray-700'}"
  >
    {$t('sidebar.allPages')}
  </button>

  {#each discoveryStore.tagGroups as group (group.id)}
    <div class="mt-1">
      <div class="flex items-center gap-1 px-2 py-1 rounded text-xs font-semibold text-gray-500 uppercase tracking-wide hover:bg-gray-200 dark:hover:bg-gray-700">
        <button onclick={() => toggleGroup(group.id)} class="flex items-center gap-1 flex-1 text-left">
          <span class="text-[10px]">{expandedGroups.has(group.id) ? '▼' : '▶'}</span>
          <span
            class="w-2 h-2 rounded-full shrink-0"
            style="background: {group.color || '#2f6fce'}"
          ></span>
          <span class="truncate flex-1">{group.name}</span>
        </button>
        <button
          onclick={() => handleDeleteGroup(group.id)}
          class="text-gray-400 hover:text-red-500 text-xs"
          title={$t('discovery.deleteGroup')}
        >✕</button>
      </div>
      {#if expandedGroups.has(group.id)}
        {@const tagsPromise = api.getTagsInGroup(group.id)}
        <div class="ml-3 mt-0.5 space-y-0.5">
          {#await tagsPromise}
            <p class="text-xs text-gray-400 px-2">...</p>
          {:then groupTags}
            {#each groupTags as tag (tag.id)}
              <button
                onclick={() => onFilter(activeFilter === tag.id ? null : tag.id)}
                class="w-full text-left px-2 py-1.5 rounded text-sm flex items-center gap-2 {activeFilter === tag.id ? 'bg-accent/10 text-accent' : 'hover:bg-gray-200 dark:hover:bg-gray-700'}"
              >
                <span
                  class="w-2.5 h-2.5 rounded-full shrink-0"
                  style="background: {tag.color || '#2f6fce'}"
                ></span>
                <span class="truncate">{tag.name}</span>
              </button>
            {/each}
          {/await}
        </div>
      {/if}
    </div>
  {/each}

  {#if discoveryStore.tagGroups.length > 0 && discoveryStore.ungroupedTags.length > 0}
    <div class="mt-1">
      <span class="px-2 py-1 text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('discovery.ungrouped')}</span>
    </div>
  {/if}
  {#each discoveryStore.ungroupedTags as tag (tag.id)}
    <button
      onclick={() => onFilter(activeFilter === tag.id ? null : tag.id)}
      class="w-full text-left px-2 py-1.5 rounded text-sm flex items-center gap-2 {activeFilter === tag.id ? 'bg-accent/10 text-accent' : 'hover:bg-gray-200 dark:hover:bg-gray-700'}"
    >
      <span
        class="w-2.5 h-2.5 rounded-full shrink-0"
        style="background: {tag.color || '#2f6fce'}"
      ></span>
      <span class="truncate">{tag.name}</span>
    </button>
  {/each}

  {#if discoveryStore.tagGroups.length === 0 && discoveryStore.ungroupedTags.length === 0}
    {#each tagStore.tags as tag (tag.id)}
      <button
        onclick={() => onFilter(activeFilter === tag.id ? null : tag.id)}
        class="w-full text-left px-2 py-1.5 rounded text-sm flex items-center gap-2 {activeFilter === tag.id ? 'bg-accent/10 text-accent' : 'hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >
        <span
          class="w-2.5 h-2.5 rounded-full shrink-0"
          style="background: {tag.color || '#2f6fce'}"
        ></span>
        <span class="truncate">{tag.name}</span>
      </button>
    {/each}
  {/if}

  {#if showCreateGroup}
    <div class="flex gap-1 px-1 py-1">
      <input
        type="text"
        bind:value={newGroupName}
        onkeydown={(e) => { if (e.key === 'Enter') handleCreateGroup(); if (e.key === 'Escape') showCreateGroup = false }}
        placeholder={$t('discovery.groupName')}
        class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent"
      />
      <button onclick={handleCreateGroup} class="px-2 py-1 text-xs rounded bg-accent text-white">+</button>
    </div>
  {/if}

  {#if showInput}
    <div class="flex gap-1 px-1 py-1">
      <input
        type="text"
        bind:value={newTagName}
        onkeydown={(e) => { if (e.key === 'Enter') handleCreate(); if (e.key === 'Escape') showInput = false }}
        placeholder={$t('tag.namePlaceholder')}
        class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 focus:outline-none focus:border-accent"
      />
      <button onclick={handleCreate} class="px-2 py-1 text-xs rounded bg-accent text-white">+</button>
    </div>
  {/if}

  <div class="flex gap-2 px-1 pt-1">
    <button
      onclick={() => (showInput = !showInput)}
      class="text-xs text-gray-400 hover:text-accent"
    >
      + {$t('editor.addTag')}
    </button>
    <button
      onclick={() => (showCreateGroup = !showCreateGroup)}
      class="text-xs text-gray-400 hover:text-accent"
    >
      + {$t('discovery.createGroup')}
    </button>
  </div>
</div>
