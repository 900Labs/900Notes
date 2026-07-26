import type { Plugin, PluginManifest, CustomBlockDef, LoadedPlugin, PluginApi } from '../types'
import { getEnabledPlugins } from '../api'
import { pluginStore } from '../../stores/plugins.svelte'

/**
 * Plugin runtime execution is intentionally NOT wired up yet.
 *
 * The management UI (scan / install / enable / disable) is live, but running
 * third-party plugin code requires a sandboxed execution context (a Web Worker
 * or sandboxed iframe). The previous prototype used `new Function(...)`, which
 * is blocked by the desktop Content-Security-Policy (`script-src 'self'`) and
 * would be unsafe to enable without that sandbox. Until the sandbox lands, this
 * loader is a no-op so enabled plugins never execute untrusted JavaScript.
 *
 * The helpers below still expose the (empty) registry accessors used by the
 * editor so call sites remain valid while runtime loading is disabled.
 */
export async function loadEnabledPlugins(): Promise<void> {
  // Touch the async API so the import graph and any future implementation
  // stay exercised, but do not evaluate plugin source.
  await getEnabledPlugins().catch(() => [])
  pluginStore.loadedPlugins = []
}

export function getCustomBlockDefs(): CustomBlockDef[] {
  return pluginStore.getCustomBlocks()
}

export function getPluginCommands(): { id: string; label: string; handler: () => void }[] {
  return pluginStore.getPluginCommands()
}

export function runPluginHooks(_event: string, ..._args: unknown[]) {
  // No-op until runtime loading is enabled.
}

/**
 * Reserved for the future sandboxed runtime. Kept so the plugin contract
 * (`PluginApi`) stays documented; not invoked until a sandbox exists.
 */
export async function buildPluginApi(_plugin: Plugin): Promise<LoadedPlugin> {
  const manifest: PluginManifest = {
    id: _plugin.id,
    name: _plugin.name,
    version: _plugin.version,
    author: _plugin.author,
    description: _plugin.description,
    entryPoint: _plugin.entryPoint,
    customBlocks: [],
  }
  const api: PluginApi = {
    registerBlock: () => {},
    registerCommand: () => {},
    registerHook: () => {},
  }
  return { manifest, api, blocks: [], commands: [], hooks: new Map() }
}
