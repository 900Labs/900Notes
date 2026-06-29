// Callout Block Plugin
// Registers a custom block type for highlighting important notes

plugin.registerBlock({
  nodeType: 'callout',
  group: 'block',
  content: 'inline*',
  attrs: { variant: { default: 'info' } },
  toDom: "(node) => ['div', { class: 'callout callout-' + node.attrs.variant, 'data-variant': node.attrs.variant }, 0]",
  parseDom: "[{ tag: 'div.callout', getAttrs: (dom) => ({ variant: dom.getAttribute('data-variant') || 'info' }) }]",
  icon: '💡',
  label: 'Callout',
})

plugin.registerCommand('insertCallout', 'Insert Callout', () => {
  console.log('Insert callout block')
})

plugin.registerHook('pageSave', (pageId) => {
  console.log('Page saved:', pageId)
})
