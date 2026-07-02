<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import type { Page } from '../../lib/types'
  import { pageStore, tagStore, attachmentStore, historyStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import * as editorLib from '../../lib/editor'
  import * as api from '../../lib/api'
  import type { EditorView as PMEditorView } from 'prosemirror-view'
  import EditorToolbar from './EditorToolbar.svelte'
  import SlashMenu from './SlashMenu.svelte'
  import WikiLinkAutocomplete from './WikiLinkAutocomplete.svelte'
  import TagInput from './TagInput.svelte'
  import PageProperties from './PageProperties.svelte'

  let {
    page,
    onNavigate,
    onAppAction = () => {},
    showOutline = false,
  }: {
    page: Page
    onNavigate: (pageId: string) => void
    onAppAction?: (action: string) => void
    showOutline?: boolean
  } = $props()

  let editorElement: HTMLElement
  let pmView: PMEditorView | null = null
  let titleValue = $state('')
  let iconValue = $state('')
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  let showSlashMenu = $state(false)
  let slashMenuPos = $state({ x: 0, y: 0 })
  let showWikiAutocomplete = $state(false)
  let wikiQuery = $state('')
  let exporting = $state(false)
  let ocrLoading = $state<string | null>(null)
  let toast = $state<{ msg: string; type: 'success' | 'error' } | null>(null)
  let recording = $state(false)
  let recordDuration = $state(0)
  let isFavorite = $state(false)

  let pageTitles = $state<{ id: string; title: string }[]>([])

  async function loadPageTitles() {
    const titles = await api.getPageTitles()
    pageTitles = titles.map(([id, title]) => ({ id, title }))
  }

  async function loadFavoriteState() {
    isFavorite = await historyStore.checkFavorite(page.id)
  }

  function debounceSave(content: string) {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(async () => {
      await pageStore.updatePage(page.id, { content })
    }, 500)
  }

  async function saveTitle() {
    if (titleValue !== page.title) {
      await pageStore.updatePage(page.id, { title: titleValue })
    }
  }

  async function saveIcon() {
    if (iconValue !== (page.icon || '')) {
      await pageStore.updatePage(page.id, { icon: iconValue || null })
    }
  }

  function handleWikiLinkClick(title: string) {
    const matched = pageTitles.find(
      (p) => p.title.toLowerCase() === title.toLowerCase(),
    )
    if (matched) {
      onNavigate(matched.id)
    }
  }

  function handleWikiLinkStart(e: Event) {
    const detail = (e as CustomEvent).detail
    wikiQuery = ''
    showWikiAutocomplete = true
  }

  function handleWikiLinkSelect(title: string, pageId: string | null) {
    if (pmView) {
      editorLib.insertWikiLink(pmView, title, pageId)
    }
    showWikiAutocomplete = false
  }

  function showToast(msg: string, type: 'success' | 'error') {
    toast = { msg, type }
    setTimeout(() => { toast = null }, 3000)
  }

  async function handleExportPdf() {
    if (exporting) return
    exporting = true
    try {
      const bytes = await api.exportPagePdf(page.id)
      const blob = new Blob([new Uint8Array(bytes)], { type: 'application/pdf' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${page.title || 'untitled'}.pdf`
      a.click()
      URL.revokeObjectURL(url)
      showToast($t('pdf.exported'), 'success')
    } catch (e) {
      showToast($t('pdf.exportFailed') + ': ' + String(e), 'error')
    }
    exporting = false
  }

  async function handleOcrAttachment(attachmentId: string) {
    if (ocrLoading) return
    ocrLoading = attachmentId
    try {
      const text = await api.ocrAttachment(attachmentId)
      showToast($t('ocr.extracted'), 'success')
      if (pmView && text.trim()) {
        const tr = pmView.state.tr.insertText('\n' + text.trim() + '\n')
        pmView.dispatch(tr)
      }
    } catch (e) {
      const msg = String(e)
      if (msg.includes('not installed')) {
        showToast($t('ocr.notInstalled'), 'error')
      } else {
        showToast($t('ocr.extractFailed') + ': ' + msg, 'error')
      }
    }
    ocrLoading = null
  }

  async function handleRecordAudio() {
    if (recording) {
      editorLib.stopAudioRecording()
      return
    }
    if (!pmView) return

    recording = true
    recordDuration = 0

    const result = await editorLib.recordAudio((rec, dur) => {
      recording = rec
      recordDuration = dur
    })

    if (!result) {
      showToast($t('audio.recordFailed'), 'error')
      return
    }

    const title = `Audio ${new Date().toLocaleTimeString()}`
    try {
      await editorLib.insertAudioBlock(
        pmView,
        page.id,
        result.data,
        result.mimeType,
        result.duration,
        title,
      )
      showToast($t('audio.recorded'), 'success')
    } catch (e) {
      showToast($t('audio.recordFailed') + ': ' + String(e), 'error')
    }
  }

  async function handleToggleFavorite() {
    if (isFavorite) {
      await historyStore.removeFavorite(page.id)
      isFavorite = false
    } else {
      await historyStore.addFavorite(page.id)
      isFavorite = true
    }
  }

  onMount(async () => {
    await loadPageTitles()
    await loadFavoriteState()

    pmView = editorLib.createEditor(editorElement, page.content, {
      onChange: (content) => debounceSave(content),
      onWikiLinkClick: handleWikiLinkClick,
      getPageTitles: () => pageTitles,
      getPageId: () => page.id,
    })

    window.addEventListener('wiki-link-start', handleWikiLinkStart as EventListener)
  })

  onDestroy(() => {
    if (pmView) {
      editorLib.destroyEditor(pmView)
    }
    if (saveTimer) {
      clearTimeout(saveTimer)
    }
    window.removeEventListener('wiki-link-start', handleWikiLinkStart as EventListener)
  })

  // Re-create editor when page changes
  let lastPageId = ''
  $effect(() => {
    if (page.id !== lastPageId) {
      lastPageId = page.id
      titleValue = page.title
      iconValue = page.icon || ''
      if (pmView) {
        editorLib.destroyEditor(pmView)
      }
      pmView = editorLib.createEditor(editorElement, page.content, {
        onChange: (content) => debounceSave(content),
        onWikiLinkClick: handleWikiLinkClick,
        getPageTitles: () => pageTitles,
        getPageId: () => page.id,
      })
      loadFavoriteState()
    }
  })

  async function handleToolbarAction(action: string) {
    if (!pmView) return
    switch (action) {
      case 'bold': editorLib.toggleBold(pmView); break
      case 'italic': editorLib.toggleItalic(pmView); break
      case 'underline': editorLib.toggleUnderline(pmView); break
      case 'strike': editorLib.toggleStrike(pmView); break
      case 'code': editorLib.toggleCode(pmView); break
      case 'h1': editorLib.setHeading(pmView, 1); break
      case 'h2': editorLib.setHeading(pmView, 2); break
      case 'h3': editorLib.setHeading(pmView, 3); break
      case 'paragraph': editorLib.setParagraph(pmView); break
      case 'bulletList': editorLib.toggleBulletList(pmView); break
      case 'orderedList': editorLib.toggleOrderedList(pmView); break
      case 'todo': editorLib.insertTodoItem(pmView); break
      case 'codeBlock': editorLib.setCodeBlock(pmView); break
      case 'quote': editorLib.toggleBlockquote(pmView); break
      case 'divider': editorLib.insertDivider(pmView); break
      case 'audio': await handleRecordAudio(); break
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === '/' && pmView) {
      const sel = window.getSelection()
      if (sel && sel.rangeCount > 0) {
        const range = sel.getRangeAt(0)
        const rect = range.getBoundingClientRect()
        slashMenuPos = { x: rect.left, y: rect.bottom + 4 }
        showSlashMenu = true
      }
    }
  }

  function handleSlashSelect(action: string) {
    showSlashMenu = false
    handleToolbarAction(action)
  }
</script>

<div class="flex-1 flex flex-col overflow-hidden bg-white dark:bg-gray-950">
  <!-- Toolbar -->
  <EditorToolbar onAction={handleToolbarAction} />

  <!-- Page Actions Bar -->
  <div class="flex items-center gap-1.5 px-12 py-1.5 max-w-3xl mx-auto w-full border-b border-gray-100 dark:border-gray-800/50 flex-wrap">
    <button
      onclick={handleToggleFavorite}
      class="text-xs px-2.5 py-1 rounded-md {isFavorite ? 'text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20' : 'text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'} transition-colors"
      title={isFavorite ? $t('favorites.remove') : $t('favorites.add')}
    >
      {isFavorite ? 'Favorited' : 'Favorite'}
    </button>
    <button
      onclick={() => onAppAction('toggleOutline')}
      class="text-xs px-2.5 py-1 rounded-md {showOutline ? 'text-accent bg-accent/10' : 'text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'} transition-colors"
      title={$t('command.toggleOutline')}
    >
      Outline
    </button>
    <button
      onclick={() => onAppAction('toggleBacklinks')}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('command.toggleBacklinks')}
    >
      Backlinks
    </button>
    <button
      onclick={() => onAppAction('toggleRelated')}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('command.toggleRelated')}
    >
      Related
    </button>
    <button
      onclick={() => onAppAction('toggleHistory')}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('command.toggleHistory')}
    >
      History
    </button>
    <button
      onclick={() => onAppAction('openLocalGraph')}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('command.openLocalGraph')}
    >
      {$t('command.openLocalGraph')}
    </button>
    <div class="w-px h-5 bg-gray-200 dark:bg-gray-800 mx-1"></div>
    <button
      onclick={() => onAppAction('exportMarkdown')}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('command.exportMarkdown')}
    >
      Markdown
    </button>
    <button
      onclick={handleExportPdf}
      disabled={exporting}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-50 transition-colors"
      title={$t('pdf.exportPage')}
    >
      {#if exporting}{$t('pdf.exporting')}{:else}📄 {$t('pdf.exportPage')}{/if}
    </button>
    <button
      onclick={handleRecordAudio}
      class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={$t('audio.record')}
    >
      {#if recording}⏹ {$t('audio.recording')} {Math.floor(recordDuration)}s{:else}🎙 {$t('audio.record')}{/if}
    </button>
    {#if attachmentStore.attachments.some((a) => a.isImage)}
      {#each attachmentStore.attachments.filter((a) => a.isImage) as att (att.id)}
        <button
          onclick={() => handleOcrAttachment(att.id)}
          disabled={ocrLoading === att.id}
          class="text-xs px-2.5 py-1 rounded-md text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-50 transition-colors"
          title={$t('ocr.extractText')}
        >
          {#if ocrLoading === att.id}{$t('ocr.extracting')}{:else}🔍 {$t('ocr.extractText')}{/if}
        </button>
      {/each}
    {/if}
  </div>

  <!-- Page Header -->
  <div class="px-12 pt-8 pb-2 max-w-3xl mx-auto w-full">
    <div class="flex items-center gap-3">
      <input
        type="text"
        bind:value={iconValue}
        onblur={saveIcon}
        placeholder="📄"
        aria-label="{$t('editor.icon')}"
        class="w-10 text-2xl bg-transparent text-center focus:outline-none"
        maxlength="2"
      />
      <input
        type="text"
        bind:value={titleValue}
        onblur={saveTitle}
        placeholder={$t('editor.titlePlaceholder')}
        aria-label="{$t('editor.titlePlaceholder')}"
        class="flex-1 text-3xl font-bold bg-transparent focus:outline-none placeholder:text-gray-300 dark:placeholder:text-gray-700 text-gray-900 dark:text-gray-100"
      />
    </div>

    <!-- Tags -->
    <div class="mt-3">
      <TagInput pageId={page.id} />
    </div>

    <!-- Properties -->
    <div class="mt-3 pt-3 border-t border-gray-100 dark:border-gray-800/50">
      <div class="text-xs font-semibold text-gray-400 dark:text-gray-600 uppercase tracking-wide mb-2">{$t('properties.title')}</div>
      <PageProperties pageId={page.id} />
    </div>
  </div>

  <!-- Editor -->
  <div class="flex-1 overflow-y-auto px-12 pb-16 max-w-3xl mx-auto w-full text-gray-900 dark:text-gray-100" onkeydown={handleKeydown} role="toolbar" aria-label="{$t('editor.titlePlaceholder')}" tabindex="0">
    <div bind:this={editorElement} class="prose-container"></div>
  </div>
</div>

{#if showSlashMenu}
  <SlashMenu
    x={slashMenuPos.x}
    y={slashMenuPos.y}
    onSelect={handleSlashSelect}
    onClose={() => (showSlashMenu = false)}
  />
{/if}

{#if showWikiAutocomplete}
  <WikiLinkAutocomplete
    titles={pageTitles}
    query={wikiQuery}
    onSelect={handleWikiLinkSelect}
    onClose={() => (showWikiAutocomplete = false)}
  />
{/if}

{#if toast}
  <div
    class="fixed bottom-4 right-4 z-50 px-4 py-2 rounded-lg shadow-lg text-sm {toast.type === 'success' ? 'bg-green-600 text-white' : 'bg-red-600 text-white'}"
  >
    {toast.msg}
  </div>
{/if}
