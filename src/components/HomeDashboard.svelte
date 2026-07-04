<script lang="ts">
  import { onMount } from 'svelte'
  import {
    executeSavedSearch,
    getAllSavedSearches,
    getAllSmartFolders,
    getGraphData,
    getSmartFolderPages,
  } from '../lib/api'
  import { t } from '../i18n'
  import type {
    GraphNode,
    PageMetadata,
    PageTreeNode,
    PageTreeNodeMeta,
    SavedSearch,
    SearchResult,
    SmartFolder,
    SmartFolderRule,
  } from '../lib/types'

  type SmartViewPage = {
    id: string
    title: string
    icon: string | null
    updatedAt: string
    snippet?: string
  }

  type DashboardSmartView = {
    id: string
    kind: 'search' | 'folder'
    name: string
    icon: string
    detail: string
    count: number
    pinned: boolean
    pages: SmartViewPage[]
  }

  let {
    pageTree,
    recentPages,
    onAction,
    onPageSelect,
  }: {
    pageTree: PageTreeNodeMeta[]
    recentPages: PageMetadata[]
    onAction: (action: string) => void
    onPageSelect: (pageId: string) => void
  } = $props()

  function countPages(nodes: PageTreeNodeMeta[]): number {
    return nodes.reduce((count, node) => count + 1 + countPages(node.children), 0)
  }

  function countPinned(nodes: PageTreeNodeMeta[]): number {
    return nodes.reduce((count, node) => count + (node.page.pinned ? 1 : 0) + countPinned(node.children), 0)
  }

  let totalPages = $derived(countPages(pageTree))
  let pinnedPages = $derived(countPinned(pageTree))
  let graphNodes = $state<GraphNode[]>([])
  let smartViews = $state<DashboardSmartView[]>([])
  let smartViewsLoading = $state(false)
  let smartViewsError = $state(false)
  let savedSearchCount = $state(0)
  let smartFolderCount = $state(0)
  let today = $derived(new Date().toLocaleDateString(undefined, {
    weekday: 'long',
    month: 'short',
    day: 'numeric',
  }))

  let primaryActions = $derived([
    { label: $t('command.quickCapture'), detail: $t('dashboard.quickCaptureDetail'), action: 'openQuickCapture' },
    { label: $t('quickCapture.webMode'), detail: $t('dashboard.webCaptureDetail'), action: 'openWebCapture' },
    { label: $t('command.newPage'), detail: $t('dashboard.newPageDetail'), action: 'newPage' },
    { label: $t('daily.title'), detail: $t('dashboard.dailyNoteDetail'), action: 'dailyNote' },
    { label: $t('template.title'), detail: $t('dashboard.templateDetail'), action: 'newFromTemplate' },
  ])

  let workspaceActions = $derived([
    { label: $t('dashboard.knowledgeGraph'), action: 'toggleGraph' },
    { label: $t('smartFolder.title'), action: 'toggleSmartFolders' },
    { label: $t('favorites.title'), action: 'toggleFavorites' },
    { label: $t('sync.title'), action: 'openSyncSettings' },
    { label: $t('security.title'), action: 'openSecuritySettings' },
    { label: $t('dashboard.plugins'), action: 'openPluginsSettings' },
  ])

  let pageIndex = $derived.by(() => {
    const map = new Map<string, PageMetadata>()
    function walk(nodes: PageTreeNodeMeta[]) {
      for (const node of nodes) {
        map.set(node.page.id, node.page)
        walk(node.children)
      }
    }
    walk(pageTree)
    return map
  })

  let orphanTotal = $derived(graphNodes.filter((node) => node.linkCount === 0).length)
  let orphanNodes = $derived(graphNodes.filter((node) => node.linkCount === 0).slice(0, 6))
  let untaggedNodes = $derived(graphNodes.filter((node) => node.tagCount === 0).slice(0, 6))
  let hubNodes = $derived([...graphNodes].sort((a, b) => b.linkCount - a.linkCount).filter((node) => node.linkCount > 0).slice(0, 6))
  let smartViewTotal = $derived(smartViews.reduce((total, view) => total + view.count, 0))

  onMount(() => {
    loadDashboardData()
  })

  async function loadDashboardData() {
    await Promise.all([loadGraphHealth(), loadSmartViews()])
  }

  async function loadGraphHealth() {
    try {
      const data = await getGraphData()
      graphNodes = data.nodes
    } catch {
      graphNodes = []
    }
  }

  async function loadSmartViews() {
    smartViewsLoading = true
    smartViewsError = false
    try {
      const [searches, folders] = await Promise.all([getAllSavedSearches(), getAllSmartFolders()])
      savedSearchCount = searches.length
      smartFolderCount = folders.length

      const orderedSearches = [...searches]
        .sort((a, b) => Number(b.pinned) - Number(a.pinned) || a.name.localeCompare(b.name))
        .slice(0, 3)
      const orderedFolders = [...folders]
        .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name))
        .slice(0, 3)

      const previews = await Promise.all([
        ...orderedSearches.map(previewSavedSearch),
        ...orderedFolders.map(previewSmartFolder),
      ])
      smartViews = previews
        .filter((view): view is DashboardSmartView => view !== null)
        .sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.count - a.count)
        .slice(0, 4)
    } catch {
      smartViews = []
      smartViewsError = true
    } finally {
      smartViewsLoading = false
    }
  }

  async function previewSavedSearch(search: SavedSearch): Promise<DashboardSmartView | null> {
    try {
      const results = await executeSavedSearch(search.id)
      return {
        id: search.id,
        kind: 'search',
        name: search.name,
        icon: 'S',
        detail: search.query || $t('search.saved'),
        count: results.length,
        pinned: search.pinned,
        pages: results.slice(0, 4).map(searchResultToPage),
      }
    } catch {
      return null
    }
  }

  async function previewSmartFolder(folder: SmartFolder): Promise<DashboardSmartView | null> {
    try {
      const tree = await getSmartFolderPages(folder.id)
      const pages = flattenPageTree(tree)
      return {
        id: folder.id,
        kind: 'folder',
        name: folder.name,
        icon: folder.icon || 'F',
        detail: describeSmartFolderRules(folder),
        count: pages.length,
        pinned: false,
        pages: pages.slice(0, 4),
      }
    } catch {
      return null
    }
  }

  function searchResultToPage(result: SearchResult): SmartViewPage {
    return {
      id: result.id,
      title: result.title,
      icon: result.icon,
      updatedAt: result.updatedAt,
      snippet: stripSnippetMarkup(result.snippet),
    }
  }

  function flattenPageTree(nodes: PageTreeNode[]): SmartViewPage[] {
    const pages: SmartViewPage[] = []
    function walk(items: PageTreeNode[]) {
      for (const node of items) {
        pages.push({
          id: node.page.id,
          title: node.page.title || $t('editor.titlePlaceholder'),
          icon: node.page.icon,
          updatedAt: node.page.updatedAt,
        })
        walk(node.children)
      }
    }
    walk(nodes)
    return pages
  }

  function stripSnippetMarkup(value: string): string {
    return value
      .replaceAll('<mark>', '')
      .replaceAll('</mark>', '')
      .replaceAll('&lt;', '<')
      .replaceAll('&gt;', '>')
      .replaceAll('&quot;', '"')
      .replaceAll('&#39;', "'")
      .replaceAll('&amp;', '&')
  }

  function describeSmartFolderRules(folder: SmartFolder): string {
    try {
      const rules = JSON.parse(folder.rules) as SmartFolderRule[]
      const visibleRules = rules.filter((rule) => rule.value?.trim()).slice(0, 2)
      if (visibleRules.length === 0) return $t('dashboard.smartViewsRulesAll')
      const suffix = rules.length > visibleRules.length
        ? ` ${$t('dashboard.smartViewsRuleCount', { count: rules.length - visibleRules.length })}`
        : ''
      return `${visibleRules.map(formatSmartRule).join(' · ')}${suffix}`
    } catch {
      return $t('dashboard.smartViewsRulesAll')
    }
  }

  function formatSmartRule(rule: SmartFolderRule): string {
    const field = rule.field.replaceAll('_', ' ')
    const operator = rule.operator === 'contains'
      ? 'contains'
      : rule.operator === 'starts_with'
        ? 'starts with'
        : '='
    return `${field} ${operator} ${rule.value}`
  }
