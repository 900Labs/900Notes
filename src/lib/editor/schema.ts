import { Schema } from 'prosemirror-model'

export const schema = new Schema({
  nodes: {
    doc: {
      content: 'block+',
    },
    paragraph: {
      content: 'inline*',
      group: 'block',
      parseDOM: [{ tag: 'p' }],
      toDOM: () => ['p', 0],
    },
    heading: {
      attrs: { level: { default: 1, validate: 'number' } },
      content: 'inline*',
      group: 'block',
      defining: true,
      parseDOM: [1, 2, 3].map((level) => ({
        tag: `h${level}`,
        attrs: { level },
      })),
      toDOM: (node) => [`h${node.attrs.level}`, 0],
    },
    bullet_list: {
      content: 'list_item+',
      group: 'block',
      parseDOM: [{ tag: 'ul' }],
      toDOM: () => ['ul', 0],
    },
    ordered_list: {
      content: 'list_item+',
      group: 'block',
      parseDOM: [{ tag: 'ol' }],
      toDOM: () => ['ol', 0],
    },
    list_item: {
      content: 'paragraph block*',
      parseDOM: [{ tag: 'li' }],
      toDOM: () => ['li', 0],
      defining: true,
    },
    todo_item: {
      attrs: { checked: { default: false, validate: 'boolean' } },
      content: 'inline*',
      group: 'block',
      defining: true,
      parseDOM: [
        {
          tag: 'div[data-type="todo"]',
          getAttrs: (dom: HTMLElement) => ({
            checked: dom.getAttribute('data-checked') === 'true',
          }),
        },
      ],
      toDOM: (node) => [
        'div',
        {
          'data-type': 'todo',
          'data-checked': node.attrs.checked,
          class: node.attrs.checked ? 'todo-item checked' : 'todo-item',
        },
        0,
      ],
    },
    code_block: {
      content: 'text*',
      group: 'block',
      code: true,
      parseDOM: [{ tag: 'pre', preserveWhitespace: 'full' }],
      toDOM: () => ['pre', ['code', 0]],
    },
    blockquote: {
      content: 'block+',
      group: 'block',
      defining: true,
      parseDOM: [{ tag: 'blockquote' }],
      toDOM: () => ['blockquote', 0],
    },
    divider: {
      group: 'block',
      parseDOM: [{ tag: 'hr' }],
      toDOM: () => ['hr'],
    },
    image: {
      inline: false,
      group: 'block',
      draggable: true,
      attrs: {
        src: { default: '' },
        alt: { default: '' },
        title: { default: '' },
        attachmentId: { default: null },
      },
      parseDOM: [
        {
          tag: 'img[src]',
          getAttrs: (dom: HTMLElement) => ({
            src: dom.getAttribute('src') || '',
            alt: dom.getAttribute('alt') || '',
            title: dom.getAttribute('title') || '',
            attachmentId: dom.getAttribute('data-attachment-id') || null,
          }),
        },
      ],
      toDOM: (node) => [
        'img',
        {
          src: node.attrs.src,
          alt: node.attrs.alt,
          title: node.attrs.title,
          'data-attachment-id': node.attrs.attachmentId,
          class: 'pm-image',
        },
      ],
    },
    table: {
      content: 'table_row+',
      group: 'block',
      parseDOM: [{ tag: 'table' }],
      toDOM: () => ['table', ['tbody', 0]],
    },
    table_row: {
      content: 'table_cell+',
      parseDOM: [{ tag: 'tr' }],
      toDOM: () => ['tr', 0],
    },
    table_cell: {
      content: 'block+',
      parseDOM: [{ tag: 'td' }],
      toDOM: () => ['td', 0],
    },
    text: {
      group: 'inline',
    },
    wiki_link: {
      inline: true,
      group: 'inline',
      attrs: { title: { default: '' }, pageId: { default: null } },
      defining: true,
      parseDOM: [
        {
          tag: 'span[data-type="wiki-link"]',
          getAttrs: (dom: HTMLElement) => ({
            title: dom.getAttribute('data-title') || '',
            pageId: dom.getAttribute('data-page-id') || null,
          }),
        },
      ],
      toDOM: (node) => [
        'span',
        {
          'data-type': 'wiki-link',
          'data-title': node.attrs.title,
          'data-page-id': node.attrs.pageId,
          class: 'wiki-link',
        },
        node.attrs.title,
      ],
    },
    hard_break: {
      inline: true,
      group: 'inline',
      selectable: false,
      parseDOM: [{ tag: 'br' }],
      toDOM: () => ['br'],
    },
    math_inline: {
      inline: true,
      group: 'inline',
      atom: true,
      attrs: { latex: { default: '' } },
      defining: true,
      parseDOM: [
        {
          tag: 'span[data-type="math-inline"]',
          getAttrs: (dom: HTMLElement) => ({
            latex: dom.textContent || '',
          }),
        },
      ],
      toDOM: (node) => [
        'span',
        {
          'data-type': 'math-inline',
          class: 'pm-math-inline',
        },
        node.attrs.latex,
      ],
    },
    math_block: {
      inline: false,
      group: 'block',
      atom: true,
      attrs: { latex: { default: '' } },
      defining: true,
      parseDOM: [
        {
          tag: 'div[data-type="math-block"]',
          getAttrs: (dom: HTMLElement) => ({
            latex: dom.textContent || '',
          }),
        },
      ],
      toDOM: (node) => [
        'div',
        {
          'data-type': 'math-block',
          class: 'pm-math-block',
        },
        node.attrs.latex,
      ],
    },
    mermaid_block: {
      inline: false,
      group: 'block',
      content: 'text*',
      code: true,
      defining: true,
      attrs: { rendered: { default: false } },
      parseDOM: [
        {
          tag: 'div[data-type="mermaid"]',
          preserveWhitespace: 'full',
        },
      ],
      toDOM: () => [
        'div',
        {
          'data-type': 'mermaid',
          class: 'pm-mermaid',
        },
        ['pre', 0],
      ],
    },
    audio_block: {
      inline: false,
      group: 'block',
      atom: true,
      defining: true,
      attrs: {
        attachmentId: { default: '' },
        audioNoteId: { default: '' },
        duration: { default: 0 },
        title: { default: '' },
        transcription: { default: '' },
      },
      parseDOM: [
        {
          tag: 'div[data-type="audio-block"]',
          getAttrs: (dom: HTMLElement) => ({
            attachmentId: dom.getAttribute('data-attachment-id') || '',
            audioNoteId: dom.getAttribute('data-audio-note-id') || '',
            duration: parseFloat(dom.getAttribute('data-duration') || '0'),
            title: dom.getAttribute('data-title') || '',
            transcription: dom.getAttribute('data-transcription') || '',
          }),
        },
      ],
      toDOM: (node) => [
        'div',
        {
          'data-type': 'audio-block',
          class: 'pm-audio-block',
          'data-attachment-id': node.attrs.attachmentId,
          'data-audio-note-id': node.attrs.audioNoteId,
          'data-duration': String(node.attrs.duration),
          'data-title': node.attrs.title,
          'data-transcription': node.attrs.transcription,
        },
      ],
    },
  },
  marks: {
    bold: {
      parseDOM: [{ tag: 'strong' }, { tag: 'b' }, { style: 'font-weight=bold' }],
      toDOM: () => ['strong', 0],
    },
    italic: {
      parseDOM: [{ tag: 'em' }, { tag: 'i' }, { style: 'font-style=italic' }],
      toDOM: () => ['em', 0],
    },
    underline: {
      parseDOM: [{ tag: 'u' }, { style: 'text-decoration=underline' }],
      toDOM: () => ['u', 0],
    },
    strike: {
      parseDOM: [{ tag: 's' }, { tag: 'del' }, { style: 'text-decoration=line-through' }],
      toDOM: () => ['s', 0],
    },
    code: {
      parseDOM: [{ tag: 'code' }],
      toDOM: () => ['code', 0],
    },
    link: {
      attrs: { href: { default: '' } },
      inclusive: false,
      parseDOM: [
        {
          tag: 'a[href]',
          getAttrs: (dom: HTMLElement) => ({ href: dom.getAttribute('href') || '' }),
        },
      ],
      toDOM: (node) => ['a', { href: node.attrs.href, class: 'pm-link' }, 0],
    },
  },
})
