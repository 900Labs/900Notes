import type { Node } from 'prosemirror-model'
import type { EditorView, NodeView } from 'prosemirror-view'
import katex from 'katex'
import mermaid from 'mermaid'
import * as api from '../api'

let mermaidInitialized = false

function ensureMermaid() {
  if (!mermaidInitialized) {
    mermaid.initialize({
      startOnLoad: false,
      theme: document.documentElement.classList.contains('dark') ? 'dark' : 'default',
      securityLevel: 'loose',
    })
    mermaidInitialized = true
  }
}

export class MathInlineView implements NodeView {
  dom: HTMLElement
  private node: Node
  private view: EditorView
  private getPos: () => number | undefined
  private editing = false

  constructor(node: Node, view: EditorView, getPos: () => number | undefined) {
    this.node = node
    this.view = view
    this.getPos = getPos
    this.dom = document.createElement('span')
    this.dom.className = 'pm-math-inline'
    this.dom.setAttribute('data-type', 'math-inline')
    this.render()
    this.setupEvents()
  }

  private render() {
    if (this.editing) {
      this.dom.contentEditable = 'true'
      this.dom.textContent = this.node.attrs.latex
    } else {
      this.dom.contentEditable = 'false'
      try {
        this.dom.innerHTML = katex.renderToString(this.node.attrs.latex, {
          throwOnError: false,
          displayMode: false,
        })
      } catch {
        this.dom.textContent = this.node.attrs.latex
      }
    }
  }

  private setupEvents() {
    this.dom.addEventListener('click', (e) => {
      e.stopPropagation()
      this.editing = true
      this.render()
      this.dom.focus()
      const range = document.createRange()
      range.selectNodeContents(this.dom)
      const sel = window.getSelection()
      sel?.removeAllRanges()
      sel?.addRange(range)
    })

    this.dom.addEventListener('blur', () => {
      this.editing = false
      const latex = this.dom.textContent || ''
      const pos = this.getPos()
      if (pos !== undefined && latex !== this.node.attrs.latex) {
        const tr = this.view.state.tr.setNodeMarkup(pos, undefined, { latex })
        this.view.dispatch(tr)
      }
      this.node = this.node.type.create({ latex })
      this.render()
    })

    this.dom.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        this.dom.blur()
      }
      if (e.key === 'Escape') {
        e.preventDefault()
        this.editing = false
        this.render()
      }
    })
  }

  update(node: Node): boolean {
    if (node.type !== this.node.type) return false
    this.node = node
    if (!this.editing) this.render()
    return true
  }

  ignoreMutation(): boolean {
    return true
  }

  stopEvent(): boolean {
    return this.editing
  }
}

export class MathBlockView implements NodeView {
  dom: HTMLElement
  private node: Node
  private view: EditorView
  private getPos: () => number | undefined
  private editing = false

  constructor(node: Node, view: EditorView, getPos: () => number | undefined) {
    this.node = node
    this.view = view
    this.getPos = getPos
    this.dom = document.createElement('div')
    this.dom.className = 'pm-math-block'
    this.dom.setAttribute('data-type', 'math-block')
    this.render()
    this.setupEvents()
  }

  private render() {
    if (this.editing) {
      this.dom.contentEditable = 'true'
      this.dom.textContent = this.node.attrs.latex
    } else {
      this.dom.contentEditable = 'false'
      try {
        this.dom.innerHTML = katex.renderToString(this.node.attrs.latex, {
          throwOnError: false,
          displayMode: true,
        })
      } catch {
        this.dom.textContent = this.node.attrs.latex
      }
    }
  }

  private setupEvents() {
    this.dom.addEventListener('click', (e) => {
      e.stopPropagation()
      this.editing = true
      this.render()
      this.dom.focus()
      const range = document.createRange()
      range.selectNodeContents(this.dom)
      const sel = window.getSelection()
      sel?.removeAllRanges()
      sel?.addRange(range)
    })

    this.dom.addEventListener('blur', () => {
      this.editing = false
      const latex = this.dom.textContent || ''
      const pos = this.getPos()
      if (pos !== undefined && latex !== this.node.attrs.latex) {
        const tr = this.view.state.tr.setNodeMarkup(pos, undefined, { latex })
        this.view.dispatch(tr)
      }
      this.node = this.node.type.create({ latex })
      this.render()
    })

    this.dom.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        this.dom.blur()
      }
      if (e.key === 'Escape') {
        e.preventDefault()
        this.editing = false
        this.render()
      }
    })
  }

  update(node: Node): boolean {
    if (node.type !== this.node.type) return false
    this.node = node
    if (!this.editing) this.render()
    return true
  }

  ignoreMutation(): boolean {
    return true
  }

  stopEvent(): boolean {
    return this.editing
  }
}

export class MermaidBlockView implements NodeView {
  dom: HTMLElement
  contentDOM: HTMLElement
  private node: Node
  private view: EditorView
  private getPos: () => number | undefined
  private renderedEl: HTMLElement | null = null
  private showingRendered = true

  constructor(node: Node, view: EditorView, getPos: () => number | undefined) {
    this.node = node
    this.view = view
    this.getPos = getPos
    this.dom = document.createElement('div')
    this.dom.className = 'pm-mermaid-wrapper'
    this.dom.setAttribute('data-type', 'mermaid')

    this.contentDOM = document.createElement('pre')
    this.contentDOM.className = 'pm-mermaid-source'
    this.contentDOM.style.display = 'none'
    this.contentDOM.textContent = node.textContent

    this.renderedEl = document.createElement('div')
    this.renderedEl.className = 'pm-mermaid-rendered'
    this.dom.appendChild(this.renderedEl)
    this.dom.appendChild(this.contentDOM)

    this.renderDiagram()
    this.setupEvents()
  }

