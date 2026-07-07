<script lang="ts">
  import { onMount } from 'svelte'
  import { pageStore, settingsStore } from '../stores/app.svelte'
  import { t, setLocale } from '../i18n'
  import * as api from '../lib/api'
  import type { PageMetadata } from '../lib/types'

  let currentPageId = $state<string | null>(null)
  let currentPageContent = $state<string>('')
  let currentPageTitle = $state<string>('')
  let currentPageIcon = $state<string | null>(null)
  let searchQuery = $state('')
  let searchResults = $state<PageMetadata[]>([])
  let showSearch = $state(false)
  let view = $state<'list' | 'reader'>('list')

  type ReaderBlock = {
    type: 'heading' | 'paragraph' | 'bullet' | 'ordered' | 'todo' | 'code' | 'quote' | 'image'
    level?: number
    text: string
    checked?: boolean
  }

  onMount(async () => {
    await settingsStore.loadSettings()
    await pageStore.loadPageTree()
    await pageStore.loadRecentPages()
    setLocale(settingsStore.language as any)
  })

  async function openPage(id: string) {
    const page = await api.getPage(id)
    currentPageId = id
    currentPageContent = page.content
    currentPageTitle = page.title
    currentPageIcon = page.icon
    view = 'reader'
  }

  function handleSearch() {
    if (!searchQuery.trim()) {
      searchResults = []
      return
    }
    const flat = flattenTree(pageStore.pageTree)
    searchResults = flat.filter((p) =>
      p.title.toLowerCase().includes(searchQuery.toLowerCase()),
    )
  }

  function flattenTree(
    tree: typeof pageStore.pageTree,
  ): PageMetadata[] {
    const result: PageMetadata[] = []
    for (const node of tree) {
      result.push(node.page)
      if (node.children.length > 0) {
        result.push(...flattenTree(node.children))
      }
    }
    return result
  }

  function renderContent(content: string): ReaderBlock[] {
    try {
      const doc = JSON.parse(content)
      return docToBlocks(doc)
    } catch {
      return [{ type: 'paragraph', text: content }]
    }
  }

  function docToBlocks(doc: any): ReaderBlock[] {
    if (!doc.content) return []
    return doc.content.flatMap((node: any, index: number) => nodeToBlocks(node, index))
  }

  function inlineText(node: any): string {
    return (node.content || [])
      .map((child: any) => {
        if (child.type === 'text') return child.text || ''
        if (child.type === 'hard_break') return '\n'
        if (child.type === 'wiki_link') return child.attrs?.title || ''
        return inlineText(child)
      })
      .join('')
  }

  function nodeToBlocks(node: any, index = 0): ReaderBlock[] {
    switch (node.type) {
      case 'heading': {
        const level = node.attrs?.level || 1
        return [{ type: 'heading', level, text: inlineText(node) }]
      }
      case 'paragraph':
        return [{ type: 'paragraph', text: inlineText(node) }]
      case 'bullet_list':
        return (node.content || []).map((n: any) => ({ type: 'bullet', text: inlineText(n) }))
      case 'ordered_list':
        return (node.content || []).map((n: any, i: number) => ({
          type: 'ordered',
          text: `${i + 1}. ${inlineText(n)}`,
        }))
      case 'list_item':
        return [{ type: 'bullet', text: inlineText(node) }]
      case 'todo_item':
        return [{ type: 'todo', text: inlineText(node), checked: !!node.attrs?.checked }]
      case 'code_block':
        return [{ type: 'code', text: inlineText(node) }]
      case 'blockquote':
        return [{ type: 'quote', text: inlineText(node) }]
      case 'image':
        return [{ type: 'image', text: node.attrs?.alt || 'Image' }]
      case 'text':
        return [{ type: 'paragraph', text: node.text || '' }]
      default:
        return [{ type: 'paragraph', text: inlineText(node) || String(index ? '' : '') }]
    }
  }
</script>

