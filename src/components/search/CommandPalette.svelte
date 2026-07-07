<script lang="ts">
  import { onMount } from 'svelte'
  import { pageStore, tagStore, settingsStore, searchStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { PageMetadata, PageTreeNodeMeta, SearchResult } from '../../lib/types'

  let {
    onClose,
    onPageSelect,
    onAction,
  }: {
    onClose: () => void
    onPageSelect: (id: string) => void
    onAction: (action: string) => void
  } = $props()

  let query = $state('')
  let results = $state<SearchResult[]>([])
  let selected = $state(0)
  let searchTimer: ReturnType<typeof setTimeout> | null = null
  let inputElement: HTMLInputElement

  type CommandItem = {
    id: string
    label: string
    icon: string
    action: string
    section: string
    keywords: string
    shortcut?: string
    requiresPage?: boolean
    type: 'command'
  }

  type SearchItem = {
    id: string
    title: string
    snippet: string
    icon: string | null
    type: 'page'
  }

  type TagItem = {
    id: string
    name: string
    color: string | null
    type: 'tag'
  }

  type ListItem = CommandItem | SearchItem | TagItem

  type SnippetPart = {
    text: string
    highlighted: boolean
  }

  let commands: CommandItem[] = $derived([
    { id: 'cmd-new', label: $t('command.newPage'), icon: '+', action: 'newPage', section: 'Create', keywords: 'blank note document', shortcut: 'Cmd+N', type: 'command' },
    { id: 'cmd-capture', label: $t('command.quickCapture'), icon: 'QC', action: 'openQuickCapture', section: 'Create', keywords: 'clip inbox capture web note', shortcut: 'Cmd+Shift+C', type: 'command' },
    { id: 'cmd-web-capture', label: $t('quickCapture.webMode'), icon: 'URL', action: 'openWebCapture', section: 'Create', keywords: 'browser clipper source url link web page article', shortcut: 'Cmd+Shift+L', type: 'command' },
    { id: 'cmd-template', label: $t('command.newFromTemplate'), icon: 'T', action: 'newFromTemplate', section: 'Create', keywords: 'template meeting project journal', type: 'command' },
    { id: 'cmd-daily', label: $t('command.dailyNote'), icon: 'D', action: 'dailyNote', section: 'Create', keywords: 'today journal calendar', type: 'command' },
    { id: 'cmd-theme', label: $t('command.toggleTheme'), icon: 'A', action: 'toggleTheme', section: 'View', keywords: 'appearance light dark', type: 'command' },
    { id: 'cmd-backlinks', label: $t('command.toggleBacklinks'), icon: '<-', action: 'toggleBacklinks', section: 'View', keywords: 'links references mentions', requiresPage: true, type: 'command' },
    { id: 'cmd-outline', label: $t('command.toggleOutline'), icon: '#', action: 'toggleOutline', section: 'View', keywords: 'headings table contents', requiresPage: true, type: 'command' },
    { id: 'cmd-graph', label: $t('command.toggleGraph'), icon: 'G', action: 'toggleGraph', section: 'View', keywords: 'network map connections local graph', type: 'command' },
    { id: 'cmd-local-graph', label: $t('command.openLocalGraph'), icon: 'LG', action: 'openLocalGraph', section: 'View', keywords: 'current note neighborhood backlinks links', requiresPage: true, type: 'command' },
    { id: 'cmd-smart', label: $t('command.toggleSmartFolders'), icon: 'F', action: 'toggleSmartFolders', section: 'Organize', keywords: 'saved search dynamic folders filters database', type: 'command' },
    { id: 'cmd-favorites', label: $t('command.toggleFavorites'), icon: '*', action: 'toggleFavorites', section: 'Organize', keywords: 'star bookmarks pinned', type: 'command' },
    { id: 'cmd-history', label: $t('command.toggleHistory'), icon: 'H', action: 'toggleHistory', section: 'Review', keywords: 'versions revisions restore', requiresPage: true, type: 'command' },
    { id: 'cmd-related', label: $t('command.toggleRelated'), icon: 'R', action: 'toggleRelated', section: 'Review', keywords: 'similar discovery suggestions', requiresPage: true, type: 'command' },
    { id: 'cmd-export', label: $t('command.exportMarkdown'), icon: 'MD', action: 'exportMarkdown', section: 'Export', keywords: 'download markdown', requiresPage: true, type: 'command' },
    { id: 'cmd-pdf-page', label: $t('pdf.exportPage'), icon: 'PDF', action: 'exportPagePdf', section: 'Export', keywords: 'download pdf page', requiresPage: true, type: 'command' },
    { id: 'cmd-pdf-workspace', label: $t('pdf.exportWorkspace'), icon: 'PDF', action: 'exportWorkspacePdf', section: 'Export', keywords: 'download pdf workspace', type: 'command' },
    { id: 'cmd-data', label: 'Import and backup', icon: 'IO', action: 'openDataSettings', section: 'Tools', keywords: 'notion obsidian evernote roam import export backup', type: 'command' },
    { id: 'cmd-sync', label: $t('sync.title'), icon: 'S', action: 'openSyncSettings', section: 'Tools', keywords: 'local network peer devices', type: 'command' },
    { id: 'cmd-sharing', label: $t('sharing.title'), icon: 'SH', action: 'openSharingSettings', section: 'Tools', keywords: 'bundle html encrypted share publish', type: 'command' },
    { id: 'cmd-workspaces', label: $t('workspaces.title'), icon: 'W', action: 'openWorkspacesSettings', section: 'Tools', keywords: 'profiles spaces databases', type: 'command' },
    { id: 'cmd-security', label: $t('security.title'), icon: 'SEC', action: 'openSecuritySettings', section: 'Tools', keywords: 'encryption passphrase secure delete', type: 'command' },
    { id: 'cmd-plugins', label: 'Plugins', icon: 'P', action: 'openPluginsSettings', section: 'Tools', keywords: 'extensions custom blocks themes', type: 'command' },
    { id: 'cmd-settings', label: $t('command.openSettings'), icon: 'SET', action: 'openSettings', section: 'Tools', keywords: 'preferences configuration', type: 'command' },
  ])

  let flatPages = $derived.by<PageMetadata[]>(() => {
    const pages: PageMetadata[] = []
    function walk(nodes: PageTreeNodeMeta[]) {
      for (const node of nodes) {
        pages.push(node.page)
        walk(node.children)
      }
    }
    walk(pageStore.pageTree)
    return pages
  })

  let normalizedQuery = $derived(query.trim())
  let commandQuery = $derived(normalizedQuery.startsWith('>') ? normalizedQuery.slice(1).trim() : normalizedQuery)
  let tagQuery = $derived(normalizedQuery.startsWith('#') ? normalizedQuery.slice(1).trim().toLowerCase() : '')
  let titleQuery = $derived(normalizedQuery.startsWith('@') ? normalizedQuery.slice(1).trim().toLowerCase() : '')

  let filteredCommands = $derived(
    commands.filter((c) => {
      if (normalizedQuery.startsWith('#') || normalizedQuery.startsWith('@')) return false
      if (c.requiresPage && !pageStore.currentPage) return false
      const q = commandQuery.toLowerCase()
      return c.label.toLowerCase().includes(q)
        || c.section.toLowerCase().includes(q)
        || c.keywords.toLowerCase().includes(q)
    }),
  )

  let tagItems = $derived<TagItem[]>(
    normalizedQuery.startsWith('#')
      ? tagStore.tags
        .filter((tag) => tag.name.toLowerCase().includes(tagQuery))
        .map((tag) => ({ id: tag.id, name: tag.name, color: tag.color, type: 'tag' as const }))
      : [],
  )

  let titleItems = $derived<SearchItem[]>(
    normalizedQuery.startsWith('@')
      ? flatPages
        .filter((page) => (page.title || 'Untitled').toLowerCase().includes(titleQuery))
        .slice(0, 20)
        .map((page) => ({
          id: page.id,
          title: page.title || 'Untitled',
          snippet: `Updated ${new Date(page.updatedAt).toLocaleString()}`,
          icon: page.icon,
          type: 'page' as const,
        }))
      : [],
  )

  let saveSearchCommand = $derived<CommandItem | null>(
    normalizedQuery && !normalizedQuery.startsWith('>') && !normalizedQuery.startsWith('#') && !normalizedQuery.startsWith('@')
      ? {
        id: 'cmd-save-search',
        label: $t('command.saveSearch', { query: normalizedQuery.slice(0, 48) }),
        icon: 'SQ',
        action: `saveSearch:${normalizedQuery}`,
        section: 'Organize',
        keywords: 'query saved search smart view',
        type: 'command',
      }
      : null,
  )

  let saveSearchIndex = $derived(filteredCommands.length)
  let tagStartIndex = $derived(filteredCommands.length + (saveSearchCommand ? 1 : 0))
  let titleStartIndex = $derived(tagStartIndex + tagItems.length)
  let resultsStartIndex = $derived(titleStartIndex + titleItems.length)

  let commandSections = $derived(
    Array.from(new Set(filteredCommands.map((c) => c.section))).map((section) => ({
      section,
      items: filteredCommands.filter((c) => c.section === section),
    })),
  )

  let items = $derived<ListItem[]>([
    ...filteredCommands,
    ...(saveSearchCommand ? [saveSearchCommand] : []),
    ...tagItems,
    ...titleItems,
    ...results.map((r) => ({
      id: r.id,
      title: r.title,
      snippet: r.snippet,
      icon: r.icon,
      type: 'page' as const,
    })),
  ])

  onMount(() => {
    tagStore.loadTags()
    searchStore.loadSavedSearches()
    inputElement.focus()
  })

  function decodeSnippetText(value: string): string {
    return value
      .replaceAll('&lt;', '<')
      .replaceAll('&gt;', '>')
      .replaceAll('&quot;', '"')
      .replaceAll('&#39;', "'")
      .replaceAll('&amp;', '&')
  }

  function snippetParts(snippet: string): SnippetPart[] {
    const parts: SnippetPart[] = []
    let cursor = 0
    while (cursor < snippet.length) {
      const start = snippet.indexOf('<mark>', cursor)
      if (start === -1) {
        parts.push({ text: decodeSnippetText(snippet.slice(cursor)), highlighted: false })
        break
      }
      if (start > cursor) {
        parts.push({ text: decodeSnippetText(snippet.slice(cursor, start)), highlighted: false })
      }
      const end = snippet.indexOf('</mark>', start + 6)
      if (end === -1) {
        parts.push({ text: decodeSnippetText(snippet.slice(start)), highlighted: false })
        break
      }
      parts.push({
        text: decodeSnippetText(snippet.slice(start + 6, end)),
        highlighted: true,
      })
      cursor = end + 7
    }
    return parts
  }

  function handleSearch() {
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(async () => {
      if (!normalizedQuery || normalizedQuery.startsWith('>') || normalizedQuery.startsWith('#') || normalizedQuery.startsWith('@')) {
        results = []
      } else {
        results = await pageStore.search(normalizedQuery)
      }
      selected = 0
    }, 200)
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selected = Math.min(selected + 1, items.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selected = Math.max(selected - 1, 0)
    } else if (e.key === 'Enter') {
      e.preventDefault()
      selectItem(selected)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }

  async function selectItem(index: number) {
    const item = items[index]
    if (!item) return
    if (item.type === 'command') {
      const action = (item as CommandItem).action
      if (action.startsWith('saveSearch:')) {
        const savedQuery = action.slice('saveSearch:'.length)
        await searchStore.createSavedSearch(savedQuery.slice(0, 48), savedQuery, null, true)
        settingsStore.sidebarTab = 'smart'
      } else {
        onAction(action)
      }
    } else if (item.type === 'tag') {
      settingsStore.activeTagFilter = item.id
      settingsStore.sidebarTab = 'tags'
    } else {
      onPageSelect(item.id)
    }
    onClose()
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
    class="bg-white dark:bg-gray-900 rounded-lg shadow-[0_24px_80px_rgba(0,0,0,0.22)] dark:shadow-[0_24px_90px_rgba(0,0,0,0.55)] w-full max-w-xl overflow-hidden border border-gray-200 dark:border-gray-700"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="{$t('command.placeholder')}"
    tabindex="-1"
  >
    <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <input
        bind:this={inputElement}
        bind:value={query}
        oninput={handleSearch}
        placeholder={$t('command.palettePlaceholder')}
        aria-label="{$t('command.placeholder')}"
        class="flex-1 bg-transparent text-base focus:outline-none placeholder:text-gray-400"
      />
      <kbd class="text-xs text-gray-400 px-1.5 py-0.5 border border-gray-300 dark:border-gray-600 rounded">ESC</kbd>
    </div>

    <div class="max-h-80 overflow-y-auto">
      <div class="px-4 py-2 border-b border-gray-100 dark:border-gray-700/50 flex flex-wrap gap-1.5 text-[11px] text-gray-500 dark:text-gray-400">
        <span class="rounded border border-gray-200 dark:border-gray-700 px-1.5 py-0.5">{$t('command.prefixCommands')}</span>
        <span class="rounded border border-gray-200 dark:border-gray-700 px-1.5 py-0.5">{$t('command.prefixTags')}</span>
        <span class="rounded border border-gray-200 dark:border-gray-700 px-1.5 py-0.5">{$t('command.prefixPages')}</span>
        <span class="rounded border border-gray-200 dark:border-gray-700 px-1.5 py-0.5">{$t('command.enterOpens')}</span>
      </div>

      {#if items.length === 0 && query}
        <div class="px-4 py-8 text-center text-sm text-gray-400">
          {$t('search.noResults')}
        </div>
      {:else}
        {#if filteredCommands.length > 0}
          <div class="py-1">
            {#each commandSections as group}
              <div class="px-4 py-1 text-xs text-gray-400 font-medium">{group.section}</div>
              {#each group.items as cmd (cmd.id)}
                {@const i = filteredCommands.findIndex((c) => c.id === cmd.id)}
                <button
                  onclick={() => selectItem(i)}
                  onmouseenter={() => (selected = i)}
                  class="w-full text-left px-4 py-2 {i === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
                >
                  <div class="flex items-center gap-3">
                    <span class="w-8 text-center text-gray-400 font-mono text-xs">{cmd.icon}</span>
                    <span class="min-w-0 flex-1 text-sm font-medium truncate">{cmd.label}</span>
                    {#if cmd.shortcut}
                      <kbd class="text-[10px] text-gray-400 border border-gray-300 dark:border-gray-600 rounded px-1.5 py-0.5">{cmd.shortcut}</kbd>
                    {/if}
                  </div>
                </button>
              {/each}
            {/each}
          </div>
        {/if}

        {#if saveSearchCommand}
          <div class="py-1 border-t border-gray-100 dark:border-gray-700/50">
            <button
              onclick={() => selectItem(saveSearchIndex)}
              onmouseenter={() => (selected = saveSearchIndex)}
              class="w-full text-left px-4 py-2 {saveSearchIndex === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
            >
              <div class="flex items-center gap-3">
                <span class="w-8 text-center text-gray-400 font-mono text-xs">SQ</span>
                <span class="min-w-0 flex-1 text-sm font-medium truncate">{saveSearchCommand.label}</span>
              </div>
            </button>
          </div>
        {/if}

        {#if tagItems.length > 0}
          <div class="py-1 border-t border-gray-100 dark:border-gray-700/50">
            <div class="px-4 py-1 text-xs text-gray-400 font-medium">{$t('sidebar.tags')}</div>
            {#each tagItems as tag, j (tag.id)}
              {@const idx = tagStartIndex + j}
              <button
                onclick={() => selectItem(idx)}
                onmouseenter={() => (selected = idx)}
                class="w-full text-left px-4 py-2 {idx === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
              >
                <div class="flex items-center gap-3">
                  <span class="h-2.5 w-2.5 rounded-full" style="background-color: {tag.color || '#2f6fce'}"></span>
                  <span class="text-sm font-medium">#{tag.name}</span>
                </div>
              </button>
            {/each}
          </div>
        {/if}

        {#if titleItems.length > 0}
          <div class="py-1 border-t border-gray-100 dark:border-gray-700/50">
            <div class="px-4 py-1 text-xs text-gray-400 font-medium">{$t('command.titleMatches')}</div>
            {#each titleItems as page, j (page.id)}
              {@const idx = titleStartIndex + j}
              <button
                onclick={() => selectItem(idx)}
                onmouseenter={() => (selected = idx)}
                class="w-full text-left px-4 py-2 {idx === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
              >
                <div class="flex items-center gap-2">
                  {#if page.icon}<span>{page.icon}</span>{/if}
                  <span class="font-medium text-sm">{page.title}</span>
                </div>
                <p class="text-xs text-gray-500 mt-0.5 line-clamp-1">{page.snippet}</p>
              </button>
            {/each}
          </div>
        {/if}

        {#if results.length > 0}
          <div class="py-1 border-t border-gray-100 dark:border-gray-700/50">
            <div class="px-4 py-1 text-xs text-gray-400 font-medium">
              {$t('search.results', { count: results.length })}
            </div>
            {#each results as result, j (result.id)}
              {@const idx = resultsStartIndex + j}
              <button
                onclick={() => selectItem(idx)}
                onmouseenter={() => (selected = idx)}
                class="w-full text-left px-4 py-2 {idx === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
              >
                <div class="flex items-center gap-2">
                  {#if result.icon}<span>{result.icon}</span>{/if}
                  <span class="font-medium text-sm">{result.title}</span>
                </div>
                <p class="text-xs text-gray-500 mt-0.5 line-clamp-2">
                  {#each snippetParts(result.snippet) as part}
                    {#if part.highlighted}
                      <mark class="bg-yellow-200 dark:bg-yellow-700/60 rounded px-0.5">{part.text}</mark>
                    {:else}
                      {part.text}
                    {/if}
                  {/each}
                </p>
              </button>
            {/each}
          </div>
        {:else if !query && filteredCommands.length > 0}
          <div class="px-4 py-8 text-center text-sm text-gray-400">
            {$t('search.placeholder')}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>