  private async renderDiagram() {
    if (!this.renderedEl) return
    const source = this.contentDOM.textContent || ''
    if (!source.trim()) {
      this.renderedEl.innerHTML = '<span class="pm-mermaid-placeholder">Mermaid diagram</span>'
      return
    }
    try {
      ensureMermaid()
      const id = 'mermaid-' + Math.random().toString(36).substring(2, 11)
      const { svg } = await mermaid.render(id, source)
      this.renderedEl.innerHTML = svg
    } catch (err) {
      this.renderedEl.textContent = `Mermaid error: ${(err as Error).message}`
    }
  }

  private setupEvents() {
    this.dom.addEventListener('dblclick', (e) => {
      e.stopPropagation()
      this.showingRendered = !this.showingRendered
      if (this.showingRendered) {
        this.renderedEl!.style.display = ''
        this.contentDOM.style.display = 'none'
        this.renderDiagram()
      } else {
        this.renderedEl!.style.display = 'none'
        this.contentDOM.style.display = ''
        this.contentDOM.focus()
      }
    })
  }

  update(node: Node): boolean {
    if (node.type !== this.node.type) return false
    this.node = node
    if (this.showingRendered) {
      this.renderDiagram()
    }
    return true
  }

  ignoreMutation(): boolean {
    return false
  }

  stopEvent(): boolean {
    return false
  }
}

export class AudioBlockView implements NodeView {
  dom: HTMLElement
  private node: Node
  private view: EditorView
  private getPos: () => number | undefined
  private audioEl: HTMLAudioElement | null = null
  private blobUrl: string | null = null

  constructor(node: Node, view: EditorView, getPos: () => number | undefined) {
    this.node = node
    this.view = view
    this.getPos = getPos
    this.dom = document.createElement('div')
    this.dom.className = 'pm-audio-block-container'
    this.render()
  }

  private async render() {
    const { attachmentId, duration, title, transcription, audioNoteId } = this.node.attrs as {
      attachmentId: string
      duration: number
      title: string
      transcription: string
      audioNoteId: string
    }

    this.dom.innerHTML = ''

    if (!attachmentId) {
      this.dom.innerHTML = '<div class="pm-audio-empty">Audio block (no data)</div>'
      return
    }

    const wrapper = document.createElement('div')
    wrapper.className = 'pm-audio-wrapper'

    const header = document.createElement('div')
    header.className = 'pm-audio-header'

    const icon = document.createElement('span')
    icon.className = 'pm-audio-icon'
    icon.textContent = '🎙'
    header.appendChild(icon)

    const titleEl = document.createElement('span')
    titleEl.className = 'pm-audio-title'
    titleEl.textContent = title || 'Audio Note'
    header.appendChild(titleEl)

    const durationEl = document.createElement('span')
    durationEl.className = 'pm-audio-duration'
    durationEl.textContent = formatDuration(duration)
    header.appendChild(durationEl)

    wrapper.appendChild(header)

    const audio = document.createElement('audio')
    audio.controls = true
    audio.className = 'pm-audio-player'
    audio.preload = 'metadata'
    wrapper.appendChild(audio)
    this.audioEl = audio

    if (transcription) {
      const transEl = document.createElement('div')
      transEl.className = 'pm-audio-transcription'
      transEl.textContent = transcription
      wrapper.appendChild(transEl)
    }

    this.dom.appendChild(wrapper)

    this.loadAudio(audio, attachmentId)
  }

  private async loadAudio(audio: HTMLAudioElement, attachmentId: string) {
    try {
      const resp = await api.getAttachment(attachmentId)
      const blob = new Blob([new Uint8Array(resp.data)], { type: resp.attachment.mimeType })
      if (this.blobUrl) URL.revokeObjectURL(this.blobUrl)
      this.blobUrl = URL.createObjectURL(blob)
      audio.src = this.blobUrl
    } catch {
      audio.removeAttribute('src')
    }
  }

  update(node: Node): boolean {
    if (node.type !== this.node.type) return false
    const oldAttrs = this.node.attrs
    this.node = node
    if (
      oldAttrs.attachmentId !== node.attrs.attachmentId ||
      oldAttrs.title !== node.attrs.title ||
      oldAttrs.transcription !== node.attrs.transcription ||
      oldAttrs.duration !== node.attrs.duration
    ) {
      this.render()
    }
    return true
  }

  ignoreMutation(): boolean {
    return true
  }

  stopEvent(): boolean {
    return true
  }

  destroy() {
    if (this.blobUrl) URL.revokeObjectURL(this.blobUrl)
  }
}

function formatDuration(sec: number): string {
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

export function createNodeViews(): Record<string, (node: Node, view: EditorView, getPos: () => number | undefined) => NodeView> {
  return {
    math_inline: (node, view, getPos) => new MathInlineView(node, view, getPos),
    math_block: (node, view, getPos) => new MathBlockView(node, view, getPos),
    mermaid_block: (node, view, getPos) => new MermaidBlockView(node, view, getPos),
    audio_block: (node, view, getPos) => new AudioBlockView(node, view, getPos),
  }
}
