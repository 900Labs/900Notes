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
    { id: 'cmd-new', label: $t('command.newPage'), icon: '+', action: 'newPage', type: 'command' },
    { id: 'cmd-template', label: $t('command.newFromTemplate'), icon: '📋', action: 'newFromTemplate', type: 'command' },
    { id: 'cmd-daily', label: $t('command.dailyNote'), icon: '📅', action: 'dailyNote', type: 'command' },
    { id: 'cmd-theme', label: $t('command.toggleTheme'), icon: '◐', action: 'toggleTheme', type: 'command' },
    { id: 'cmd-settings', label: $t('command.openSettings'), icon: '⚙', action: 'openSettings', type: 'command' },
    { id: 'cmd-backlinks', label: $t('command.toggleBacklinks'), icon: '←', action: 'toggleBacklinks', type: 'command' },
    { id: 'cmd-outline', label: $t('command.toggleOutline'), icon: '≡', action: 'toggleOutline', type: 'command' },
    { id: 'cmd-graph', label: $t('command.toggleGraph'), icon: '◉', action: 'toggleGraph', type: 'command' },
    { id: 'cmd-smart', label: $t('command.toggleSmartFolders'), icon: '📁', action: 'toggleSmartFolders', type: 'command' },
    { id: 'cmd-favorites', label: $t('command.toggleFavorites'), icon: '⭐', action: 'toggleFavorites', type: 'command' },
    { id: 'cmd-history', label: $t('command.toggleHistory'), icon: '🕐', action: 'toggleHistory', type: 'command' },
    { id: 'cmd-related', label: $t('command.toggleRelated'), icon: '🔗', action: 'toggleRelated', type: 'command' },
    { id: 'cmd-export', label: $t('command.exportMarkdown'), icon: '↓', action: 'exportMarkdown', type: 'command' },
    { id: 'cmd-pdf-page', label: $t('pdf.exportPage'), icon: '📄', action: 'exportPagePdf', type: 'command' },
    { id: 'cmd-pdf-workspace', label: $t('pdf.exportWorkspace'), icon: '📚', action: 'exportWorkspacePdf', type: 'command' },
  ])

  let filteredCommands = $derived(
    commands.filter((c) => c.label.toLowerCase().includes(query.toLowerCase())),
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
            <div class="px-4 py-1 text-xs text-gray-400 font-medium">{$t('command.actions')}</div>
            {#each filteredCommands as cmd, i (cmd.id)}
              <button
                onclick={() => selectItem(i)}
                onmouseenter={() => (selected = i)}
                class="w-full text-left px-4 py-2 {i === selected ? 'bg-accent/10' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
              >
                <div class="flex items-center gap-3">
                  <span class="w-5 text-center text-gray-400 font-mono text-sm">{cmd.icon}</span>
                  <span class="text-sm font-medium">{cmd.label}</span>
                </div>
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
