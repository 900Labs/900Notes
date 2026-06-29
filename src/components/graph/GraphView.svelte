<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { getGraphData } from '../../lib/api'
  import { tagStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { GraphNode, GraphEdge } from '../../lib/types'

  let { onNavigate }: { onNavigate: (pageId: string) => void } = $props()

  let canvas: HTMLCanvasElement
  let ctx: CanvasRenderingContext2D | null = null
  let rafId = 0
  let nodes = $state<(GraphNode & { x: number; y: number; vx: number; vy: number; r: number })[]>([])
  let edges: GraphEdge[] = []
  let nodeMap: Map<string, number> = new Map()
  let hoveredIdx = -1
  let selectedTag = $state<string | null>(null)
  let minLinks = $state(0)
  let isDragging = false
  let dragIdx = -1
  let mouseX = 0
  let mouseY = 0
  let width = 800
  let height = 600
  let animationActive = true

  onMount(async () => {
    await tagStore.loadTags()
    await loadGraph()
    ctx = canvas.getContext('2d')
    if (ctx) {
      resize()
      window.addEventListener('resize', resize)
      animate()
    }
  })

  onDestroy(() => {
    animationActive = false
    cancelAnimationFrame(rafId)
    window.removeEventListener('resize', resize)
  })

  async function loadGraph() {
    const data = await getGraphData()
    const cx = width / 2
    const cy = height / 2
    nodes = data.nodes.map((n) => ({
      ...n,
      x: cx + (Math.random() - 0.5) * 200,
      y: cy + (Math.random() - 0.5) * 200,
      vx: 0,
      vy: 0,
      r: 6 + Math.min(n.linkCount * 1.5, 16),
    }))
    edges = data.edges
    rebuildNodeMap()
  }

  function rebuildNodeMap() {
    nodeMap = new Map()
    nodes.forEach((n, i) => nodeMap.set(n.id, i))
  }

  function resize() {
    const rect = canvas.getBoundingClientRect()
    width = rect.width
    height = rect.height
    canvas.width = width * window.devicePixelRatio
    canvas.height = height * window.devicePixelRatio
    if (ctx) {
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio)
    }
  }

  function getFilteredNodes(): number[] {
    return nodes
      .map((_, i) => i)
      .filter((i) => {
        if (nodes[i].linkCount < minLinks) return false
        return true
      })
  }

  function isNodeVisible(idx: number): boolean {
    return nodes[idx].linkCount >= minLinks
  }

  function simulate() {
    const filtered = new Set(getFilteredNodes())
    const cx = width / 2
    const cy = height / 2
    const k = 0.02
    const repulsion = 800
    const damping = 0.85

    for (let i = 0; i < nodes.length; i++) {
      if (!filtered.has(i)) continue
      if (isDragging && dragIdx === i) continue

      for (let j = 0; j < nodes.length; j++) {
        if (i === j || !filtered.has(j)) continue
        const dx = nodes[i].x - nodes[j].x
        const dy = nodes[i].y - nodes[j].y
        const dist = Math.sqrt(dx * dx + dy * dy) || 1
        const force = repulsion / (dist * dist)
        nodes[i].vx += (dx / dist) * force
        nodes[i].vy += (dy / dist) * force
      }

      nodes[i].vx += (cx - nodes[i].x) * k
      nodes[i].vy += (cy - nodes[i].y) * k

      nodes[i].vx *= damping
      nodes[i].vy *= damping
      nodes[i].x += nodes[i].vx
      nodes[i].y += nodes[i].vy
    }

    const linkSpring = 0.01
    const linkLength = 120
    for (const edge of edges) {
      const si = nodeMap.get(edge.source)
      const ti = nodeMap.get(edge.target)
      if (si === undefined || ti === undefined) continue
      if (!filtered.has(si) || !filtered.has(ti)) continue
      const dx = nodes[ti].x - nodes[si].x
      const dy = nodes[ti].y - nodes[si].y
      const dist = Math.sqrt(dx * dx + dy * dy) || 1
      const force = (dist - linkLength) * linkSpring
      const fx = (dx / dist) * force
      const fy = (dy / dist) * force
      nodes[si].vx += fx
      nodes[si].vy += fy
      nodes[ti].vx -= fx
      nodes[ti].vy -= fy
    }
  }

  function draw() {
    if (!ctx) return
    ctx.clearRect(0, 0, width, height)

    const filtered = new Set(getFilteredNodes())

    ctx.strokeStyle = 'rgba(150, 150, 150, 0.3)'
    ctx.lineWidth = 1
    for (const edge of edges) {
      const si = nodeMap.get(edge.source)
      const ti = nodeMap.get(edge.target)
      if (si === undefined || ti === undefined) continue
      if (!filtered.has(si) || !filtered.has(ti)) continue
      ctx.beginPath()
      ctx.moveTo(nodes[si].x, nodes[si].y)
      ctx.lineTo(nodes[ti].x, nodes[ti].y)
      ctx.stroke()
    }

    for (let i = 0; i < nodes.length; i++) {
      if (!filtered.has(i)) continue
      const n = nodes[i]
      const isHovered = i === hoveredIdx

      ctx.beginPath()
      ctx.arc(n.x, n.y, n.r, 0, Math.PI * 2)
      if (isHovered) {
        ctx.fillStyle = '#3b82f6'
        ctx.strokeStyle = '#1d4ed8'
        ctx.lineWidth = 2
      } else {
        const intensity = Math.min(n.linkCount / 10, 1)
        ctx.fillStyle = `rgb(${100 + intensity * 50}, ${149 + intensity * 50}, ${237})`
        ctx.strokeStyle = '#2563eb'
        ctx.lineWidth = 1
      }
      ctx.fill()
      ctx.stroke()

      if (isHovered || n.r > 12) {
        ctx.fillStyle = isHovered ? '#1e293b' : '#64748b'
        ctx.font = '11px system-ui, sans-serif'
        ctx.textAlign = 'center'
        ctx.fillText(n.title.slice(0, 24), n.x, n.y - n.r - 4)
      }
    }
  }

  function animate() {
    if (!animationActive) return
    simulate()
    draw()
    rafId = requestAnimationFrame(animate)
  }

  function getNodeAt(x: number, y: number): number {
    for (let i = nodes.length - 1; i >= 0; i--) {
      if (!isNodeVisible(i)) continue
      const dx = nodes[i].x - x
      const dy = nodes[i].y - y
      if (Math.sqrt(dx * dx + dy * dy) < nodes[i].r + 4) return i
    }
    return -1
  }

  function handleMousemove(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect()
    mouseX = e.clientX - rect.left
    mouseY = e.clientY - rect.top

    if (isDragging && dragIdx >= 0) {
      nodes[dragIdx].x = mouseX
      nodes[dragIdx].y = mouseY
      nodes[dragIdx].vx = 0
      nodes[dragIdx].vy = 0
    } else {
      hoveredIdx = getNodeAt(mouseX, mouseY)
      canvas.style.cursor = hoveredIdx >= 0 ? 'pointer' : 'default'
    }
  }

  function handleMousedown(_e: MouseEvent) {
    if (hoveredIdx >= 0) {
      isDragging = true
      dragIdx = hoveredIdx
    }
  }

  function handleMouseup(_e: MouseEvent) {
    if (isDragging && dragIdx >= 0) {
      isDragging = false
      dragIdx = -1
    }
  }

  function handleClick(_e: MouseEvent) {
    if (hoveredIdx >= 0 && !isDragging) {
      onNavigate(nodes[hoveredIdx].id)
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center gap-3 px-4 py-2.5 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-sm font-semibold">{$t('graph.title')}</h3>
    <div class="flex-1"></div>
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
  {$t('graph.minLinks')}
  <input
    type="range"
    min="0"
    max="10"
    bind:value={minLinks}
    class="w-20 accent-blue-500"
  />
  <span class="w-4 text-center">{minLinks}</span>
</label>
  </div>

  <div class="flex-1 relative overflow-hidden bg-gray-50 dark:bg-gray-900">
    <canvas
      bind:this={canvas}
      class="w-full h-full"
      onmousemove={handleMousemove}
      onmousedown={handleMousedown}
      onmouseup={handleMouseup}
      onclick={handleClick}
      onmouseleave={() => { hoveredIdx = -1; isDragging = false; dragIdx = -1 }}
      aria-label={$t('graph.title')}
    ></canvas>
    {#if nodes.length === 0}
      <div class="absolute inset-0 flex items-center justify-center text-sm text-gray-400">
        {$t('graph.empty')}
      </div>
    {/if}
  </div>
</div>
