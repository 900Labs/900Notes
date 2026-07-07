<script lang="ts">
  import { onMount } from 'svelte'
  import { pageStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { SearchResult } from '../../lib/types'

  let {
    onClose,
    onPageSelect,
  }: {
    onClose: () => void
    onPageSelect: (id: string) => void
  } = $props()

  let query = $state('')
  let results = $state<SearchResult[]>([])
  let selected = $state(0)
  let searchTimer: ReturnType<typeof setTimeout> | null = null
  let inputElement: HTMLInputElement

  type SnippetPart = {
    text: string
    highlighted: boolean
  }

  onMount(() => {
    inputElement.focus()
  })

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
      selected = Math.min(selected + 1, results.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selected = Math.max(selected - 1, 0)
    } else if (e.key === 'Enter') {
      e.preventDefault()
      if (results[selected]) {
        onPageSelect(results[selected].id)
        onClose()
      }
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }

  function handleResultClick(id: string) {
    onPageSelect(id)
    onClose()
  }

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
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 bg-black/35 dark:bg-black/60 backdrop-blur-sm flex items-start justify-center pt-[12vh]"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose() }}
  role="presentation"
>
  <!-- Search Panel -->
  <div
    class="bg-white dark:bg-gray-900 rounded-lg shadow-[0_24px_80px_rgba(0,0,0,0.22)] dark:shadow-[0_24px_90px_rgba(0,0,0,0.55)] w-full max-w-lg overflow-hidden border border-gray-200 dark:border-gray-700"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Search Input -->
    <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <input
        bind:this={inputElement}
        bind:value={query}
        oninput={handleSearch}
        placeholder={$t('search.placeholder')}
        class="flex-1 bg-transparent text-base focus:outline-none placeholder:text-gray-400"
      />
      <kbd class="text-xs text-gray-400 px-1.5 py-0.5 border border-gray-300 dark:border-gray-600 rounded">ESC</kbd>
    </div>

    <!-- Results -->
    <div class="max-h-80 overflow-y-auto">
      {#if query && results.length === 0}
        <div class="px-4 py-8 text-center text-sm text-gray-400">
          {$t('search.noResults')}
        </div>
      {:else if results.length > 0}
        <div class="py-1">
          <div class="px-4 py-1 text-xs text-gray-400 font-medium">
            {$t('search.results', { count: results.length })}
          </div>
          {#each results as result, i (result.id)}
            <button
              onclick={() => handleResultClick(result.id)}
              onmouseenter={() => (selected = i)}
              class="w-full text-left px-4 py-2 {i === selected ? 'bg-accent/10 dark:bg-gray-800' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
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
      {:else}
        <div class="px-4 py-8 text-center text-sm text-gray-400">
          {$t('search.placeholder')}
        </div>
      {/if}
    </div>
  </div>
</div>
