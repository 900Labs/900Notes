import type { Plugin, PluginManifest, CustomBlockDef, LoadedPlugin, PluginApi } from '../types'
import { readPluginFile, getEnabledPlugins } from '../api'
import { pluginStore } from '../../stores/plugins.svelte'

export async function loadEnabledPlugins(appDataDir: string): Promise<void> {
  const enabledPlugins = await getEnabledPlugins()
  const loaded: LoadedPlugin[] = []

  for (const plugin of enabledPlugins) {
    try {
      const loadedPlugin = await loadPlugin(appDataDir, plugin)
      loaded.push(loadedPlugin)
    } catch (e) {
      console.error(`Failed to load plugin ${plugin.id}:`, e)
    }
  }

  pluginStore.loadedPlugins = loaded
}

async function loadPlugin(appDataDir: string, plugin: Plugin): Promise<LoadedPlugin> {
  const blocks: CustomBlockDef[] = []
  const commands: { id: string; label: string; handler: () => void }[] = []
  const hooks = new Map<string, ((...args: unknown[]) => void)[]>()

  const api: PluginApi = {
    registerBlock: (def: CustomBlockDef) => {
      blocks.push(def)
    },
    registerCommand: (id: string, label: string, handler: () => void) => {
      commands.push({ id, label, handler })
    },
    registerHook: (event: string, handler: (...args: unknown[]) => void) => {
      const existing = hooks.get(event) ?? []
      existing.push(handler)
      hooks.set(event, existing)
    },
  }

  const manifest: PluginManifest = {
    id: plugin.id,
    name: plugin.name,
    version: plugin.version,
    author: plugin.author,
    description: plugin.description,
    entryPoint: plugin.entryPoint,
    customBlocks: [],
  }

  const jsCode = await readPluginFile(appDataDir, plugin.id, plugin.entryPoint)

  const moduleFn = new Function('plugin', jsCode)
  moduleFn(api)

  return { manifest, api, blocks, commands, hooks }
}

export function getCustomBlockDefs(): CustomBlockDef[] {
  return pluginStore.getCustomBlocks()
}

export function getPluginCommands(): { id: string; label: string; handler: () => void }[] {
  return pluginStore.getPluginCommands()
}

export function runPluginHooks(event: string, ...args: unknown[]) {
  pluginStore.runHooks(event, ...args)
}
