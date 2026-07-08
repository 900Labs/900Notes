<script lang="ts">
  import { t } from '../../i18n'
  import { schema } from '../../lib/editor/schema'

  interface HeadingItem {
    level: number
    text: string
    pos: number
  }

  let {
    content,
    onHeadingClick,
  }: {
    content: string
    onHeadingClick: (pos: number) => void
  } = $props()

  let headings = $derived<HeadingItem[]>(extractHeadings(content))

  function extractHeadings(contentStr: string): HeadingItem[] {
    try {
      const doc = schema.nodeFromJSON(JSON.parse(contentStr))
      const items: HeadingItem[] = []
      doc.descendants((node, pos) => {
        if (node.type.name === 'heading') {
          const level = Number(node.attrs.level) || 1
          items.push({ level, text: node.textContent, pos })
        }
      })
      return items
    } catch {
      return []
    }
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-900">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-800">
    <h3 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">{$t('outline.title')}</h3>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if headings.length === 0}
      <p class="text-xs text-gray-400 dark:text-gray-600 px-2 py-4">{$t('outline.empty')}</p>
    {:else}
      <div class="space-y-0.5">
        {#each headings as heading, i (i)}
          <button
            onclick={() => onHeadingClick(heading.pos)}
            class="w-full text-left py-1 px-2 rounded-md text-sm hover:bg-gray-100 dark:hover:bg-gray-800 truncate text-gray-700 dark:text-gray-300 transition-colors"
            style="padding-left: {heading.level * 12 + 8}px"
            title={heading.text}
          >
            {heading.text || 'Untitled'}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
