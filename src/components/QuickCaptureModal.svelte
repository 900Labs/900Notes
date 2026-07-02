<script lang="ts">
  import { onMount } from 'svelte'
  import { tagStore } from '../stores/app.svelte'
  import { t } from '../i18n'

  let {
    onClose,
    onCapture,
  }: {
    onClose: () => void
    onCapture: (input: { title: string; body: string; tags: string[]; useInbox: boolean }) => Promise<void>
  } = $props()

  let title = $state('')
  let body = $state('')
  let tags = $state('')
  let useInbox = $state(true)
  let isSaving = $state(false)
  let error = $state<string | null>(null)
  let titleInput: HTMLInputElement

  onMount(() => {
    setTimeout(() => titleInput?.focus(), 0)
  })

  function tagList(): string[] {
    return tags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)
  }

  async function submit() {
    if (!title.trim() && !body.trim()) return
    isSaving = true
    error = null
    try {
      await onCapture({
        title: title.trim() || body.trim().split('\n')[0]?.slice(0, 80) || $t('command.quickCapture'),
        body: body.trim(),
        tags: tagList(),
        useInbox,
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
    aria-label={$t('command.quickCapture')}
    tabindex="-1"
  >
    <div class="border-b border-gray-200 dark:border-gray-800 px-4 py-3">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">{$t('command.quickCapture')}</h2>
      <p class="mt-0.5 text-xs text-gray-500 dark:text-gray-400">{$t('quickCapture.description')}</p>
    </div>

    <div class="space-y-3 p-4">
      <input
        bind:this={titleInput}
        bind:value={title}
        type="text"
        placeholder={$t('common.title')}
        class="w-full rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
      />
      <textarea
        bind:value={body}
        rows="7"
        placeholder={$t('quickCapture.bodyPlaceholder')}
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
          disabled={isSaving || (!title.trim() && !body.trim())}
          class="px-3 py-1.5 rounded-md text-sm font-medium bg-accent text-white hover:bg-accent-dark disabled:opacity-50"
        >
          {isSaving ? $t('quickCapture.saving') : $t('quickCapture.capture')}
        </button>
      </div>
    </div>
  </div>
</div>
