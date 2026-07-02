<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { getGraphData } from '../../lib/api'
  import { tagStore } from '../../stores/app.svelte'
  import { t } from '../../i18n'
  import type { GraphNode, GraphEdge } from '../../lib/types'

  let {
    focusPageId = null,
    onNavigate,
    onClose,
  }: {
    focusPageId?: string | null
    onNavigate: (pageId: string) => void
    onClose: () => void
  } = $props()

  let canvas: HTMLCanvasElement
  let ctx: CanvasRenderingContext2D | null = null
  let rafId = 0
  let nodes = $state<(GraphNode & { x: number; y: number; vx: number; vy: number; r: number })[]>([])
  let edges = $state<GraphEdge[]>([])
  let nodeMap: Map<string, number> = new Map()
  let hoveredIdx = -1
  let minLinks = $state(0)
  let localDepth = $state(1)
  let showOrphans = $state(true)
  let showLabels = $state(false)
  let graphQuery = $state('')
  let paused = $state(false)
  let linkDistance = $state(120)
  let repelForce = $state(800)
  let viewScale = $state(1)
  let viewX = $state(0)
  let viewY = $state(0)
  let isDragging = false
  let dragIdx = -1
  let isPanning = false
  let panStartX = 0
  let panStartY = 0
  let panStartViewX = 0
  let panStartViewY = 0
  let suppressClick = false
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
      ctx.setTransform(window.devicePixelRatio, 0, 0, window.devicePixelRatio, 0, 0)
    }
  }

  function resetLayout() {
    const cx = width / 2
    const cy = height / 2
    nodes = nodes.map((n) => ({
      ...n,
      x: cx + (Math.random() - 0.5) * Math.min(width, 500),
      y: cy + (Math.random() - 0.5) * Math.min(height, 400),
      vx: 0,
      vy: 0,
    }))
    rebuildNodeMap()
  }

  function getFilteredNodes(): number[] {
    const localIds = focusPageId ? getLocalNodeIds(focusPageId, localDepth) : null
    const query = graphQuery.trim().toLowerCase()
    return nodes
      .map((_, i) => i)
      .filter((i) => {
        if (localIds && !localIds.has(nodes[i].id)) return false
        if (nodes[i].linkCount < minLinks) return false
        if (!showOrphans && nodes[i].linkCount === 0) return false
        if (query && !nodes[i].title.toLowerCase().includes(query)) return false
        return true
      })
  }

  function isNodeVisible(idx: number): boolean {
    return visibleNodeIndices.includes(idx)
  }

  function getLocalNodeIds(rootId: string, depth: number): Set<string> {
    const visible = new Set<string>([rootId])
    let frontier = new Set<string>([rootId])

    for (let level = 0; level < depth; level++) {
      const next = new Set<string>()
      for (const edge of edges) {
        if (frontier.has(edge.source)) next.add(edge.target)
        if (frontier.has(edge.target)) next.add(edge.source)
      }
      for (const id of next) visible.add(id)
      frontier = next
      if (frontier.size === 0) break
    }

    return visible
  }

  let focusTitle = $derived(
    focusPageId ? nodes.find((node) => node.id === focusPageId)?.title ?? 'Current page' : null,
  )

  let visibleNodeIndices = $derived(getFilteredNodes())
  let visibleNodes = $derived(visibleNodeIndices.map((idx) => nodes[idx]))
  let visibleNodeIds = $derived(new Set(visibleNodes.map((node) => node.id)))

  let visibleEdgeCount = $derived.by(() => {
    return edges.filter((edge) => visibleNodeIds.has(edge.source) && visibleNodeIds.has(edge.target)).length
  })

  function screenToWorld(x: number, y: number): { x: number; y: number } {
    return {
      x: (x - viewX) / viewScale,
      y: (y - viewY) / viewScale,
    }
  }

  function centerNode(id: string) {
    const idx = nodeMap.get(id)
    if (idx === undefined) return
    viewX = width / 2 - nodes[idx].x * viewScale
    viewY = height / 2 - nodes[idx].y * viewScale
    hoveredIdx = idx
  }

  function resetView() {
    viewScale = 1
    viewX = 0
    viewY = 0
  }

  function simulate() {
    const filtered = new Set(visibleNodeIndices)
    const cx = width / 2
    const cy = height / 2
    const k = 0.02
    const repulsion = Number(repelForce)
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
    const linkLength = Number(linkDistance)
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

    const filtered = new Set(visibleNodeIndices)

    ctx.save()
    ctx.translate(viewX, viewY)
    ctx.scale(viewScale, viewScale)
    ctx.strokeStyle = 'rgba(150, 150, 150, 0.3)'
    ctx.lineWidth = 1 / viewScale
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
        ctx.lineWidth = 2 / viewScale
      } else {
        const intensity = Math.min(n.linkCount / 10, 1)
        ctx.fillStyle = `rgb(${100 + intensity * 50}, ${149 + intensity * 50}, ${237})`
        ctx.strokeStyle = '#2563eb'
        ctx.lineWidth = 1 / viewScale
      }
      ctx.fill()
      ctx.stroke()

      if (showLabels || isHovered || n.r > 12) {
        ctx.fillStyle = isHovered ? '#1e293b' : '#64748b'
        ctx.font = `${11 / viewScale}px system-ui, sans-serif`
        ctx.textAlign = 'center'
        ctx.fillText(n.title.slice(0, 24), n.x, n.y - n.r - 4)
      }
    }
    ctx.restore()
  }

  function animate() {
    if (!animationActive) return
    if (!paused) simulate()
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
    const screenX = e.clientX - rect.left
    const screenY = e.clientY - rect.top
    const point = screenToWorld(screenX, screenY)
    mouseX = point.x
    mouseY = point.y

    if (isPanning) {
      viewX = panStartViewX + (e.clientX - panStartX)
      viewY = panStartViewY + (e.clientY - panStartY)
      if (Math.abs(e.clientX - panStartX) > 3 || Math.abs(e.clientY - panStartY) > 3) suppressClick = true
      canvas.style.cursor = 'grabbing'
      return
    }

    if (isDragging && dragIdx >= 0) {
      nodes[dragIdx].x = mouseX
      nodes[dragIdx].y = mouseY
      nodes[dragIdx].vx = 0
      nodes[dragIdx].vy = 0
      suppressClick = true
    } else {
      hoveredIdx = getNodeAt(mouseX, mouseY)
      canvas.style.cursor = hoveredIdx >= 0 ? 'pointer' : 'default'
    }
  }

  function handleMousedown(e: MouseEvent) {
    suppressClick = false
    if (hoveredIdx >= 0) {
      isDragging = true
      dragIdx = hoveredIdx
    } else {
      isPanning = true
      panStartX = e.clientX
      panStartY = e.clientY
      panStartViewX = viewX
      panStartViewY = viewY
      canvas.style.cursor = 'grab'
    }
  }

  function handleMouseup(_e: MouseEvent) {
    if (isDragging && dragIdx >= 0) {
      isDragging = false
      dragIdx = -1
    }
    if (isPanning) {
      isPanning = false
    }
  }

  function handleClick(_e: MouseEvent) {
    if (suppressClick) {
      suppressClick = false
      return
    }
    if (hoveredIdx >= 0 && !isDragging) {
      onNavigate(nodes[hoveredIdx].id)
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault()
    const rect = canvas.getBoundingClientRect()
    const screenX = e.clientX - rect.left
    const screenY = e.clientY - rect.top
    const before = screenToWorld(screenX, screenY)
    const nextScale = Math.min(2.5, Math.max(0.4, viewScale * (e.deltaY > 0 ? 0.9 : 1.1)))
    viewScale = nextScale
    viewX = screenX - before.x * viewScale
    viewY = screenY - before.y * viewScale
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex flex-wrap items-center gap-3 px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-950">
    <div>
      <h3 class="text-sm font-semibold">{focusTitle ? $t('graph.localTitle', { title: focusTitle }) : $t('graph.title')}</h3>
      <p class="text-xs text-gray-500 dark:text-gray-400">{$t('graph.stats', { pages: visibleNodes.length, links: visibleEdgeCount })}</p>
    </div>
    <div class="flex-1"></div>
    <input
      bind:value={graphQuery}
      type="search"
      placeholder={$t('graph.searchPlaceholder')}
      class="h-8 w-44 rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 px-2 text-xs focus:border-accent focus:outline-none"
    />
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
    {#if focusPageId}
      <label class="text-xs text-gray-500 flex items-center gap-1.5">
        {$t('graph.depth')}
        <input
          type="range"
          min="1"
          max="4"
          bind:value={localDepth}
          class="w-16 accent-blue-500"
        />
        <span class="w-4 text-center">{localDepth}</span>
      </label>
    {/if}
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
      {$t('graph.zoom')}
      <input
        type="range"
        min="0.4"
        max="2.5"
        step="0.1"
        bind:value={viewScale}
        class="w-20 accent-blue-500"
      />
    </label>
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
      {$t('graph.distance')}
      <input
        type="range"
        min="60"
        max="220"
        bind:value={linkDistance}
        class="w-20 accent-blue-500"
      />
    </label>
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
      {$t('graph.repel')}
      <input
        type="range"
        min="200"
        max="1600"
        step="100"
        bind:value={repelForce}
        class="w-20 accent-blue-500"
      />
    </label>
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
      <input type="checkbox" bind:checked={showOrphans} class="rounded border-gray-300" />
      {$t('graph.orphans')}
    </label>
    <label class="text-xs text-gray-500 flex items-center gap-1.5">
      <input type="checkbox" bind:checked={showLabels} class="rounded border-gray-300" />
      {$t('graph.labels')}
    </label>
    <button
      onclick={() => (paused = !paused)}
      class="px-2.5 py-1 rounded-md text-xs border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800"
    >
      {paused ? $t('graph.resume') : $t('graph.pause')}
    </button>
    <button
      onclick={resetLayout}
      class="px-2.5 py-1 rounded-md text-xs border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800"
    >
      {$t('graph.reset')}
    </button>
    <button
      onclick={resetView}
      class="px-2.5 py-1 rounded-md text-xs border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800"
    >
      {$t('graph.resetView')}
    </button>
    <button
      onclick={onClose}
      class="px-2.5 py-1 rounded-md text-xs border border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800"
    >
      {$t('common.close')}
    </button>
  </div>

  <div class="flex-1 relative overflow-hidden bg-gray-50 dark:bg-gray-900">
    <canvas
      bind:this={canvas}
      class="w-full h-full"
      onmousemove={handleMousemove}
      onmousedown={handleMousedown}
      onmouseup={handleMouseup}
      onclick={handleClick}
      onwheel={handleWheel}
      onmouseleave={() => { hoveredIdx = -1; isDragging = false; isPanning = false; dragIdx = -1 }}
      aria-label={$t('graph.title')}
    ></canvas>
    {#if graphQuery || focusPageId}
      <div class="absolute right-3 top-3 w-72 max-w-[calc(100%-1.5rem)] rounded-lg border border-gray-200 dark:border-gray-700 bg-white/95 dark:bg-gray-950/95 shadow-lg overflow-hidden">
        <div class="flex items-center justify-between border-b border-gray-100 dark:border-gray-800 px-3 py-2">
          <div class="text-xs font-semibold text-gray-500 uppercase tracking-wide">{$t('graph.visiblePages')}</div>
          <div class="text-[11px] text-gray-400">{visibleNodes.length}</div>
        </div>
        <div class="max-h-64 overflow-y-auto p-1">
          {#if visibleNodes.length === 0}
            <div class="px-3 py-4 text-sm text-gray-400">{$t('graph.noMatches')}</div>
          {:else}
            {#each visibleNodes.slice(0, 12) as node (node.id)}
              <div class="flex items-center gap-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800">
                <button
                  type="button"
                  onclick={() => centerNode(node.id)}
                  class="min-w-0 flex-1 px-2 py-1.5 text-left"
                  title={$t('graph.center')}
                >
                  <span class="block truncate text-sm">{node.icon || '#'} {node.title || $t('editor.titlePlaceholder')}</span>
                  <span class="block text-[11px] text-gray-400">{$t('graph.linkCount', { count: node.linkCount })}</span>
                </button>
                <button
                  type="button"
                  onclick={() => onNavigate(node.id)}
                  class="mr-1 rounded px-2 py-1 text-xs text-accent hover:bg-accent/10"
                >
                  {$t('graph.open')}
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
    {#if nodes.length === 0}
      <div class="absolute inset-0 flex items-center justify-center text-sm text-gray-400">
        {$t('graph.empty')}
      </div>
    {/if}
    <div class="absolute bottom-3 left-3 rounded-md bg-white/90 dark:bg-gray-950/90 border border-gray-200 dark:border-gray-800 px-2 py-1 text-[11px] text-gray-500 dark:text-gray-400">
      {$t('graph.panZoomHint')}
    </div>
  </div>
</div>
