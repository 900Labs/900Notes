<script lang="ts">
  import { onMount } from 'svelte'
  import { pageStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { SearchResult } from '../../lib/types'

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
    type: 'command'
  }

  type SearchItem = {
    id: string
    title: string
    snippet: string
    icon: string | null
    type: 'page'
  }

  type ListItem = CommandItem | SearchItem

  type SnippetPart = {
    text: string
    highlighted: boolean
  }

  let commands: CommandItem[] = $derived([
    { id: 'cmd-new', label: $t('command.newPage'), icon: '+', action: 'newPage', section: 'Create', keywords: 'blank note document', shortcut: 'Cmd+N', type: 'command' },
    { id: 'cmd-template', label: $t('command.newFromTemplate'), icon: 'T', action: 'newFromTemplate', section: 'Create', keywords: 'template meeting project journal', type: 'command' },
    { id: 'cmd-daily', label: $t('command.dailyNote'), icon: 'D', action: 'dailyNote', section: 'Create', keywords: 'today journal calendar', type: 'command' },
    { id: 'cmd-theme', label: $t('command.toggleTheme'), icon: 'A', action: 'toggleTheme', section: 'View', keywords: 'appearance light dark', type: 'command' },
    { id: 'cmd-backlinks', label: $t('command.toggleBacklinks'), icon: '<-', action: 'toggleBacklinks', section: 'View', keywords: 'links references mentions', type: 'command' },
    { id: 'cmd-outline', label: $t('command.toggleOutline'), icon: '#', action: 'toggleOutline', section: 'View', keywords: 'headings table contents', type: 'command' },
    { id: 'cmd-graph', label: $t('command.toggleGraph'), icon: 'G', action: 'toggleGraph', section: 'View', keywords: 'network map connections local graph', type: 'command' },
    { id: 'cmd-smart', label: $t('command.toggleSmartFolders'), icon: 'F', action: 'toggleSmartFolders', section: 'Organize', keywords: 'saved search dynamic folders filters database', type: 'command' },
    { id: 'cmd-favorites', label: $t('command.toggleFavorites'), icon: '*', action: 'toggleFavorites', section: 'Organize', keywords: 'star bookmarks pinned', type: 'command' },
    { id: 'cmd-history', label: $t('command.toggleHistory'), icon: 'H', action: 'toggleHistory', section: 'Review', keywords: 'versions revisions restore', type: 'command' },
    { id: 'cmd-related', label: $t('command.toggleRelated'), icon: 'R', action: 'toggleRelated', section: 'Review', keywords: 'similar discovery suggestions', type: 'command' },
    { id: 'cmd-export', label: $t('command.exportMarkdown'), icon: 'MD', action: 'exportMarkdown', section: 'Export', keywords: 'download markdown', type: 'command' },
    { id: 'cmd-pdf-page', label: $t('pdf.exportPage'), icon: 'PDF', action: 'exportPagePdf', section: 'Export', keywords: 'download pdf page', type: 'command' },
    { id: 'cmd-pdf-workspace', label: $t('pdf.exportWorkspace'), icon: 'PDF', action: 'exportWorkspacePdf', section: 'Export', keywords: 'download pdf workspace', type: 'command' },
    { id: 'cmd-data', label: 'Import and backup', icon: 'IO', action: 'openDataSettings', section: 'Tools', keywords: 'notion obsidian evernote roam import export backup', type: 'command' },
    { id: 'cmd-sync', label: $t('sync.title'), icon: 'S', action: 'openSyncSettings', section: 'Tools', keywords: 'local network peer devices', type: 'command' },
    { id: 'cmd-sharing', label: $t('sharing.title'), icon: 'SH', action: 'openSharingSettings', section: 'Tools', keywords: 'bundle html encrypted share publish', type: 'command' },
    { id: 'cmd-workspaces', label: $t('workspaces.title'), icon: 'W', action: 'openWorkspacesSettings', section: 'Tools', keywords: 'profiles spaces databases', type: 'command' },
    { id: 'cmd-security', label: $t('security.title'), icon: 'SEC', action: 'openSecuritySettings', section: 'Tools', keywords: 'encryption passphrase secure delete', type: 'command' },
    { id: 'cmd-plugins', label: 'Plugins', icon: 'P', action: 'openPluginsSettings', section: 'Tools', keywords: 'extensions custom blocks themes', type: 'command' },
    { id: 'cmd-settings', label: $t('command.openSettings'), icon: 'SET', action: 'openSettings', section: 'Tools', keywords: 'preferences configuration', type: 'command' },
  ])

  let filteredCommands = $derived(
    commands.filter((c) => {
      const q = query.toLowerCase()
      return c.label.toLowerCase().includes(q)
        || c.section.toLowerCase().includes(q)
        || c.keywords.toLowerCase().includes(q)
    }),
  )

  let commandSections = $derived(
    Array.from(new Set(filteredCommands.map((c) => c.section))).map((section) => ({
      section,
      items: filteredCommands.filter((c) => c.section === section),
    })),
  )

  let items = $derived<ListItem[]>([
    ...filteredCommands,
    ...results.map((r) => ({
      id: r.id,
      title: r.title,
      snippet: r.snippet,
      icon: r.icon,
      type: 'page' as const,
    })),
  ])

  onMount(() => {
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
      results = await pageStore.search(query)
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

  function selectItem(index: number) {
    const item = items[index]
    if (!item) return
    if (item.type === 'command') {
      onAction((item as CommandItem).action)
    } else {
      onPageSelect(item.id)
    }
    onClose()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 bg-black/30 flex items-start justify-center pt-[15vh]"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose() }}
  role="presentation"
>
  <div
    class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg overflow-hidden border border-gray-200 dark:border-gray-700"
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
        placeholder={$t('command.placeholder')}
        aria-label="{$t('command.placeholder')}"
        class="flex-1 bg-transparent text-base focus:outline-none placeholder:text-gray-400"
      />
      <kbd class="text-xs text-gray-400 px-1.5 py-0.5 border border-gray-300 dark:border-gray-600 rounded">ESC</kbd>
    </div>

    <div class="max-h-80 overflow-y-auto">
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
                  class="w-full text-left px-4 py-2 {i === selected ? 'bg-accent/10' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
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

        {#if results.length > 0}
          <div class="py-1 border-t border-gray-100 dark:border-gray-700/50">
            <div class="px-4 py-1 text-xs text-gray-400 font-medium">
              {$t('search.results', { count: results.length })}
            </div>
            {#each results as result, j (result.id)}
              {@const idx = filteredCommands.length + j}
              <button
                onclick={() => selectItem(idx)}
                onmouseenter={() => (selected = idx)}
                class="w-full text-left px-4 py-2 {idx === selected ? 'bg-accent/10' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
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
