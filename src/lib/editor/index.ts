import { EditorState, Plugin, PluginKey } from 'prosemirror-state'
import { EditorView } from 'prosemirror-view'
import { keymap } from 'prosemirror-keymap'
import { history, undo, redo } from 'prosemirror-history'
import { baseKeymap, toggleMark, setBlockType, wrapIn } from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { InputRule, inputRules } from 'prosemirror-inputrules'
import { schema } from './schema'
import { createNodeViews } from './nodeviews'
import * as api from '../api'

const MAX_ATTACHMENT_BYTES = 25 * 1024 * 1024

export interface EditorCallbacks {
  onChange: (content: string) => void
  onWikiLinkClick: (title: string) => void
  getPageTitles: () => { id: string; title: string }[]
  getPageId?: () => string | null
}

export function createEditor(
  element: HTMLElement,
  initialContent: string,
  callbacks: EditorCallbacks,
): EditorView {
  let doc
  try {
    doc = schema.nodeFromJSON(JSON.parse(initialContent))
  } catch {
    doc = schema.nodes.doc.create(null, [schema.nodes.paragraph.create()])
  }

  const state = EditorState.create({
    doc,
    plugins: [
      buildKeymaps(),
      buildInputRules(),
      history(),
      dropCursor(),
      keymap(baseKeymap),
      buildWikiLinkPlugin(callbacks),
      buildImagePastePlugin(callbacks),
      buildInlineMathPlugin(),
    ],
  })

  const view = new EditorView(element, {
    state,
    nodeViews: createNodeViews(),
    dispatchTransaction(transaction) {
      const newState = view.state.apply(transaction)
      view.updateState(newState)
      if (transaction.docChanged) {
        callbacks.onChange(JSON.stringify(view.state.doc.toJSON()))
      }
      const active = activeWikiLinkQuery(view)
      if (active) {
        window.dispatchEvent(new CustomEvent('wiki-link-query', { detail: active }))
      } else {
        window.dispatchEvent(new CustomEvent('wiki-link-close'))
      }
    },
    handleClickOn(view, _pos, node) {
      if (node.type.name === 'wiki_link') {
        callbacks.onWikiLinkClick(node.attrs.title)
        return true
      }
      return false
    },
  })

  return view
}

function buildKeymaps() {
  return keymap({
    'Mod-z': undo,
    'Mod-y': redo,
    'Mod-Shift-z': redo,
    'Mod-b': toggleMark(schema.marks.bold),
    'Mod-i': toggleMark(schema.marks.italic),
    'Mod-u': toggleMark(schema.marks.underline),
    'Mod-Shift-x': toggleMark(schema.marks.strike),
    'Mod-`': toggleMark(schema.marks.code),
  })
}

