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

  function renderContent(content: string): string {
    try {
      const doc = JSON.parse(content)
      return docToHtml(doc)
    } catch {
      return content
    }
  }

  function docToHtml(doc: any): string {
    if (!doc.content) return ''
    return doc.content.map((node: any) => nodeToHtml(node)).join('')
  }

  function nodeToHtml(node: any): string {
    switch (node.type) {
      case 'heading': {
        const level = node.attrs?.level || 1
        const text = (node.content || []).map((n: any) => n.text || '').join('')
        return `<h${level}>${text}</h${level}>`
      }
      case 'paragraph':
        return '<p>' + (node.content || []).map((n: any) => n.text || '').join('') + '</p>'
      case 'bullet_list':
        return '<ul>' + (node.content || []).map((n: any) => nodeToHtml(n)).join('') + '</ul>'
      case 'ordered_list':
        return '<ol>' + (node.content || []).map((n: any) => nodeToHtml(n)).join('') + '</ol>'
      case 'list_item':
        return '<li>' + (node.content || []).map((n: any) => nodeToHtml(n)).join('') + '</li>'
      case 'todo_item': {
        const checked = node.attrs?.checked ? 'checked' : ''
        return `<div class="todo-item ${checked}">${(node.content || []).map((n: any) => nodeToHtml(n)).join('')}</div>`
      }
      case 'code_block':
        return '<pre><code>' + (node.content || []).map((n: any) => n.text || '').join('') + '</code></pre>'
      case 'blockquote':
        return '<blockquote>' + (node.content || []).map((n: any) => nodeToHtml(n)).join('') + '</blockquote>'
      case 'image':
        return `<img src="${node.attrs?.src || ''}" alt="${node.attrs?.alt || ''}" />`
      case 'text':
        return node.text || ''
      default:
        return ''
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
      {@html renderContent(currentPageContent)}
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
    background: #1e1e2e;
    color: #cdd6f4;
    border-radius: 0.5rem;
    padding: 0.75rem 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
  }

  .reader-content :global(blockquote) {
    border-left: 3px solid #7c3aed;
    padding-left: 1rem;
    margin: 0.5rem 0;
    color: #6b7280;
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
