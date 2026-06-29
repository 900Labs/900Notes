<script lang="ts">
  import { t } from '../../i18n'

  let { onAction }: { onAction: (action: string) => void } = $props()
</script>

<div class="flex items-center gap-0.5 px-4 py-2 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
  <!-- Inline formatting -->
  <button onclick={() => onAction('bold')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 font-bold text-sm text-gray-700 dark:text-gray-300 transition-colors" title="Bold (Ctrl+B)">
    B
  </button>
  <button onclick={() => onAction('italic')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 italic text-sm text-gray-700 dark:text-gray-300 transition-colors" title="Italic (Ctrl+I)">
    I
  </button>
  <button onclick={() => onAction('underline')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 underline text-sm text-gray-700 dark:text-gray-300 transition-colors" title="Underline (Ctrl+U)">
    U
  </button>
  <button onclick={() => onAction('strike')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 line-through text-sm text-gray-700 dark:text-gray-300 transition-colors" title="Strikethrough">
    S
  </button>
  <button onclick={() => onAction('code')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 font-mono text-sm text-gray-700 dark:text-gray-300 transition-colors" title="Inline code">
    {'</>'}
  </button>

  <div class="w-px h-5 bg-gray-200 dark:bg-gray-700 mx-1.5"></div>

  <!-- Block types -->
  <button onclick={() => onAction('paragraph')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.paragraph')}>
    ¶
  </button>
  <button onclick={() => onAction('h1')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm font-bold text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.heading1')}>
    H1
  </button>
  <button onclick={() => onAction('h2')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm font-semibold text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.heading2')}>
    H2
  </button>
  <button onclick={() => onAction('h3')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm font-medium text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.heading3')}>
    H3
  </button>

  <div class="w-px h-5 bg-gray-200 dark:bg-gray-700 mx-1.5"></div>

  <!-- Lists -->
  <button onclick={() => onAction('bulletList')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.bulletList')}>
    •
  </button>
  <button onclick={() => onAction('orderedList')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.orderedList')}>
    1.
  </button>
  <button onclick={() => onAction('todo')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.todo')}>
    ☐
  </button>

  <div class="w-px h-5 bg-gray-200 dark:bg-gray-700 mx-1.5"></div>

  <!-- Other blocks -->
  <button onclick={() => onAction('quote')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.quote')}>
    "
  </button>
  <button onclick={() => onAction('codeBlock')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm font-mono text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.code')}>
    {'{ }'}
  </button>
  <button onclick={() => onAction('divider')} class="px-2.5 py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 transition-colors" title={$t('slash.divider')}>
    ―
  </button>
</div>
