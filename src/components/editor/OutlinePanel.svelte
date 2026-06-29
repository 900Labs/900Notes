<script lang="ts">
  import { t } from '../../i18n'

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
      const doc = JSON.parse(contentStr)
      if (!doc.content) return []
      const items: HeadingItem[] = []
      let pos = 0
      for (const node of doc.content) {
        if (node.type === 'heading' && node.attrs) {
          const level = node.attrs.level || 1
          let text = ''
          if (node.content) {
            for (const child of node.content) {
              if (child.type === 'text') {
                text += child.text || ''
              }
            }
          }
          items.push({ level, text, pos })
        }
        pos += 1
      }
      return items
    } catch {
      return []
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="px-3 py-2.5 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('outline.title')}</h3>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if headings.length === 0}
      <p class="text-xs text-gray-400 px-2 py-4">{$t('outline.empty')}</p>
    {:else}
      <div class="space-y-0.5">
        {#each headings as heading, i (i)}
          <button
            onclick={() => onHeadingClick(heading.pos)}
            class="w-full text-left py-1 px-2 rounded text-sm hover:bg-gray-200 dark:hover:bg-gray-700 truncate"
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
