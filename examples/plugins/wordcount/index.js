// Word Count Plugin
// Registers a command that counts words in the current page content

plugin.registerCommand('wordCount', 'Word Count', () => {
  const editor = document.querySelector('.ProseMirror')
  if (!editor) {
    alert('Editor not found')
    return
  }
  const text = editor.textContent || ''
  const words = text.trim().split(/\s+/).filter(Boolean).length
  const chars = text.length
  alert(`Words: ${words}\nCharacters: ${chars}`)
})

plugin.registerHook('pageSave', (pageId) => {
  const editor = document.querySelector('.ProseMirror')
  if (editor) {
    const text = editor.textContent || ''
    const words = text.trim().split(/\s+/).filter(Boolean).length
    console.log(`[wordcount] Page ${pageId}: ${words} words`)
  }
})