function buildInputRules() {
  return inputRules({
    rules: [
      // Heading shortcuts: # ## ###
      headingRule(1),
      headingRule(2),
      headingRule(3),
      // Bullet list: - or * at start
      new InputRule(/^[-*]\s$/, (state, _match, start, end) => {
        const tr = state.tr.delete(start, end)
        const $start = tr.doc.resolve(start)
        const range = $start.blockRange()
        if (range) {
          tr.wrap(range, [schema.nodes.bullet_list.create()])
        }
        return tr
      }),
      // Ordered list: 1. at start
      new InputRule(/^1\.\s$/, (state, _match, start, end) => {
        const tr = state.tr.delete(start, end)
        const $start = tr.doc.resolve(start)
        const range = $start.blockRange()
        if (range) {
          tr.wrap(range, [schema.nodes.ordered_list.create()])
        }
        return tr
      }),
      // Todo item: [] at start
      new InputRule(/^\[\]\s$/, (state, _match, start, end) => {
        return state.tr
          .delete(start, end)
          .setBlockType(start, start, schema.nodes.todo_item, { checked: false })
      }),
      // Code block: ```
      new InputRule(/^```$/, (state, _match, start, end) => {
        return state.tr
          .delete(start, end)
          .setBlockType(start, start, schema.nodes.code_block)
      }),
      // Blockquote: >
      new InputRule(/^>\s$/, (state, _match, start, end) => {
        const tr = state.tr.delete(start, end)
        const $start = tr.doc.resolve(start)
        const range = $start.blockRange()
        if (range) {
          tr.wrap(range, [schema.nodes.blockquote.create()])
        }
        return tr
      }),
      // Divider: ---
      new InputRule(/^---$/, (state, _match, start, end) => {
        const tr = state.tr.delete(start, end)
        tr.insert(start, schema.nodes.divider.create())
        return tr
      }),
      // Math block: $$
      new InputRule(/^\$\$$/, (state, _match, start, end) => {
        return state.tr
          .delete(start, end)
          .replaceSelectionWith(schema.nodes.math_block.create({ latex: '' }))
      }),
      // Mermaid block: ~~~
      new InputRule(/^~~~$/, (state, _match, start, end) => {
        return state.tr
          .delete(start, end)
          .setBlockType(start, start, schema.nodes.mermaid_block)
      }),
    ],
  })
}

function headingRule(level: number) {
  const pattern = new RegExp(`^${'#'.repeat(level)}\\s$`)
  return new InputRule(pattern, (state, _match, start, end) => {
    return state.tr
      .delete(start, end)
      .setBlockType(start, start, schema.nodes.heading, { level })
  })
}

function buildInlineMathPlugin(): Plugin {
  return new Plugin({
    key: new PluginKey('inlineMath'),
    props: {
      handleTextInput(view: EditorView, from: number, to: number, text: string): boolean {
        if (text !== '$') return false
        const $from = view.state.doc.resolve(from)
        const textBefore = view.state.doc.textBetween($from.start(), from, '')
        const lastDollar = textBefore.lastIndexOf('$')
        if (lastDollar < 0) return false
        const mathStart = $from.start() + lastDollar + 1
        const latex = textBefore.slice(lastDollar + 1)
        if (!latex) return false
        const tr = view.state.tr
          .delete(mathStart - 1, to)
          .replaceRangeWith(mathStart - 1, mathStart - 1, schema.nodes.math_inline.create({ latex }))
        view.dispatch(tr)
        return true
      },
    },
  })
}

function buildWikiLinkPlugin(callbacks: EditorCallbacks): Plugin {
  return new Plugin({
    key: new PluginKey('wikiLink'),
    props: {
      handleTextInput(view: EditorView, from: number, to: number, text: string): boolean {
        if (text === '[') {
          const textBefore = view.state.doc.textBetween(from - 1, from, '')
          if (textBefore === '[') {
            showWikiLinkAutocomplete(view, from, callbacks)
          }
        }
        return false
      },
    },
  })
}

function showWikiLinkAutocomplete(
  _view: EditorView,
  pos: number,
  callbacks: EditorCallbacks,
) {
  // This will be handled by the Svelte component's autocomplete overlay
  // The component listens for [[ and shows a dropdown
  const titles = callbacks.getPageTitles()
  // Dispatch a custom event that the Svelte component listens for
  window.dispatchEvent(
    new CustomEvent('wiki-link-start', { detail: { titles, from: pos - 1 } }),
  )
}

export function activeWikiLinkQuery(view: EditorView): { from: number; query: string } | null {
  const { $from } = view.state.selection
  if (!$from.parent.isTextblock) return null
  const before = $from.parent.textBetween(0, $from.parentOffset, '')
  return wikiLinkQueryFromText(before, $from.start())
}

export function wikiLinkQueryFromText(
  beforeCursor: string,
  textBlockStart = 0,
): { from: number; query: string } | null {
  const trigger = beforeCursor.lastIndexOf('[[')
  if (trigger < 0) return null
  const query = beforeCursor.slice(trigger + 2)
  if (query.includes(']]') || query.includes('\n')) return null
  return { from: textBlockStart + trigger, query }
}

export function insertWikiLink(
  view: EditorView,
  title: string,
  pageId: string | null,
  triggerFrom: number | null = null,
) {
  const { state } = view
  const { selection } = state
  const wikiLinkNode = schema.nodes.wiki_link.create({ title, pageId })
  const tr = triggerFrom === null
    ? state.tr.replaceSelectionWith(wikiLinkNode, false)
    : state.tr.replaceRangeWith(triggerFrom, selection.from, wikiLinkNode)
  view.dispatch(tr)
  view.focus()
}

export function setHeading(view: EditorView, level: number) {
  setBlockType(schema.nodes.heading, { level })(view.state, view.dispatch)
  view.focus()
}

export function setParagraph(view: EditorView) {
  setBlockType(schema.nodes.paragraph)(view.state, view.dispatch)
  view.focus()
}

export function toggleBulletList(view: EditorView) {
  wrapIn(schema.nodes.bullet_list)(view.state, view.dispatch)
  view.focus()
}

export function toggleOrderedList(view: EditorView) {
  wrapIn(schema.nodes.ordered_list)(view.state, view.dispatch)
  view.focus()
}

export function toggleBlockquote(view: EditorView) {
  wrapIn(schema.nodes.blockquote)(view.state, view.dispatch)
  view.focus()
}

export function setCodeBlock(view: EditorView) {
  setBlockType(schema.nodes.code_block)(view.state, view.dispatch)
  view.focus()
}

export function insertDivider(view: EditorView) {
  const { state } = view
  const tr = state.tr.replaceSelectionWith(schema.nodes.divider.create())
  view.dispatch(tr)
  view.focus()
}

export function insertTodoItem(view: EditorView) {
  setBlockType(schema.nodes.todo_item, { checked: false })(view.state, view.dispatch)
  view.focus()
}

export function toggleBold(view: EditorView) {
  toggleMark(schema.marks.bold)(view.state, view.dispatch)
  view.focus()
}

export function toggleItalic(view: EditorView) {
  toggleMark(schema.marks.italic)(view.state, view.dispatch)
  view.focus()
}

export function toggleUnderline(view: EditorView) {
  toggleMark(schema.marks.underline)(view.state, view.dispatch)
  view.focus()
}

export function toggleStrike(view: EditorView) {
  toggleMark(schema.marks.strike)(view.state, view.dispatch)
  view.focus()
}

export function toggleCode(view: EditorView) {
  toggleMark(schema.marks.code)(view.state, view.dispatch)
  view.focus()
}

export function destroyEditor(view: EditorView) {
  view.destroy()
}

function buildImagePastePlugin(callbacks: EditorCallbacks): Plugin {
  return new Plugin({
    key: new PluginKey('imagePaste'),
    props: {
      handleDrop(view, event) {
        const dragEvent = event as DragEvent
        const files = dragEvent.dataTransfer?.files
        if (!files || files.length === 0) return false

        let handled = false
        for (const file of Array.from(files)) {
          if (file.type.startsWith('image/')) {
            handled = true
            insertImageFromFile(view, file, callbacks).catch((error) => {
              console.error('Image attachment failed:', error)
            })
          }
        }
        if (handled) {
          event.preventDefault()
          return true
        }
        return false
      },
      handlePaste(view, event) {
        const clipboardEvent = event as ClipboardEvent
        const files = clipboardEvent.clipboardData?.files
        if (!files || files.length === 0) return false

        let handled = false
        for (const file of Array.from(files)) {
          if (file.type.startsWith('image/')) {
            handled = true
            insertImageFromFile(view, file, callbacks).catch((error) => {
              console.error('Image attachment failed:', error)
            })
          }
        }
        if (handled) {
          event.preventDefault()
          return true
        }
        return false
      },
    },
  })
}

async function insertImageFromFile(view: EditorView, file: File, callbacks: EditorCallbacks) {
  const pageId = callbacks.getPageId?.()
  if (!pageId) return
  if (file.size > MAX_ATTACHMENT_BYTES) {
    throw new Error('Attachment exceeds 25 MB limit')
  }

  const arrayBuffer = await file.arrayBuffer()
  const data = Array.from(new Uint8Array(arrayBuffer))

  const attachment = await api.createAttachment({
    pageId,
    fileName: file.name,
    mimeType: file.type,
    data,
  })

  const blobUrl = URL.createObjectURL(file)
  const imageNode = schema.nodes.image.create({
    src: blobUrl,
    alt: file.name,
    title: '',
    attachmentId: attachment.id,
  })

  const tr = view.state.tr.replaceSelectionWith(imageNode, false)
  view.dispatch(tr)
  view.focus()
}

export function insertImage(view: EditorView, src: string, alt: string, attachmentId: string | null) {
  const imageNode = schema.nodes.image.create({ src, alt, title: '', attachmentId })
  const tr = view.state.tr.replaceSelectionWith(imageNode, false)
  view.dispatch(tr)
  view.focus()
}

export async function insertAudioBlock(
  view: EditorView,
  pageId: string,
  audioData: number[],
  mimeType: string,
  duration: number,
  title: string,
): Promise<void> {
  if (audioData.length > MAX_ATTACHMENT_BYTES) {
    throw new Error('Attachment exceeds 25 MB limit')
  }

  const attachment = await api.createAttachment({
    pageId,
    fileName: title,
    mimeType,
    data: audioData,
  })

  const audioNote = await api.createAudioNote({
    pageId,
    attachmentId: attachment.id,
    durationSec: duration,
    title,
  })

  const audioNode = schema.nodes.audio_block.create({
    attachmentId: attachment.id,
    audioNoteId: audioNote.id,
    duration,
    title,
    transcription: '',
  })

  const tr = view.state.tr.replaceSelectionWith(audioNode, false)
  view.dispatch(tr)
  view.focus()
}

export async function updateAudioBlockTranscription(
  view: EditorView,
  pos: number,
  transcription: string,
  audioNoteId: string,
) {
  if (audioNoteId) {
    await api.updateAudioNote({ id: audioNoteId, transcription })
  }
  const tr = view.state.tr.setNodeAttribute(pos, 'transcription', transcription)
  view.dispatch(tr)
}

let stopRecordingFn: (() => void) | null = null

export function stopAudioRecording() {
  stopRecordingFn?.()
}

export async function recordAudio(
  onStatus: (recording: boolean, duration: number) => void,
): Promise<{ data: number[]; mimeType: string; duration: number } | null> {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    const recorder = new MediaRecorder(stream)
    const chunks: Blob[] = []
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data)
    }

    return new Promise((resolve) => {
      let duration = 0
      const timer = setInterval(() => {
        duration += 0.1
        onStatus(true, duration)
      }, 100)

      recorder.onstop = async () => {
        clearInterval(timer)
        stream.getTracks().forEach((t) => t.stop())
        onStatus(false, duration)
        const blob = new Blob(chunks, { type: recorder.mimeType || 'audio/webm' })
        const arrayBuffer = await blob.arrayBuffer()
        resolve({
          data: Array.from(new Uint8Array(arrayBuffer)),
          mimeType: recorder.mimeType || 'audio/webm',
          duration,
        })
      }

      recorder.start()

      stopRecordingFn = () => {
        if (recorder.state !== 'inactive') {
          recorder.stop()
        }
      }
    })
  } catch {
    onStatus(false, 0)
    return null
  }
}
