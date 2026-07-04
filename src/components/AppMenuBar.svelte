<script lang="ts">
  import { t } from '../i18n'
  import type { Page } from '../lib/types'

  type MenuItem = {
    label: string
    action: string
    shortcut?: string
    disabled?: boolean
  }

  let {
    currentPage,
    onAction,
  }: {
    currentPage: Page | null
    onAction: (action: string) => void
  } = $props()

  let openMenu = $state<string | null>(null)

  let menus: { id: string; label: string; items: MenuItem[] }[] = $derived([
    {
      id: 'create',
      label: 'Create',
      items: [
        { label: 'New page', action: 'newPage', shortcut: 'Cmd+N' },
        { label: $t('command.quickCapture'), action: 'openQuickCapture', shortcut: 'Cmd+Shift+C' },
        { label: $t('quickCapture.webMode'), action: 'openWebCapture', shortcut: 'Cmd+Shift+L' },
        { label: 'Today', action: 'dailyNote' },
        { label: 'From template', action: 'newFromTemplate' },
      ],
    },
    {
      id: 'navigate',
      label: 'Navigate',
      items: [
        { label: 'Command palette', action: 'openCommandPalette', shortcut: 'Cmd+K' },
        { label: $t('command.quickCapture'), action: 'openQuickCapture', shortcut: 'Cmd+Shift+C' },
        { label: $t('quickCapture.webMode'), action: 'openWebCapture', shortcut: 'Cmd+Shift+L' },
        { label: 'Favorites', action: 'toggleFavorites' },
        { label: 'Recent pages', action: 'openRecent' },
        { label: 'Smart folders', action: 'toggleSmartFolders' },
      ],
    },
    {
      id: 'view',
      label: 'View',
      items: [
        { label: 'Knowledge graph', action: 'toggleGraph' },
        { label: $t('command.openLocalGraph'), action: 'openLocalGraph', disabled: !currentPage },
        { label: 'Outline', action: 'toggleOutline', disabled: !currentPage },
        { label: 'Backlinks', action: 'toggleBacklinks', disabled: !currentPage },
        { label: 'Related pages', action: 'toggleRelated', disabled: !currentPage },
        { label: 'Page history', action: 'toggleHistory', disabled: !currentPage },
      ],
    },
    {
      id: 'export',
      label: 'Export',
      items: [
        { label: 'Current page as Markdown', action: 'exportMarkdown', disabled: !currentPage },
        { label: 'Current page as PDF', action: 'exportPagePdf', disabled: !currentPage },
        { label: 'Workspace as PDF', action: 'exportWorkspacePdf' },
        { label: 'Import and backup', action: 'openDataSettings' },
      ],
    },
    {
      id: 'tools',
      label: 'Tools',
      items: [
        { label: 'Local sync', action: 'openSyncSettings' },
        { label: 'Sharing', action: 'openSharingSettings' },
        { label: 'Workspaces', action: 'openWorkspacesSettings' },
        { label: 'Security', action: 'openSecuritySettings' },
        { label: 'Plugins', action: 'openPluginsSettings' },
      ],
    },
  ])

  function choose(item: MenuItem) {
    if (item.disabled) return
    openMenu = null
    onAction(item.action)
  }
</script>

<svelte:window onclick={() => (openMenu = null)} />

<div class="h-11 shrink-0 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 flex items-center px-3 gap-1">
  <button
    onclick={(e) => { e.stopPropagation(); onAction('openCommandPalette') }}
    class="h-8 min-w-0 flex-1 max-w-sm flex items-center gap-2 rounded-md border border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900 px-3 text-sm text-gray-500 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-700"
  >
    <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-4.35-4.35m1.35-5.15a6.5 6.5 0 11-13 0 6.5 6.5 0 0113 0z" />
    </svg>
    <span class="truncate">Search pages, run commands, jump anywhere</span>
    <kbd class="ml-auto hidden sm:inline-flex text-[10px] text-gray-400 border border-gray-300 dark:border-gray-700 rounded px-1.5 py-0.5">Cmd+K</kbd>
  </button>

  <nav class="flex items-center gap-0.5" aria-label="Application menus">
    {#each menus as menu (menu.id)}
      <div class="relative">
        <button
          onclick={(e) => { e.stopPropagation(); openMenu = openMenu === menu.id ? null : menu.id }}
          class="h-8 px-2.5 rounded-md text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-900 {openMenu === menu.id ? 'bg-gray-100 dark:bg-gray-900' : ''}"
          aria-haspopup="menu"
          aria-expanded={openMenu === menu.id}
        >
          {menu.label}
        </button>
        {#if openMenu === menu.id}
          <div
            class="absolute right-0 top-9 z-40 w-60 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 shadow-xl py-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="menu"
            tabindex="-1"
          >
            {#each menu.items as item}
              <button
                onclick={() => choose(item)}
                disabled={item.disabled}
                class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm disabled:opacity-40 disabled:cursor-not-allowed {item.disabled ? '' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
                role="menuitem"
              >
                <span class="min-w-0 flex-1 truncate">{item.label}</span>
                {#if item.shortcut}
                  <kbd class="text-[10px] text-gray-400 border border-gray-300 dark:border-gray-700 rounded px-1.5 py-0.5">{item.shortcut}</kbd>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </nav>
</div>
