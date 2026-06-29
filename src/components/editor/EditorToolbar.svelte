<script lang="ts">
  import { t } from '../../i18n'

  let { onAction }: { onAction: (action: string) => void } = $props()
</script>

<div class="flex items-center gap-0.5 px-4 py-1.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
  <!-- Inline formatting -->
  <button onclick={() => onAction('bold')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 font-bold text-sm" title="Bold (Ctrl+B)">
    B
  </button>
  <button onclick={() => onAction('italic')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 italic text-sm" title="Italic (Ctrl+I)">
    I
  </button>
  <button onclick={() => onAction('underline')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 underline text-sm" title="Underline (Ctrl+U)">
    U
  </button>
  <button onclick={() => onAction('strike')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 line-through text-sm" title="Strikethrough">
    S
  </button>
  <button onclick={() => onAction('code')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 font-mono text-sm" title="Inline code">
    {'</>'}
  </button>

  <div class="w-px h-5 bg-gray-300 dark:bg-gray-600 mx-1"></div>

  <!-- Block types -->
  <button onclick={() => onAction('paragraph')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.paragraph')}>
    ¶
  </button>
  <button onclick={() => onAction('h1')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm font-bold" title={$t('slash.heading1')}>
    H1
  </button>
  <button onclick={() => onAction('h2')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm font-semibold" title={$t('slash.heading2')}>
    H2
  </button>
  <button onclick={() => onAction('h3')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm font-medium" title={$t('slash.heading3')}>
    H3
  </button>

  <div class="w-px h-5 bg-gray-300 dark:bg-gray-600 mx-1"></div>

  <!-- Lists -->
  <button onclick={() => onAction('bulletList')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.bulletList')}>
    •
  </button>
  <button onclick={() => onAction('orderedList')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.orderedList')}>
    1.
  </button>
  <button onclick={() => onAction('todo')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.todo')}>
    ☐
  </button>

  <div class="w-px h-5 bg-gray-300 dark:bg-gray-600 mx-1"></div>

  <!-- Other blocks -->
  <button onclick={() => onAction('quote')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.quote')}>
    "
  </button>
  <button onclick={() => onAction('codeBlock')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm font-mono" title={$t('slash.code')}>
    {'{ }'}
  </button>
  <button onclick={() => onAction('divider')} class="px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-sm" title={$t('slash.divider')}>
    —
  </button>
</div>
