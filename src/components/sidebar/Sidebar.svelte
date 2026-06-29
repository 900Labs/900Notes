<script lang="ts">
  import { onMount } from 'svelte'
  import PageTree from './PageTree.svelte'
  import TagList from './TagList.svelte'
  import { pageStore, tagStore, settingsStore, searchStore, historyStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import * as api from '../../lib/api'
  import type { SearchResult, SmartFolderRule } from '../../lib/types'

  let {
    onPageSelect,
    onPageCreated,
    onOpenSettings,
  }: {
    onPageSelect: (id: string) => void
    onPageCreated: (id: string) => void
    onOpenSettings: () => void
  } = $props()

  let activeTab = $derived(settingsStore.sidebarTab) as 'pages' | 'tags' | 'recent' | 'trash' | 'smart' | 'favorites'
  let collapsed = $state(false)
  let searchResults = $state<SearchResult[]>([])
  let showCreateSearch = $state(false)
  let newSearchName = $state('')
  let showCreateFolder = $state(false)
  let newFolderName = $state('')
  let newFolderIcon = $state('📁')
  let folderRules = $state<SmartFolderRule[]>([{ field: 'tag', operator: 'equals', value: '' }])

  async function handleNewPage(parentId: string | null = null) {
    const page = await pageStore.createPage('Untitled', parentId)
    onPageCreated(page.id)
  }

  async function handleSelectPage(id: string) {
    onPageSelect(id)
  }

  async function handleEmptyTrash() {
    if (confirm($t('trash.emptyConfirm'))) {
      await pageStore.emptyTrash()
    }
  }

  async function handleRestorePage(id: string) {
    await pageStore.restorePage(id)
  }

  async function handleDeletePage(id: string) {
    await pageStore.deletePage(id)
  }

  async function handleTagFilter(tagId: string | null) {
    settingsStore.activeTagFilter = tagId
  }

  async function handleExecuteSearch(id: string) {
    searchResults = await searchStore.executeSavedSearch(id)
  }

  async function handleSaveSearch() {
    if (!newSearchName.trim()) return
    await searchStore.createSavedSearch(newSearchName, '', null, false)
    newSearchName = ''
    showCreateSearch = false
  }

  async function handleDeleteSearch(id: string) {
    await searchStore.deleteSavedSearch(id)
  }

  async function handleCreateFolder() {
    if (!newFolderName.trim()) return
    const rulesJson = JSON.stringify(folderRules.filter((r) => r.value.trim()))
    await searchStore.createSmartFolder(newFolderName, newFolderIcon, rulesJson)
    newFolderName = ''
    newFolderIcon = '📁'
    folderRules = [{ field: 'tag', operator: 'equals', value: '' }]
    showCreateFolder = false
  }

  async function handleDeleteFolder(id: string) {
    await searchStore.deleteSmartFolder(id)
  }

  async function handleSelectFolder(id: string) {
    await searchStore.loadSmartFolderPages(id)
  }

  function addRule() {
    folderRules = [...folderRules, { field: 'tag', operator: 'equals', value: '' }]
  }

  function removeRule(idx: number) {
    folderRules = folderRules.filter((_, i) => i !== idx)
  }

  async function handleRemoveFavorite(pageId: string) {
    await historyStore.removeFavorite(pageId)
  }

  async function handleFavoriteDragReorder(orderedPageIds: string[]) {
    await historyStore.reorderFavorites(orderedPageIds)
  }
</script>

{#if !collapsed}
  <aside class="w-64 h-full bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
    <!-- Header -->
    <div class="p-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <svg class="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <span class="font-semibold text-sm">{$t('app.title')}</span>
      </div>
      <button onclick={() => (collapsed = true)} class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700" title={$t('sidebar.collapse')}>
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        </svg>
      </button>
    </div>

    <!-- New Page Button -->
    <div class="p-2">
      <button
        onclick={() => handleNewPage(null)}
        class="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-accent text-white text-sm font-medium hover:bg-accent-dark transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {$t('sidebar.newPage')}
      </button>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-gray-200 dark:border-gray-700 px-2">
      <button
        onclick={() => (settingsStore.sidebarTab = 'pages')}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'pages' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('sidebar.pages')}</button>
      <button
        onclick={() => (settingsStore.sidebarTab = 'tags')}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'tags' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('sidebar.tags')}</button>
      <button
        onclick={() => (settingsStore.sidebarTab = 'recent')}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'recent' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('sidebar.recent')}</button>
      <button
        onclick={() => { settingsStore.sidebarTab = 'trash'; pageStore.loadTrash() }}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'trash' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('sidebar.trash')}</button>
      <button
        onclick={() => { settingsStore.sidebarTab = 'smart'; searchStore.loadSavedSearches(); searchStore.loadSmartFolders() }}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'smart' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('smartFolder.title')}</button>
      <button
        onclick={() => { settingsStore.sidebarTab = 'favorites'; historyStore.loadFavorites() }}
        class="px-2 py-1.5 text-xs font-medium {activeTab === 'favorites' ? 'text-accent border-b-2 border-accent' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
      >{$t('favorites.title')}</button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-2">
      {#if activeTab === 'pages'}
        <PageTree
          tree={pageStore.pageTree}
          onSelectPage={handleSelectPage}
          onNewPage={handleNewPage}
          onDeletePage={handleDeletePage}
          currentPageId={pageStore.currentPage?.id}
        />
      {:else if activeTab === 'tags'}
        <TagList onFilter={handleTagFilter} activeFilter={settingsStore.activeTagFilter} />
      {:else if activeTab === 'recent'}
        <div class="space-y-0.5">
          {#each pageStore.recentPages as page (page.id)}
            <button
              onclick={() => handleSelectPage(page.id)}
              class="w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
            >
              {#if page.icon}<span class="mr-1">{page.icon}</span>{/if}
              {page.title || $t('editor.titlePlaceholder')}
            </button>
          {/each}
          {#if pageStore.recentPages.length === 0}
            <p class="text-xs text-gray-400 px-2 py-4">No recent pages</p>
          {/if}
        </div>
      {:else if activeTab === 'trash'}
        <div class="space-y-0.5">
          {#each pageStore.trashPages as page (page.id)}
            <div class="flex items-center group">
              <button
                onclick={() => handleSelectPage(page.id)}
                class="flex-1 text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate text-gray-500"
              >
                {#if page.icon}<span class="mr-1">{page.icon}</span>{/if}
                {page.title || $t('editor.titlePlaceholder')}
              </button>
              <button onclick={() => handleRestorePage(page.id)} class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-0 group-hover:opacity-100" title={$t('trash.restore')}>
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
                </svg>
              </button>
            </div>
          {/each}
          {#if pageStore.trashPages.length === 0}
            <p class="text-xs text-gray-400 px-2 py-4">{$t('trash.noItems')}</p>
          {/if}
          {#if pageStore.trashPages.length > 0}
            <button onclick={handleEmptyTrash} class="w-full mt-2 px-2 py-1.5 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded">
              {$t('trash.empty')}
            </button>
          {/if}
        </div>
      {:else if activeTab === 'smart'}
        <div class="space-y-3">
          <!-- Saved Searches -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('search.saved')}</span>
              <button onclick={() => (showCreateSearch = !showCreateSearch)} class="text-xs text-accent hover:underline">+</button>
            </div>
            {#if showCreateSearch}
              <div class="flex gap-1 mb-2">
                <input
                  type="text"
                  bind:value={newSearchName}
                  placeholder={$t('search.name')}
                  class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700"
                  onkeydown={(e) => e.key === 'Enter' && handleSaveSearch()}
                />
                <button onclick={handleSaveSearch} class="px-2 py-1 text-xs bg-accent text-white rounded">{$t('search.save')}</button>
              </div>
            {/if}
            {#each searchStore.savedSearches as search (search.id)}
              <div class="flex items-center group">
                <button
                  onclick={() => handleExecuteSearch(search.id)}
                  class="flex-1 text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
                >🔍 {search.name}</button>
                <button onclick={() => handleDeleteSearch(search.id)} class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-0 group-hover:opacity-100" title={$t('search.delete')}>
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
              </div>
            {/each}
            {#if searchStore.savedSearches.length === 0}
              <p class="text-xs text-gray-400 px-2 py-2">{$t('search.empty')}</p>
            {/if}
          </div>

          <!-- Search Results -->
          {#if searchResults.length > 0}
            <div>
              <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1 block">{$t('search.execute')} ({searchResults.length})</span>
              {#each searchResults as result (result.id)}
                <button
                  onclick={() => handleSelectPage(result.id)}
                  class="w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
                >
                  {#if result.icon}<span class="mr-1">{result.icon}</span>{/if}
                  {result.title}
                </button>
              {/each}
            </div>
          {/if}

          <!-- Smart Folders -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('smartFolder.title')}</span>
              <button onclick={() => (showCreateFolder = !showCreateFolder)} class="text-xs text-accent hover:underline">+</button>
            </div>
            {#if showCreateFolder}
              <div class="space-y-2 mb-2 p-2 rounded border border-gray-200 dark:border-gray-600">
                <div class="flex gap-1">
                  <input
                    type="text"
                    bind:value={newFolderIcon}
                    class="w-8 px-1 py-1 text-xs text-center rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700"
                  />
                  <input
                    type="text"
                    bind:value={newFolderName}
                    placeholder={$t('smartFolder.name')}
                    class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700"
                  />
                </div>
                {#each folderRules as rule, idx}
                  <div class="flex gap-1 items-center">
                    <select bind:value={rule.field} class="text-xs px-1 py-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700">
                      <option value="tag">{$t('sidebar.tags')}</option>
                      <option value="title">{$t('sidebar.pages')}</option>
                      <option value="created_after">created_after</option>
                      <option value="created_before">created_before</option>
                      <option value="has_property">{$t('smartFolder.field')}</option>
                    </select>
                    <select bind:value={rule.operator} class="text-xs px-1 py-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700">
                      <option value="equals">=</option>
                      <option value="contains">~</option>
                      <option value="starts_with">^</option>
                    </select>
                    <input
                      type="text"
                      bind:value={rule.value}
                      placeholder={$t('smartFolder.value')}
                      class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700"
                    />
                    <button onclick={() => removeRule(idx)} class="text-red-400 hover:text-red-600 text-xs">✕</button>
                  </div>
                {/each}
                <div class="flex gap-1">
                  <button onclick={addRule} class="text-xs text-accent hover:underline">+ {$t('smartFolder.addField')}</button>
                  <div class="flex-1"></div>
                  <button onclick={handleCreateFolder} class="px-2 py-1 text-xs bg-accent text-white rounded">{$t('smartFolder.create')}</button>
                </div>
              </div>
            {/if}
            {#each searchStore.smartFolders as folder (folder.id)}
              <div class="flex items-center group">
                <button
                  onclick={() => handleSelectFolder(folder.id)}
                  class="flex-1 text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
                >{folder.icon} {folder.name}</button>
                <button onclick={() => handleDeleteFolder(folder.id)} class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-0 group-hover:opacity-100" title={$t('smartFolder.delete')}>
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
              </div>
            {/each}
            {#if searchStore.smartFolders.length === 0}
              <p class="text-xs text-gray-400 px-2 py-2">{$t('smartFolder.empty')}</p>
            {/if}
          </div>

          <!-- Smart Folder Pages -->
          {#if searchStore.smartFolderPages.length > 0}
            <div>
              {#each searchStore.smartFolderPages as node (node.page.id)}
                <button
                  onclick={() => handleSelectPage(node.page.id)}
                  class="w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
                >
                  {#if node.page.icon}<span class="mr-1">{node.page.icon}</span>{/if}
                  {node.page.title || $t('editor.titlePlaceholder')}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {:else if activeTab === 'favorites'}
        <div class="space-y-0.5">
          {#each historyStore.favorites as fav (fav.pageId)}
            <div class="flex items-center group">
              <button
                onclick={() => handleSelectPage(fav.pageId)}
                class="flex-1 text-left px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
              >
                <span class="mr-1">⭐</span>
                {fav.pageId}
              </button>
              <button
                onclick={() => handleRemoveFavorite(fav.pageId)}
                class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-0 group-hover:opacity-100"
                title={$t('favorites.remove')}
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
              </button>
            </div>
          {/each}
          {#if historyStore.favorites.length === 0}
            <p class="text-xs text-gray-400 px-2 py-4">{$t('favorites.empty')}</p>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="p-2 border-t border-gray-200 dark:border-gray-700">
      <button
        onclick={onOpenSettings}
        class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        {$t('settings.title')}
      </button>
    </div>
  </aside>
{:else}
  <aside class="w-12 h-full bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col items-center py-3">
    <button onclick={() => (collapsed = false)} class="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700" title={$t('sidebar.expand')}>
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
      </svg>
    </button>
    <button onclick={() => handleNewPage(null)} class="mt-2 p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-accent" title={$t('sidebar.newPage')}>
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </aside>
{/if}
