<script lang="ts">
  import { onMount } from 'svelte'
  import { tagStore } from '../stores/app.svelte'
  import { t } from '../i18n'

  type CaptureMode = 'note' | 'web'
  type CaptureInput = {
    title: string
    body: string
    tags: string[]
    useInbox: boolean
    sourceUrl?: string
  }

  let {
    initialMode = 'note',
    onClose,
    onCapture,
  }: {
    initialMode?: CaptureMode
    onClose: () => void
    onCapture: (input: CaptureInput) => Promise<void>
  } = $props()

  let captureMode = $state<CaptureMode>('note')
  let title = $state('')
  let sourceUrl = $state('')
  let body = $state('')
  let tags = $state('')
  let useInbox = $state(true)
  let isSaving = $state(false)
  let error = $state<string | null>(null)
  let titleInput = $state<HTMLInputElement>()
  let sourceInput = $state<HTMLInputElement>()

  let isWebCapture = $derived(captureMode === 'web')
  let normalizedSourceUrl = $derived(isWebCapture ? normalizeSourceUrl(sourceUrl) : '')
  let canSubmit = $derived(
    isWebCapture
      ? Boolean(normalizedSourceUrl)
      : Boolean(title.trim() || body.trim()),
  )

  onMount(() => {
    captureMode = initialMode
    if (initialMode === 'web' && !tags.trim()) {
      tags = 'web'
    }
    setTimeout(() => {
      if (initialMode === 'web') {
        sourceInput?.focus()
      } else {
        titleInput?.focus()
      }
    }, 0)
  })

  function normalizeSourceUrl(value: string): string {
    const raw = value.trim()
    if (!raw) return ''
    const withScheme = raw.includes('://') ? raw : `https://${raw}`
    try {
      const url = new URL(withScheme)
      if (url.protocol !== 'http:' && url.protocol !== 'https:') return ''
      return url.toString()
    } catch {
      return ''
    }
  }

  function titleFromUrl(url: string): string {
    try {
      const parsed = new URL(url)
      return parsed.hostname || $t('quickCapture.webTitleFallback')
    } catch {
      return $t('quickCapture.webTitleFallback')
    }
  }

  function setMode(mode: CaptureMode) {
    captureMode = mode
    if (mode === 'web' && !tags.trim()) {
      tags = 'web'
    }
    setTimeout(() => {
      if (mode === 'web') {
        sourceInput?.focus()
      } else {
        titleInput?.focus()
      }
    }, 0)
  }

  function tagList(): string[] {
    return tags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)
  }

  async function submit() {
    if (!canSubmit) return
    isSaving = true
    error = null
    try {
      const source = isWebCapture ? normalizedSourceUrl : undefined
      await onCapture({
        title: title.trim()
          || (source ? titleFromUrl(source) : '')
          || body.trim().split('\n')[0]?.slice(0, 80)
          || $t('command.quickCapture'),
        body: body.trim(),
        tags: tagList(),
        useInbox,
        sourceUrl: source,
      })
      onClose()
    } catch (e) {
      error = String(e)
    } finally {
      isSaving = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      submit()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 bg-black/30 flex items-start justify-center pt-[12vh]"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose() }}
  role="presentation"
>
  <div
    class="w-full max-w-xl overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 shadow-2xl"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label={isWebCapture ? $t('quickCapture.webMode') : $t('command.quickCapture')}
    tabindex="-1"
  >
    <div class="border-b border-gray-200 dark:border-gray-800 px-4 py-3">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {isWebCapture ? $t('quickCapture.webMode') : $t('command.quickCapture')}
          </h2>
          <p class="mt-0.5 text-xs text-gray-500 dark:text-gray-400">
            {isWebCapture ? $t('quickCapture.webDescription') : $t('quickCapture.description')}
          </p>
        </div>
        <div class="grid grid-cols-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 p-0.5 text-xs">
          <button
            type="button"
            onclick={() => setMode('note')}
            class="rounded-md px-2.5 py-1.5 {captureMode === 'note' ? 'bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 shadow-sm' : 'text-gray-500 dark:text-gray-400'}"
          >
            {$t('quickCapture.noteMode')}
          </button>
          <button
            type="button"
            onclick={() => setMode('web')}
            class="rounded-md px-2.5 py-1.5 {captureMode === 'web' ? 'bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 shadow-sm' : 'text-gray-500 dark:text-gray-400'}"
          >
            {$t('quickCapture.webMode')}
          </button>
        </div>
      </div>
    </div>

    <div class="space-y-3 p-4">
      {#if isWebCapture}
        <input
          bind:this={sourceInput}
          bind:value={sourceUrl}
          type="url"
          placeholder={$t('quickCapture.sourceUrlPlaceholder')}
          class="w-full rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
        />
      {/if}
      <input
        bind:this={titleInput}
        bind:value={title}
        type="text"
        placeholder={isWebCapture ? $t('quickCapture.webTitlePlaceholder') : $t('common.title')}
        class="w-full rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
      />
      <textarea
        bind:value={body}
        rows="7"
        placeholder={isWebCapture ? $t('quickCapture.webBodyPlaceholder') : $t('quickCapture.bodyPlaceholder')}
        class="w-full resize-none rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
      ></textarea>
      <input
        bind:value={tags}
        type="text"
        placeholder={$t('quickCapture.tagsPlaceholder')}
        class="w-full rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
      />

      <div class="flex items-center justify-between gap-3">
        <label class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
          <input type="checkbox" bind:checked={useInbox} class="rounded border-gray-300 dark:border-gray-700" />
          {$t('quickCapture.addUnderInbox')}
        </label>
        <div class="text-xs text-gray-400">
          {$t('quickCapture.tagsAvailable', { count: tagStore.tags.length })}
        </div>
      </div>

      {#if error}
        <div class="rounded-lg bg-red-50 dark:bg-red-900/20 px-3 py-2 text-sm text-red-600 dark:text-red-400">{error}</div>
      {/if}
    </div>

    <div class="flex items-center justify-between border-t border-gray-200 dark:border-gray-800 px-4 py-3">
      <kbd class="hidden sm:inline-flex text-[10px] text-gray-400 border border-gray-300 dark:border-gray-700 rounded px-1.5 py-0.5">Cmd+Enter</kbd>
      <div class="flex items-center gap-2 ml-auto">
        <button
          type="button"
          onclick={onClose}
          class="px-3 py-1.5 rounded-md text-sm border border-gray-300 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800"
        >
          {$t('common.cancel')}
        </button>
        <button
          type="button"
          onclick={submit}
          disabled={isSaving || !canSubmit}
          class="px-3 py-1.5 rounded-md text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
        >
          {isSaving ? $t('quickCapture.saving') : (isWebCapture ? $t('quickCapture.captureWeb') : $t('quickCapture.capture'))}
        </button>
      </div>
    </div>
  </div>
</div>
