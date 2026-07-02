<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import Sidebar from './components/sidebar/Sidebar.svelte'
  import AppMenuBar from './components/AppMenuBar.svelte'
  import HomeDashboard from './components/HomeDashboard.svelte'
  import EditorView from './components/editor/EditorView.svelte'
  import CommandPalette from './components/search/CommandPalette.svelte'
  import TemplatePicker from './components/search/TemplatePicker.svelte'
  import SettingsModal from './components/settings/SettingsModal.svelte'
  import BacklinksPanel from './components/backlinks/BacklinksPanel.svelte'
  import OutlinePanel from './components/editor/OutlinePanel.svelte'
  import GraphView from './components/graph/GraphView.svelte'
  import HistoryPanel from './components/history/HistoryPanel.svelte'
  import RelatedPagesPanel from './components/discovery/RelatedPagesPanel.svelte'
  import { pageStore, tagStore, backlinkStore, propertyStore, templateStore, settingsStore, historyStore, discoveryStore, encryptionStore, attachmentStore } from './stores/app.svelte'
  import { t, locale, setLocale } from './i18n'
  import * as api from './lib/api'

  let showCommandPalette = $state(false)
  let showTemplatePicker = $state(false)
  let showSettings = $state(false)
  let showBacklinks = $state(true)
  let showOutline = $state(false)
  let showGraph = $state(false)
  let showHistory = $state(false)
  let showRelated = $state(false)
  let settingsSection = $state<'appearance' | 'language' | 'data' | 'sync' | 'sharing' | 'workspaces' | 'security' | 'plugins' | 'about'>('appearance')
  let unlockPassphrase = $state('')
  let unlockError = $state<string | null>(null)
  let isUnlocking = $state(false)

  let savedCallback: ((e: KeyboardEvent) => void) | null = null

  async function loadAppData() {
    await settingsStore.loadSettings()
    await pageStore.loadPageTree()
    await pageStore.loadRecentPages()
    await tagStore.loadTags()
    setLocale(settingsStore.language as any)
  }

  function installShortcuts() {
    if (savedCallback) return
    savedCallback = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        showCommandPalette = !showCommandPalette
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'n') {
        e.preventDefault()
        handleCommandAction('newPage')
      }
      if (e.key === 'Escape') {
        showCommandPalette = false
        showSettings = false
      }
    }
    window.addEventListener('keydown', savedCallback)
  }

  onMount(async () => {
    await encryptionStore.checkStatus()
    if (encryptionStore.enabled && !encryptionStore.unlocked) {
      return
    }
    await loadAppData()
    installShortcuts()
  })

  async function handleUnlock() {
    unlockError = null
    isUnlocking = true
    try {
      const result = await encryptionStore.unlock(unlockPassphrase)
      if (result) {
        unlockPassphrase = ''
        await loadAppData()
        installShortcuts()
      } else {
        unlockError = 'Invalid passphrase'
      }
    } catch (e) {
      unlockError = String(e)
    } finally {
      isUnlocking = false
    }
  }

  onDestroy(() => {
    if (savedCallback) {
      window.removeEventListener('keydown', savedCallback)
    }
  })

  async function handlePageSelect(pageId: string) {
    await pageStore.loadPage(pageId)
    await tagStore.loadPageTags(pageId)
    await backlinkStore.loadBacklinks(pageId)
    await propertyStore.loadProperties(pageId)
    await attachmentStore.loadAttachments(pageId)
  }

  async function handlePageCreated(pageId: string) {
    await handlePageSelect(pageId)
  }

  function openSettings(section: typeof settingsSection = 'appearance') {
    settingsSection = section
    showSettings = true
  }

  async function handleCommandAction(action: string) {
    switch (action) {
      case 'openCommandPalette':
        showCommandPalette = true
        break
      case 'newPage': {
        const page = await pageStore.createPage('Untitled', null)
        await handlePageCreated(page.id)
        break
      }
      case 'newFromTemplate':
        showTemplatePicker = true
        break
      case 'dailyNote': {
        const today = new Date().toISOString().slice(0, 10)
        const page = await templateStore.getOrCreateDailyNote(today)
        await handlePageCreated(page.id)
        await pageStore.loadPageTree()
        break
      }
      case 'toggleTheme': {
        const next = settingsStore.theme === 'dark' ? 'light' : 'dark'
        await settingsStore.setTheme(next)
        break
      }
      case 'openSettings':
        openSettings('appearance')
        break
      case 'openDataSettings':
        openSettings('data')
        break
      case 'openSyncSettings':
        openSettings('sync')
        break
      case 'openSharingSettings':
        openSettings('sharing')
        break
      case 'openWorkspacesSettings':
        openSettings('workspaces')
        break
      case 'openSecuritySettings':
        openSettings('security')
        break
      case 'openPluginsSettings':
        openSettings('plugins')
        break
      case 'openRecent':
        settingsStore.sidebarTab = 'recent'
        pageStore.loadRecentPages()
        break
      case 'toggleBacklinks':
        showBacklinks = !showBacklinks
        break
      case 'toggleOutline':
        showOutline = !showOutline
        break
      case 'toggleGraph':
        showGraph = !showGraph
        break
      case 'toggleSmartFolders':
        settingsStore.sidebarTab = 'smart'
        break
      case 'toggleFavorites':
        settingsStore.sidebarTab = 'favorites'
        historyStore.loadFavorites()
        break
      case 'toggleHistory':
        showHistory = !showHistory
        if (showHistory && pageStore.currentPage) {
          historyStore.loadRevisions(pageStore.currentPage.id)
        }
        break
      case 'toggleRelated':
        showRelated = !showRelated
        if (showRelated && pageStore.currentPage) {
          discoveryStore.loadRelatedPages(pageStore.currentPage.id, 10)
        }
        break
      case 'exportMarkdown': {
        if (pageStore.currentPage) {
          const md = await api.exportPageMarkdown(pageStore.currentPage.id)
          const blob = new Blob([md], { type: 'text/markdown' })
          const url = URL.createObjectURL(blob)
          const a = document.createElement('a')
          a.href = url
          a.download = `${pageStore.currentPage.title || 'untitled'}.md`
          a.click()
          URL.revokeObjectURL(url)
        }
        break
      }
      case 'exportPagePdf': {
        if (pageStore.currentPage) {
          const bytes = await api.exportPagePdf(pageStore.currentPage.id)
          const blob = new Blob([new Uint8Array(bytes)], { type: 'application/pdf' })
          const url = URL.createObjectURL(blob)
          const a = document.createElement('a')
          a.href = url
          a.download = `${pageStore.currentPage.title || 'untitled'}.pdf`
          a.click()
          URL.revokeObjectURL(url)
        }
        break
      }
      case 'exportWorkspacePdf': {
        const bytes = await api.exportWorkspacePdf()
        const blob = new Blob([new Uint8Array(bytes)], { type: 'application/pdf' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = '900notes-workspace.pdf'
        a.click()
        URL.revokeObjectURL(url)
        break
      }
    }
  }

  async function handleTemplateSelect(template: import('./lib/types').Template) {
    const page = await templateStore.createPageFromTemplate(template.id, 'Untitled', null)
    await handlePageCreated(page.id)
    await pageStore.loadPageTree()
    showTemplatePicker = false
  }
</script>

{#if encryptionStore.enabled && !encryptionStore.unlocked}
  <div class="flex h-screen w-screen items-center justify-center bg-gray-50 dark:bg-gray-950">
    <div class="w-full max-w-sm space-y-6 rounded-xl bg-white dark:bg-gray-900 p-8 shadow-xl border border-gray-200 dark:border-gray-800">
      <div class="text-center">
        <svg class="w-12 h-12 mx-auto mb-3 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
        <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-100">{$t('security.unlockDatabase')}</h1>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{$t('security.unlockDescription')}</p>
      </div>
      {#if unlockError}
        <div class="rounded-lg bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-600 dark:text-red-400">
          {unlockError}
        </div>
      {/if}
      <div class="space-y-4">
        <input
          type="password"
          bind:value={unlockPassphrase}
          onkeydown={(e) => { if (e.key === 'Enter') handleUnlock() }}
          placeholder={$t('security.passphrase')}
          class="w-full rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-4 py-2.5 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:border-accent focus:ring-1 focus:ring-accent outline-none transition-colors"
          disabled={isUnlocking}
        />
        <button
          type="button"
          onclick={handleUnlock}
          disabled={isUnlocking || !unlockPassphrase}
          class="w-full rounded-lg bg-accent hover:bg-accent-dark disabled:opacity-50 disabled:cursor-not-allowed px-4 py-2.5 text-white font-medium transition-colors"
        >
          {isUnlocking ? '...' : $t('security.unlock')}
        </button>
      </div>
    </div>
  </div>
{:else}
<div class="flex h-screen w-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
  <a href="#main-content" class="skip-link">{$t('a11y.skipToContent')}</a>
  <Sidebar
    onPageSelect={handlePageSelect}
    onPageCreated={handlePageCreated}
    onOpenSettings={() => openSettings('appearance')}
  />

  <main id="main-content" class="flex-1 flex flex-col overflow-hidden min-w-0" aria-label="{$t('a11y.mainContent')}">
    <AppMenuBar currentPage={pageStore.currentPage} onAction={handleCommandAction} />

    {#if showGraph}
      <GraphView onNavigate={(id) => { showGraph = false; handlePageSelect(id) }} />
    {:else if pageStore.currentPage}
      <EditorView
        page={pageStore.currentPage}
        onNavigate={handlePageSelect}
        onAppAction={handleCommandAction}
        {showOutline}
      />
    {:else}
      <HomeDashboard
        pageTree={pageStore.pageTree}
        recentPages={pageStore.recentPages}
        onAction={handleCommandAction}
        onPageSelect={handlePageSelect}
      />
    {/if}
  </main>

  {#if showOutline && pageStore.currentPage}
    <aside class="w-56 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 flex flex-col overflow-hidden shrink-0">
      <OutlinePanel
        content={pageStore.currentPage.content}
        onHeadingClick={() => {}}
      />
    </aside>
  {/if}

  {#if showBacklinks && pageStore.currentPage}
    <BacklinksPanel pageId={pageStore.currentPage.id} onNavigate={handlePageSelect} />
  {/if}

  {#if showHistory && pageStore.currentPage}
    <aside class="w-64 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 flex flex-col overflow-hidden shrink-0">
      <HistoryPanel onPageRestored={handlePageSelect} />
    </aside>
  {/if}

  {#if showRelated && pageStore.currentPage}
    <aside class="w-64 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 flex flex-col overflow-hidden shrink-0">
      <RelatedPagesPanel onNavigate={handlePageSelect} />
    </aside>
  {/if}
</div>

{#if showCommandPalette}
  <CommandPalette
    onClose={() => (showCommandPalette = false)}
    onPageSelect={handlePageSelect}
    onAction={handleCommandAction}
  />
{/if}

{#if showTemplatePicker}
  <TemplatePicker
    onSelect={handleTemplateSelect}
    onClose={() => (showTemplatePicker = false)}
  />
{/if}

{#if showSettings}
  <SettingsModal initialSection={settingsSection} onClose={() => (showSettings = false)} />
{/if}
{/if}