<div class="mobile-app">
  {#if view === 'list'}
    <header class="header">
      <h1>{$t('app.title')}</h1>
      <button onclick={() => (showSearch = !showSearch)} aria-label="Search">
        <svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </button>
    </header>

    {#if showSearch}
      <div class="search-bar">
        <input
          type="text"
          bind:value={searchQuery}
          oninput={handleSearch}
          placeholder={$t('sidebar.search')}
          aria-label={$t('sidebar.search')}
        />
      </div>
    {/if}

    <div class="content">
      {#if searchQuery && searchResults.length > 0}
        {#each searchResults as page (page.id)}
          <button class="page-item" onclick={() => openPage(page.id)}>
            {#if page.icon}<span class="icon">{page.icon}</span>{/if}
            <span class="title">{page.title || $t('editor.titlePlaceholder')}</span>
          </button>
        {/each}
      {:else if searchQuery}
        <p class="empty">{$t('search.noResults')}</p>
      {:else}
        <h2 class="section-title">{$t('sidebar.recent')}</h2>
        {#each pageStore.recentPages as page (page.id)}
          <button class="page-item" onclick={() => openPage(page.id)}>
            {#if page.icon}<span class="icon">{page.icon}</span>{/if}
            <span class="title">{page.title || $t('editor.titlePlaceholder')}</span>
          </button>
        {/each}

        <h2 class="section-title">{$t('sidebar.pages')}</h2>
        {#each pageStore.recentPages as page (page.id)}
          <button class="page-item" onclick={() => openPage(page.id)}>
            {#if page.icon}<span class="icon">{page.icon}</span>{/if}
            <span class="title">{page.title || $t('editor.titlePlaceholder')}</span>
          </button>
        {/each}
      {/if}
    </div>
  {:else if view === 'reader' && currentPageId}
    <header class="header">
      <button onclick={() => (view = 'list')} aria-label="Back">
        <svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="reader-title">
        {#if currentPageIcon}{currentPageIcon} {/if}
        {currentPageTitle || $t('editor.titlePlaceholder')}
      </h1>
    </header>

    <div class="reader-content">
      {#each renderContent(currentPageContent) as block}
        {#if block.type === 'heading' && block.level === 1}
          <h1>{block.text}</h1>
        {:else if block.type === 'heading' && block.level === 2}
          <h2>{block.text}</h2>
        {:else if block.type === 'heading'}
          <h3>{block.text}</h3>
        {:else if block.type === 'bullet'}
          <p class="list-line">• {block.text}</p>
        {:else if block.type === 'ordered'}
          <p class="list-line">{block.text}</p>
        {:else if block.type === 'todo'}
          <p class="todo-item" class:checked={block.checked}>[{block.checked ? 'x' : ' '}] {block.text}</p>
        {:else if block.type === 'code'}
          <pre><code>{block.text}</code></pre>
        {:else if block.type === 'quote'}
          <blockquote>{block.text}</blockquote>
        {:else if block.type === 'image'}
          <p class="image-placeholder">[Image: {block.text}]</p>
        {:else}
          <p>{block.text}</p>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .mobile-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #fff;
    color: #1a1a1a;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e5e7eb;
    background: #f9fafb;
  }

  .header h1 {
    font-size: 1.125rem;
    font-weight: 700;
    flex: 1;
    margin: 0;
  }

  .reader-title {
    font-size: 1rem;
    font-weight: 600;
    flex: 1;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header button {
    background: none;
    border: none;
    padding: 0.25rem;
    cursor: pointer;
    color: #6b7280;
  }

  .search-bar {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid #e5e7eb;
  }

  .search-bar input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.5rem;
    font-size: 16px;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: #9ca3af;
    padding: 0.75rem 0.5rem 0.25rem;
    margin: 0;
  }

  .page-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    text-align: left;
    padding: 0.75rem;
    border: none;
    background: none;
    border-radius: 0.5rem;
    cursor: pointer;
    font-size: 0.9375rem;
  }

  .page-item:active {
    background: #f3f4f6;
  }

  .page-item .icon {
    font-size: 1.125rem;
  }

  .page-item .title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    text-align: center;
    color: #9ca3af;
    padding: 2rem;
    font-size: 0.875rem;
  }

  .reader-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.25rem 2rem;
    font-size: 16px;
    line-height: 1.6;
  }

  .reader-content :global(h1) {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 1rem 0 0.5rem;
  }

  .reader-content :global(h2) {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0.875rem 0 0.5rem;
  }

  .reader-content :global(h3) {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0.75rem 0 0.5rem;
  }

  .reader-content :global(p) {
    margin: 0.5rem 0;
  }

  .reader-content :global(ul),
  .reader-content :global(ol) {
    padding-left: 1.5rem;
    margin: 0.5rem 0;
  }

  .reader-content :global(pre) {
    background: #0d0d0d;
    color: #f4f4f6;
    border: 1px solid #242728;
    border-radius: 0.5rem;
    padding: 0.75rem 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
  }

  .reader-content :global(blockquote) {
    border-left: 3px solid #2f6fce;
    padding-left: 1rem;
    margin: 0.5rem 0;
    color: #6b655d;
  }

  .reader-content :global(.todo-item) {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    margin: 0.25rem 0;
  }

  .reader-content :global(.todo-item)::before {
    content: '☐';
    font-size: 1.25rem;
  }

  .reader-content :global(.todo-item.checked)::before {
    content: '☑';
  }

  .reader-content :global(.todo-item.checked) {
    text-decoration: line-through;
    color: #9ca3af;
  }

  .reader-content :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
  }
</style>