</script>

<div class="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
  <div class="max-w-5xl mx-auto px-8 py-8 space-y-8">
    <section class="flex flex-col gap-5 md:flex-row md:items-end md:justify-between">
      <div>
        <p class="text-sm text-gray-500 dark:text-gray-400">{today}</p>
        <h1 class="mt-1 text-2xl font-semibold tracking-normal">{$t('dashboard.workspace')}</h1>
      </div>
      <div class="grid grid-cols-4 gap-2 w-full md:w-auto">
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">{$t('dashboard.pages')}</div>
          <div class="text-xl font-semibold">{totalPages}</div>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">{$t('dashboard.pinned')}</div>
          <div class="text-xl font-semibold">{pinnedPages}</div>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">{$t('sidebar.recent')}</div>
          <div class="text-xl font-semibold">{recentPages.length}</div>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3">
          <div class="text-xs text-gray-500 dark:text-gray-400">{$t('dashboard.orphans')}</div>
          <div class="text-xl font-semibold">{orphanTotal}</div>
        </div>
      </div>
    </section>

    <section>
      <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        {#each primaryActions as action}
          <button
            onclick={() => onAction(action.action)}
            class="min-h-28 rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-4 text-left hover:border-accent/50 hover:bg-accent/5 transition-colors"
          >
            <div class="text-sm font-semibold">{action.label}</div>
            <div class="mt-2 text-sm text-gray-500 dark:text-gray-400 leading-snug">{action.detail}</div>
          </button>
        {/each}
      </div>
    </section>

    <section>
      <div class="mb-2 flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 class="text-sm font-semibold">{$t('dashboard.smartViews')}</h2>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            {$t('dashboard.smartViewsDetail', { searches: savedSearchCount, folders: smartFolderCount, pages: smartViewTotal })}
          </p>
        </div>
        <button
          onclick={() => onAction('toggleSmartFolders')}
          class="rounded-md border border-gray-200 dark:border-gray-800 px-2.5 py-1.5 text-xs text-accent hover:bg-accent/10"
        >
          {$t('dashboard.smartViewsManage')}
        </button>
      </div>
      <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        {#if smartViewsLoading}
          {#each Array(4) as _, idx}
            <div class="min-h-44 rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-4" aria-hidden={idx >= 0}>
              <div class="h-4 w-24 rounded bg-gray-100 dark:bg-gray-800"></div>
              <div class="mt-4 h-3 w-full rounded bg-gray-100 dark:bg-gray-800"></div>
              <div class="mt-2 h-3 w-4/5 rounded bg-gray-100 dark:bg-gray-800"></div>
            </div>
          {/each}
        {:else if smartViewsError}
          <div class="rounded-lg border border-red-200 dark:border-red-900/60 bg-white dark:bg-gray-900 px-4 py-6 text-sm text-red-500 dark:text-red-300 md:col-span-2 xl:col-span-4">
            {$t('dashboard.smartViewsError')}
          </div>
        {:else if smartViews.length === 0}
          <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-6 text-sm text-gray-500 dark:text-gray-400 md:col-span-2 xl:col-span-4">
            <div class="font-medium text-gray-800 dark:text-gray-200">{$t('dashboard.smartViewsEmpty')}</div>
            <button onclick={() => onAction('toggleSmartFolders')} class="mt-2 text-xs text-accent hover:underline">{$t('dashboard.smartViewsManage')}</button>
          </div>
        {:else}
          {#each smartViews as view (view.kind + view.id)}
            <div class="min-h-44 rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-4">
              <div class="mb-3 flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="grid h-7 w-7 place-items-center rounded-md bg-accent/10 text-xs font-semibold text-accent">{view.icon}</span>
                    <div class="min-w-0">
                      <div class="truncate text-sm font-semibold">{view.name}</div>
                      <div class="text-[11px] text-gray-400">
                        {view.kind === 'search' ? $t('dashboard.smartViewsSavedSearch') : $t('dashboard.smartViewsSmartFolder')}
                        {#if view.pinned} · {$t('dashboard.smartViewsPinned')}{/if}
                      </div>
                    </div>
                  </div>
                  <div class="mt-2 line-clamp-2 text-xs text-gray-500 dark:text-gray-400">{view.detail}</div>
                </div>
                <span class="shrink-0 rounded-full bg-gray-100 dark:bg-gray-800 px-2 py-0.5 text-[11px] text-gray-500">
                  {$t('dashboard.smartViewsResultCount', { count: view.count })}
                </span>
              </div>

              {#if view.pages.length === 0}
                <div class="rounded-md bg-gray-50 dark:bg-gray-950 px-3 py-4 text-sm text-gray-400">
                  {$t('dashboard.smartViewsNoResults')}
                </div>
              {:else}
                <div class="space-y-1">
                  {#each view.pages.slice(0, 3) as page (page.id)}
                    <button
                      onclick={() => onPageSelect(page.id)}
                      class="w-full rounded-md px-2 py-1.5 text-left hover:bg-gray-100 dark:hover:bg-gray-800"
                    >
                      <span class="block truncate text-sm">{page.icon || '#'} {page.title || $t('editor.titlePlaceholder')}</span>
                      <span class="block truncate text-[11px] text-gray-400">{page.snippet || new Date(page.updatedAt).toLocaleString()}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </section>

    <section class="grid gap-6 lg:grid-cols-[minmax(0,1.5fr)_minmax(280px,1fr)]">
      <div>
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-sm font-semibold">{$t('sidebar.recent')}</h2>
          <button
            onclick={() => onAction('openCommandPalette')}
            class="text-xs text-accent hover:underline"
          >{$t('dashboard.searchAll')}</button>
        </div>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 divide-y divide-gray-100 dark:divide-gray-800 overflow-hidden">
          {#if recentPages.length === 0}
            <div class="px-4 py-8 text-sm text-gray-500 dark:text-gray-400">No pages yet.</div>
          {:else}
            {#each recentPages as page (page.id)}
              <button
                onclick={() => onPageSelect(page.id)}
                class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                <span class="w-6 text-center">{page.icon || '#'}</span>
                <span class="min-w-0 flex-1">
                  <span class="block text-sm font-medium truncate">{page.title || 'Untitled'}</span>
                  <span class="block text-xs text-gray-500 dark:text-gray-400">{new Date(page.updatedAt).toLocaleString()}</span>
                </span>
              </button>
            {/each}
          {/if}
        </div>
      </div>

      <div>
        <h2 class="text-sm font-semibold mb-2">{$t('dashboard.workspaceTools')}</h2>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-2 grid grid-cols-2 gap-1">
          {#each workspaceActions as action}
            <button
              onclick={() => onAction(action.action)}
              class="px-3 py-2 rounded-md text-sm text-left hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              {action.label}
            </button>
          {/each}
        </div>

        <h2 class="text-sm font-semibold mt-6 mb-2">{$t('dashboard.reviewQueue')}</h2>
        <div class="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 divide-y divide-gray-100 dark:divide-gray-800 overflow-hidden">
          <div class="p-3">
            <div class="mb-2 flex items-center justify-between">
              <div class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('dashboard.orphanPages')}</div>
              <button onclick={() => onAction('toggleGraph')} class="text-xs text-accent hover:underline">{$t('graph.title')}</button>
            </div>
            {#if orphanNodes.length === 0}
              <p class="text-xs text-gray-400">{$t('dashboard.noUnlinkedPages')}</p>
            {:else}
              <div class="space-y-1">
                {#each orphanNodes as node (node.id)}
                  {@const page = pageIndex.get(node.id)}
                  <button onclick={() => onPageSelect(node.id)} class="w-full text-left text-sm truncate rounded px-2 py-1 hover:bg-gray-100 dark:hover:bg-gray-800">
                    {page?.icon || '#'} {node.title || 'Untitled'}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <div class="p-3">
            <div class="mb-2 text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('dashboard.needsTags')}</div>
            {#if untaggedNodes.length === 0}
              <p class="text-xs text-gray-400">{$t('dashboard.allIndexedTagged')}</p>
            {:else}
              <div class="space-y-1">
                {#each untaggedNodes as node (node.id)}
                  {@const page = pageIndex.get(node.id)}
                  <button onclick={() => onPageSelect(node.id)} class="w-full text-left text-sm truncate rounded px-2 py-1 hover:bg-gray-100 dark:hover:bg-gray-800">
                    {page?.icon || '#'} {node.title || 'Untitled'}
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <div class="p-3">
            <div class="mb-2 text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('dashboard.hubs')}</div>
            {#if hubNodes.length === 0}
              <p class="text-xs text-gray-400">{$t('dashboard.noLinkedHubs')}</p>
            {:else}
              <div class="space-y-1">
                {#each hubNodes as node (node.id)}
                  {@const page = pageIndex.get(node.id)}
                  <button onclick={() => onPageSelect(node.id)} class="w-full text-left text-sm truncate rounded px-2 py-1 hover:bg-gray-100 dark:hover:bg-gray-800">
                    {page?.icon || '#'} {node.title || 'Untitled'} <span class="text-xs text-gray-400">({node.linkCount})</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </section>
  </div>
</div>
