<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog'
  import { getVersion } from '@tauri-apps/api/app'
  import { onMount } from 'svelte'
  import { readTextFile } from '@tauri-apps/plugin-fs'
  import { settingsStore, syncStore, workspaceStore, pageStore, encryptionStore } from '../../stores/app.svelte'
  import { pluginStore } from '../../stores/plugins.svelte'
  import { t, locales, setLocale, type Locale } from '../../i18n'
  import * as api from '../../lib/api'

  type SettingsSection = 'appearance' | 'language' | 'data' | 'sync' | 'sharing' | 'workspaces' | 'security' | 'plugins' | 'about'

  let {
    onClose,
    initialSection = 'appearance',
  }: {
    onClose: () => void
    initialSection?: SettingsSection
  } = $props()

  let activeSection = $state<SettingsSection>('appearance')
  let deviceName = $state('')
  let port = $state(9876)
  let syncPairingSecret = $state('')
  let importStatus = $state<{ type: 'success' | 'error'; message: string } | null>(null)
  let appVersion = $state('1.6.0')

  onMount(async () => {
    try {
      appVersion = await getVersion()
    } catch {
      // Browser preview does not expose Tauri application metadata.
    }
  })

  $effect(() => {
    activeSection = initialSection
    if (initialSection === 'sync') syncStore.loadStatus()
    if (initialSection === 'workspaces') workspaceStore.load()
    if (initialSection === 'security') encryptionStore.checkStatus()
    if (initialSection === 'plugins') pluginStore.loadPlugins()
  })

  async function handleExport() {
    await pageStore.flushPendingEdits?.()
    const json = await api.exportWorkspace()
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `900notes-export-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  async function handleImport() {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json'
    input.onchange = async () => {
      const file = input.files?.[0]
      if (!file) return
      if (!confirm($t('settings.restoreConfirm'))) return
      await pageStore.flushPendingEdits?.()
      const text = await file.text()
      const count = await api.importWorkspace(text)
      alert($t('settings.restoreComplete', { count }))
      location.reload()
    }
    input.click()
  }

  function singlePath(selection: string | string[] | null): string | null {
    if (!selection) return null
    return Array.isArray(selection) ? (selection[0] ?? null) : selection
  }

  async function handleImportExternal(kind: 'notion' | 'obsidian' | 'evernote' | 'roam') {
    importStatus = null
    try {
      if (kind === 'notion' || kind === 'obsidian') {
        const selected = singlePath(await open({ directory: true, multiple: false }))
        if (!selected) return
        const result = kind === 'notion'
          ? await api.importNotion(selected)
          : await api.importObsidian(selected)
        importStatus = importResultMessage(kind, result)
      } else {
        const selected = singlePath(await open({
          multiple: false,
          filters: [{ name: kind === 'evernote' ? 'Evernote export' : 'Roam export', extensions: kind === 'evernote' ? ['enex'] : ['json'] }],
        }))
        if (!selected) return
        const text = await readTextFile(selected)
        const result = kind === 'evernote'
          ? await api.importEvernote(text)
          : await api.importRoam(text)
        importStatus = importResultMessage(kind, result)
      }
      await pageStore.loadPageTree()
      await pageStore.loadRecentPages()
    } catch (e) {
      importStatus = { type: 'error', message: String(e) }
    }
  }

  function importResultMessage(kind: string, result: api.ImportResult) {
    if (result.errors.length === 0) {
      return { type: 'success' as const, message: `Imported ${result.pagesCreated} pages from ${kind}.` }
    }
    const details = result.errors.slice(0, 3).join(' | ')
    const remaining = result.errors.length > 3 ? ` (+${result.errors.length - 3} more)` : ''
    return {
      type: 'error' as const,
      message: `Imported ${result.pagesCreated} pages from ${kind}, with ${result.errors.length} errors: ${details}${remaining}`,
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }

  async function handleStartSync() {
    await pageStore.flushPendingEdits?.()
    await syncStore.start(deviceName || undefined, port || undefined, syncPairingSecret)
  }

  async function handleStopSync() {
    await syncStore.stop()
  }

  async function handleSyncWithPeer(peerId: string) {
    await pageStore.flushPendingEdits?.()
    await syncStore.syncWithPeer(peerId)
    if (pageStore.currentPage) await pageStore.loadPage(pageStore.currentPage.id)
    await pageStore.loadPageTree()
  }

  async function handleRefreshPeers() {
    await syncStore.loadStatus()
  }

  async function handleApplyCrdt() {
    await pageStore.flushPendingEdits?.()
    await syncStore.applyCrdtToDb()
    if (pageStore.currentPage) await pageStore.loadPage(pageStore.currentPage.id)
    await pageStore.loadPageTree()
  }

  let sharePageIds = $state('')
  let sharePassphrase = $state('')
  let importPassphrase = $state('')
  let newWorkspaceName = $state('')
  let renameWorkspaceId = $state('')
  let renameWorkspaceName = $state('')

  async function handleExportShareBundle() {
    const ids = sharePageIds.split(',').map(s => s.trim()).filter(Boolean)
    if (ids.length === 0 || !sharePassphrase) return
    await pageStore.flushPendingEdits?.()
    const encrypted = await api.exportShareBundle(ids, sharePassphrase)
    const blob = new Blob([encrypted], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `900notes-share-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  }

  async function handleImportShareBundle() {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json'
    input.onchange = async () => {
      const file = input.files?.[0]
      if (!file || !importPassphrase) return
      const text = await file.text()
      const count = await api.importShareBundle(text, importPassphrase)
      alert(`Imported ${count} pages from share bundle`)
      await pageStore.loadPageTree()
    }
    input.click()
  }

  async function handleExportPageHtml() {
    if (!pageStore.currentPage) return
    await pageStore.flushPendingEdits?.()
    const html = await api.exportPageHtml(pageStore.currentPage.id)
    const blob = new Blob([html], { type: 'text/html' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${pageStore.currentPage.title.replace(/[^a-zA-Z0-9]/g, '_')}.html`
    a.click()
    URL.revokeObjectURL(url)
  }

  async function handleCreateWorkspace() {
    if (!newWorkspaceName.trim()) return
    await workspaceStore.create(newWorkspaceName.trim())
    newWorkspaceName = ''
  }

  async function handleSwitchWorkspace(id: string) {
    if (id === workspaceStore.activeWorkspace?.id) return
    await workspaceStore.switch(id)
    onClose()
  }

  async function handleDeleteWorkspace(id: string) {
    const workspace = workspaceStore.workspaces.find((item) => item.id === id)
    if (!confirm($t('workspaces.deleteConfirm', { name: workspace?.name ?? '' }))) return
    await workspaceStore.remove(id)
  }

  async function handleRenameWorkspace() {
    if (!renameWorkspaceId || !renameWorkspaceName.trim()) return
    await workspaceStore.rename(renameWorkspaceId, renameWorkspaceName.trim())
    renameWorkspaceId = ''
    renameWorkspaceName = ''
  }

  let enablePassphrase = $state('')
  let enableConfirm = $state('')
  let unlockPassphrase = $state('')
  let disablePassphrase = $state('')
  let changeOldPassphrase = $state('')
  let changeNewPassphrase = $state('')
  let exportEncPassphrase = $state('')

  async function handleEnableEncryption() {
    if (!enablePassphrase || enablePassphrase !== enableConfirm) return
    await pageStore.flushPendingEdits?.()
    await encryptionStore.enable(enablePassphrase)
    enablePassphrase = ''
    enableConfirm = ''
  }

  async function handleUnlock() {
    if (!unlockPassphrase) return
    await encryptionStore.unlock(unlockPassphrase)
    unlockPassphrase = ''
  }

  async function handleDisableEncryption() {
    if (!disablePassphrase) return
    await pageStore.flushPendingEdits?.()
    await encryptionStore.disable(disablePassphrase)
    disablePassphrase = ''
  }

  async function handleChangePassphrase() {
    if (!changeOldPassphrase || !changeNewPassphrase) return
    await pageStore.flushPendingEdits?.()
    await encryptionStore.changePassphrase(changeOldPassphrase, changeNewPassphrase)
    changeOldPassphrase = ''
    changeNewPassphrase = ''
  }

  async function handleExportEncrypted() {
    if (!exportEncPassphrase) return
    await pageStore.flushPendingEdits?.()
    const encoded = await api.exportEncryptedWorkspace(exportEncPassphrase)
    const blob = new Blob([encoded], { type: 'application/octet-stream' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `900notes-encrypted-export-${Date.now()}.enc`
    a.click()
    URL.revokeObjectURL(url)
    exportEncPassphrase = ''
  }

  async function handleImportEncrypted() {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.enc,.json'
    input.onchange = async () => {
      const file = input.files?.[0]
      if (!file || !exportEncPassphrase) return
      if (!confirm($t('settings.restoreConfirm'))) return
      await pageStore.flushPendingEdits?.()
      const text = await file.text()
      const count = await api.importEncryptedWorkspace(text, exportEncPassphrase)
      alert($t('settings.restoreComplete', { count }))
      location.reload()
    }
    input.click()
  }

  let secureDeletePageId = $state('')

  async function handleSecureDeletePage() {
    const id = secureDeletePageId || pageStore.currentPage?.id
    if (!id) return
    if (!confirm($t('security.secureDeleteConfirm'))) return
    await pageStore.secureDeletePage(id)
    secureDeletePageId = ''
  }

  async function handleSecureEmptyTrash() {
    if (!confirm($t('security.secureDeleteConfirm'))) return
    await pageStore.secureEmptyTrash()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 bg-black/30 flex items-center justify-center"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose() }}
  role="presentation"
>
  <div
    class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-hidden border border-gray-200 dark:border-gray-700 flex"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="{$t('settings.title')}"
    tabindex="-1"
  >
    <!-- Sidebar -->
    <div class="w-48 bg-gray-50 dark:bg-gray-900/50 border-r border-gray-200 dark:border-gray-700 p-2 shrink-0">
      <div class="px-2 py-3 text-sm font-semibold text-gray-900 dark:text-gray-100">{$t('settings.title')}</div>
      <button
        onclick={() => (activeSection = 'appearance')}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'appearance' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('settings.theme')}</button>
      <button
        onclick={() => (activeSection = 'language')}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'language' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('settings.language')}</button>
      <button
        onclick={() => (activeSection = 'data')}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'data' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('settings.dataLocation')}</button>
      <button
        onclick={() => { activeSection = 'sync'; syncStore.loadStatus() }}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'sync' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('sync.title')}</button>
      <button
        onclick={() => (activeSection = 'sharing')}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'sharing' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('sharing.title')}</button>
      <button
        onclick={() => { activeSection = 'workspaces'; workspaceStore.load() }}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'workspaces' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('workspaces.title')}</button>
      <button
        onclick={() => { activeSection = 'security'; encryptionStore.checkStatus() }}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'security' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('security.title')}</button>
      <button
        onclick={() => { activeSection = 'plugins'; pluginStore.loadPlugins() }}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'plugins' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >Plugins</button>
      <button
        onclick={() => (activeSection = 'about')}
        class="w-full text-left px-2 py-1.5 rounded-md text-sm {activeSection === 'about' ? 'bg-accent/10 text-accent font-medium' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700'}"
      >{$t('settings.about')}</button>
    </div>

    <!-- Content -->
    <div class="flex-1 p-6 overflow-y-auto text-gray-900 dark:text-gray-100">
      {#if activeSection === 'appearance'}
        <h2 class="text-lg font-semibold mb-4">{$t('settings.theme')}</h2>
        <div class="space-y-3">
          <div>
            <span class="text-sm font-medium block mb-2">{$t('settings.theme')}</span>
            <div class="flex gap-2 flex-wrap">
              <button
                onclick={() => settingsStore.setTheme('light')}
                class="px-3 py-2 rounded-lg text-sm border {settingsStore.theme === 'light' ? 'border-accent bg-accent/10 text-accent' : 'border-gray-300 dark:border-gray-600'}"
              >{$t('settings.theme.light')}</button>
              <button
                onclick={() => settingsStore.setTheme('dark')}
                class="px-3 py-2 rounded-lg text-sm border {settingsStore.theme === 'dark' ? 'border-accent bg-accent/10 text-accent' : 'border-gray-300 dark:border-gray-600'}"
              >{$t('settings.theme.dark')}</button>
              <button
                onclick={() => settingsStore.setTheme('system')}
                class="px-3 py-2 rounded-lg text-sm border {settingsStore.theme === 'system' ? 'border-accent bg-accent/10 text-accent' : 'border-gray-300 dark:border-gray-600'}"
              >{$t('settings.theme.system')}</button>
              <button
                onclick={() => settingsStore.setTheme('high-contrast')}
                class="px-3 py-2 rounded-lg text-sm border {settingsStore.theme === 'high-contrast' ? 'border-accent bg-accent/10 text-accent' : 'border-gray-300 dark:border-gray-600'}"
              >{$t('settings.theme.highContrast')}</button>
            </div>
          </div>
          <div>
            <label for="font-size-slider" class="text-sm font-medium block mb-2">{$t('settings.fontSize')}: {settingsStore.fontSize}px</label>
            <input
              id="font-size-slider"
              type="range"
              min="12"
              max="24"
              value={settingsStore.fontSize}
              oninput={(e) => settingsStore.setFontSize(parseInt(e.currentTarget.value, 10))}
              class="w-full"
            />
          </div>
          <div>
            <label for="line-spacing-slider" class="text-sm font-medium block mb-2">{$t('settings.lineSpacing')}: {settingsStore.lineSpacing}</label>
            <input
              id="line-spacing-slider"
              type="range"
              min="1"
              max="2.5"
              step="0.1"
              value={settingsStore.lineSpacing}
              oninput={(e) => settingsStore.setLineSpacing(parseFloat(e.currentTarget.value))}
              class="w-full"
            />
          </div>
        </div>
      {:else if activeSection === 'language'}
        <h2 class="text-lg font-semibold mb-4">{$t('settings.language')}</h2>
        <div class="space-y-1">
          {#each locales as loc}
            <button
              onclick={() => { setLocale(loc.code); settingsStore.setLanguage(loc.code) }}
              class="w-full text-left px-3 py-2 rounded-lg text-sm {settingsStore.language === loc.code ? 'bg-accent/10 text-accent' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
            >
              <span class="font-medium">{loc.name}</span>
              {#if loc.rtl}<span class="text-xs text-gray-400 ml-2">RTL</span>{/if}
            </button>
          {/each}
        </div>
      {:else if activeSection === 'data'}
        <h2 class="text-lg font-semibold mb-4">{$t('settings.dataLocation')}</h2>
        <div class="space-y-4">
          <p class="text-sm text-gray-500 dark:text-gray-400">
            All data is stored locally in a single SQLite file. No cloud. No server.
          </p>
          <div class="flex gap-2">
            <button
              onclick={handleExport}
              class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark"
            >{$t('settings.export')}</button>
            <button
              onclick={handleImport}
              class="px-4 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
            >{$t('settings.import')}</button>
          </div>

          <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
            <h3 class="text-sm font-semibold mb-2">Import from other apps</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">
              Bring an existing knowledge base into 900Notes without leaving the local app.
            </p>
            <div class="grid grid-cols-2 gap-2">
              <button
                onclick={() => handleImportExternal('notion')}
                class="px-3 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
              >Notion export folder</button>
              <button
                onclick={() => handleImportExternal('obsidian')}
                class="px-3 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
              >Obsidian vault folder</button>
              <button
                onclick={() => handleImportExternal('evernote')}
                class="px-3 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
              >Evernote ENEX file</button>
              <button
                onclick={() => handleImportExternal('roam')}
                class="px-3 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
              >Roam JSON file</button>
            </div>
            {#if importStatus}
              <p class="mt-3 text-sm {importStatus.type === 'success' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
                {importStatus.message}
              </p>
            {/if}
          </div>
        </div>
      {:else if activeSection === 'sync'}
        <h2 class="text-lg font-semibold mb-4">{$t('sync.title')}</h2>
        <div class="space-y-4">
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {$t('sync.description')}
          </p>

          {#if !syncStore.enabled}
            <div class="space-y-3">
              <div>
                <label class="text-sm font-medium block mb-1" for="sync-device-name">{$t('sync.deviceName')}</label>
                <input
                  id="sync-device-name"
                  type="text"
                  bind:value={deviceName}
                  placeholder={$t('sync.deviceNamePlaceholder')}
                  class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
                />
              </div>
              <div>
                <label class="text-sm font-medium block mb-1" for="sync-port">{$t('sync.port')}</label>
                <input
                  id="sync-port"
                  type="number"
                  bind:value={port}
                  min="1024"
                  max="65535"
                  class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
                />
              </div>
              <div>
                <label class="text-sm font-medium block mb-1" for="sync-pairing-secret">{$t('sync.pairingSecret')}</label>
                <input
                  id="sync-pairing-secret"
                  type="password"
                  bind:value={syncPairingSecret}
                  minlength="12"
                  autocomplete="new-password"
                  class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
                />
                <p class="mt-1 text-xs text-gray-500">{$t('sync.pairingSecretHelp')}</p>
              </div>
              <button
                onclick={handleStartSync}
                disabled={syncPairingSecret.trim().length < 12}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark"
              >{$t('sync.start')}</button>
            </div>
          {:else}
            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <span class="inline-block w-2 h-2 rounded-full bg-green-500"></span>
                <span class="text-sm font-medium">{$t('sync.active')}</span>
              </div>

              {#if syncStore.lastSync}
                <div class="text-xs text-gray-400">
                  {$t('sync.lastSync')}: {new Date(syncStore.lastSync).toLocaleString()}
                </div>
              {/if}

              {#if syncStore.pendingCount > 0}
                <div class="text-xs text-amber-500">
                  {$t('sync.pendingItems', { count: syncStore.pendingCount })}
                </div>
              {/if}

              <div class="flex gap-2">
                <button
                  onclick={handleRefreshPeers}
                  class="px-3 py-1.5 rounded-lg text-sm border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700"
                >{$t('sync.refreshPeers')}</button>
                <button
                  onclick={handleApplyCrdt}
                  disabled={syncStore.syncing}
                  class="px-3 py-1.5 rounded-lg text-sm border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50"
                >{$t('sync.applyChanges')}</button>
                <button
                  onclick={handleStopSync}
                  class="px-3 py-1.5 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600"
                >{$t('sync.stop')}</button>
              </div>

              <div>
                <h3 class="text-sm font-semibold mb-2">{$t('sync.discoveredDevices')}</h3>
                {#if syncStore.peers.length === 0}
                  <p class="text-sm text-gray-400">{$t('sync.noDevices')}</p>
                {:else}
                  <div class="space-y-2">
                    {#each syncStore.peers as peer (peer.id)}
                      <div class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                        <div>
                          <div class="text-sm font-medium">{peer.name}</div>
                          <div class="text-xs text-gray-400">{peer.host}:{peer.port}</div>
                        </div>
                        <button
                          onclick={() => handleSyncWithPeer(peer.id)}
                          disabled={syncStore.syncing}
                          class="px-3 py-1 rounded-lg text-xs font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
                        >{#if syncStore.syncing}{$t('sync.syncing')}{:else}{$t('sync.syncNow')}{/if}</button>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          {#if syncStore.error}
            <p class="text-sm text-red-500">{syncStore.error}</p>
          {/if}
        </div>
      {:else if activeSection === 'sharing'}
        <h2 class="text-lg font-semibold mb-4">{$t('sharing.title')}</h2>
        <div class="space-y-6">
          <div>
            <h3 class="text-sm font-semibold mb-2">{$t('sharing.exportBundle')}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{$t('sharing.exportDescription')}</p>
            <div class="space-y-2">
              <input
                type="text"
                bind:value={sharePageIds}
                placeholder={$t('sharing.pageIdsPlaceholder')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <input
                type="password"
                bind:value={sharePassphrase}
                placeholder={$t('sharing.passphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <button
                onclick={handleExportShareBundle}
                disabled={!sharePageIds || !sharePassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('sharing.exportBundle')}</button>
            </div>
          </div>

          <div>
            <h3 class="text-sm font-semibold mb-2">{$t('sharing.importBundle')}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{$t('sharing.importDescription')}</p>
            <div class="space-y-2">
              <input
                type="password"
                bind:value={importPassphrase}
                placeholder={$t('sharing.passphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <button
                onclick={handleImportShareBundle}
                disabled={!importPassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('sharing.importBundle')}</button>
            </div>
          </div>

          <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
            <h3 class="text-sm font-semibold mb-2">{$t('sharing.htmlExport')}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{$t('sharing.htmlExportDescription')}</p>
            <button
              onclick={handleExportPageHtml}
              disabled={!pageStore.currentPage}
              class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
            >{$t('sharing.exportCurrentPage')}</button>
          </div>
        </div>
      {:else if activeSection === 'workspaces'}
        <h2 class="text-lg font-semibold mb-4">{$t('workspaces.title')}</h2>
        <div class="space-y-4">
          <p class="text-sm text-gray-500 dark:text-gray-400">{$t('workspaces.description')}</p>

          <div class="space-y-2">
            {#each workspaceStore.workspaces as ws (ws.id)}
              <div class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div>
                  <div class="text-sm font-medium">
                    {ws.name}
                    {#if ws.id === workspaceStore.activeWorkspace?.id}
                      <span class="text-xs text-accent ml-2">{$t('workspaces.active')}</span>
                    {/if}
                  </div>
                  <div class="text-xs text-gray-400">{new Date(ws.createdAt).toLocaleDateString()}</div>
                </div>
                <div class="flex gap-2">
                  {#if ws.id !== workspaceStore.activeWorkspace?.id}
                    <button
                      onclick={() => handleSwitchWorkspace(ws.id)}
                      class="px-3 py-1 rounded-lg text-xs font-medium bg-accent text-white hover:bg-accent-dark"
                    >{$t('workspaces.switch')}</button>
                  {/if}
                  {#if !ws.isDefault}
                    <button
                      onclick={() => handleDeleteWorkspace(ws.id)}
                      class="px-3 py-1 rounded-lg text-xs font-medium bg-red-500 text-white hover:bg-red-600"
                    >{$t('workspaces.delete')}</button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-2">
            <h3 class="text-sm font-semibold">{$t('workspaces.create')}</h3>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={newWorkspaceName}
                placeholder={$t('workspaces.namePlaceholder')}
                class="flex-1 px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700"
              />
              <button
                onclick={handleCreateWorkspace}
                disabled={!newWorkspaceName.trim()}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('workspaces.create')}</button>
            </div>
          </div>
        </div>
      {:else if activeSection === 'security'}
        <h2 class="text-lg font-semibold mb-4">{$t('security.title')}</h2>
        <div class="space-y-6">
          <p class="text-sm text-gray-500 dark:text-gray-400">{$t('security.description')}</p>

          {#if encryptionStore.error}
            <div class="text-sm text-red-500 bg-red-50 dark:bg-red-900/20 p-3 rounded-lg">
              {encryptionStore.error}
            </div>
          {/if}

          {#if !encryptionStore.enabled}
            <div class="space-y-3">
              <h3 class="text-sm font-semibold">{$t('security.enableEncryption')}</h3>
              <p class="text-sm text-gray-500 dark:text-gray-400">{$t('security.enableDescription')}</p>
              <input
                type="password"
                bind:value={enablePassphrase}
                placeholder={$t('security.passphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <input
                type="password"
                bind:value={enableConfirm}
                placeholder={$t('security.confirmPassphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              {#if enablePassphrase && enableConfirm && enablePassphrase !== enableConfirm}
                <p class="text-xs text-red-500">{$t('security.passphraseMismatch')}</p>
              {/if}
              <button
                onclick={handleEnableEncryption}
                disabled={!enablePassphrase || enablePassphrase !== enableConfirm}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('security.enableEncryption')}</button>
            </div>
          {:else if !encryptionStore.unlocked}
            <div class="space-y-3">
              <h3 class="text-sm font-semibold">{$t('security.unlockDatabase')}</h3>
              <p class="text-sm text-gray-500 dark:text-gray-400">{$t('security.unlockDescription')}</p>
              <input
                type="password"
                bind:value={unlockPassphrase}
                placeholder={$t('security.passphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <button
                onclick={handleUnlock}
                disabled={!unlockPassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('security.unlock')}</button>
            </div>
          {:else}
            <div class="space-y-2">
              <div class="flex items-center gap-2">
                <span class="inline-block w-2 h-2 rounded-full bg-green-500"></span>
                <span class="text-sm font-medium">{$t('security.encryptionActive')}</span>
              </div>
            </div>

            <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
              <h3 class="text-sm font-semibold">{$t('security.changePassphrase')}</h3>
              <input
                type="password"
                bind:value={changeOldPassphrase}
                placeholder={$t('security.currentPassphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <input
                type="password"
                bind:value={changeNewPassphrase}
                placeholder={$t('security.newPassphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <button
                onclick={handleChangePassphrase}
                disabled={!changeOldPassphrase || !changeNewPassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('security.changePassphrase')}</button>
            </div>

            <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
              <h3 class="text-sm font-semibold">{$t('security.disableEncryption')}</h3>
              <p class="text-sm text-red-500">{$t('security.disableWarning')}</p>
              <input
                type="password"
                bind:value={disablePassphrase}
                placeholder={$t('security.passphrase')}
                class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
              />
              <button
                onclick={handleDisableEncryption}
                disabled={!disablePassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600 disabled:opacity-50"
              >{$t('security.disableEncryption')}</button>
            </div>
          {/if}

          <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
            <h3 class="text-sm font-semibold">{$t('security.encryptedExport')}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">{$t('security.encryptedExportDescription')}</p>
            <input
              type="password"
              bind:value={exportEncPassphrase}
              placeholder={$t('security.exportPassphrase')}
              class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
            />
            <div class="flex gap-2">
              <button
                onclick={handleExportEncrypted}
                disabled={!exportEncPassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
              >{$t('security.exportEncrypted')}</button>
              <button
                onclick={handleImportEncrypted}
                disabled={!exportEncPassphrase}
                class="px-4 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50"
              >{$t('security.importEncrypted')}</button>
            </div>
          </div>

          <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
            <h3 class="text-sm font-semibold">{$t('security.secureDelete')}</h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">{$t('security.secureDeleteDescription')}</p>
            <input
              type="text"
              bind:value={secureDeletePageId}
              placeholder="Page ID (leave empty to delete current page)"
              class="w-full px-3 py-2 rounded-lg text-sm border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500"
            />
            <div class="flex gap-2">
              <button
                onclick={handleSecureDeletePage}
                class="px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600"
              >{$t('security.secureDeletePage')}</button>
              <button
                onclick={handleSecureEmptyTrash}
                class="px-4 py-2 rounded-lg text-sm font-medium border border-red-300 dark:border-red-700 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
              >{$t('security.secureEmptyTrash')}</button>
            </div>
          </div>
        </div>
      {:else if activeSection === 'plugins'}
        <h2 class="text-lg font-semibold mb-4">Plugins</h2>
        <div class="space-y-3">
          {#if pluginStore.loading}
            <p class="text-sm text-gray-500 dark:text-gray-400">Loading...</p>
          {:else if pluginStore.plugins.length === 0}
            <p class="text-sm text-gray-500 dark:text-gray-400">No plugins installed. Place plugin directories in the <code class="text-xs bg-gray-100 dark:bg-gray-800 px-1 rounded">plugins/</code> folder in your app data directory and click "Scan for Plugins".</p>
          {:else}
            {#each pluginStore.plugins as plugin (plugin.id)}
              <div class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-sm truncate">{plugin.name}</span>
                    <span class="text-xs text-gray-400">v{plugin.version}</span>
                  </div>
                  <p class="text-xs text-gray-500 truncate">{plugin.description}</p>
                  <p class="text-xs text-gray-400">by {plugin.author}</p>
                </div>
                <div class="flex items-center gap-2 ml-3">
                  <button
                    onclick={() => pluginStore.togglePlugin(plugin.id, !plugin.enabled)}
                    class="px-3 py-1 rounded text-xs font-medium {plugin.enabled ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}"
                  >{plugin.enabled ? 'Enabled' : 'Disabled'}</button>
                  <button
                    onclick={() => pluginStore.removePlugin(plugin.id)}
                    class="px-3 py-1 rounded text-xs font-medium text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
                  >Remove</button>
                </div>
              </div>
            {/each}
          {/if}
          {#if pluginStore.error}
            <p class="text-sm text-red-500">{pluginStore.error}</p>
          {/if}
          <div class="pt-3 border-t border-gray-200 dark:border-gray-700">
            <button
              onclick={() => pluginStore.scanAndInstall()}
              class="px-4 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent/90"
            >Scan for Plugins</button>
          </div>
        </div>
      {:else if activeSection === 'about'}
        <h2 class="text-lg font-semibold mb-4">{$t('settings.about')}</h2>
        <div class="space-y-3 text-sm">
          <div>
            <span class="text-gray-500 dark:text-gray-400">{$t('settings.version')}: </span>
            <span class="font-medium">{appVersion}</span>
          </div>
          <div>
            <span class="text-gray-500 dark:text-gray-400">{$t('settings.license')}: </span>
            <span class="font-medium">Apache 2.0</span>
          </div>
          <div class="pt-4 border-t border-gray-200 dark:border-gray-700">
            <p class="text-gray-600 dark:text-gray-400">
              {$t('about.what')}
            </p>
            <p class="text-gray-600 dark:text-gray-400 mt-3">
              {$t('about.how')}
            </p>
            <p class="text-gray-500 dark:text-gray-500 text-xs mt-3">
              {$t('about.mission')}
            </p>
            <p class="text-gray-400 dark:text-gray-600 text-xs mt-2">
              {$t('about.free')}
            </p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
