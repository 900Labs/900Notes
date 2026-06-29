import type { Plugin, PluginManifest, CustomBlockDef, LoadedPlugin, PluginApi } from '../lib/types'
import { invoke } from '@tauri-apps/api/core'
import { getAllPlugins, getEnabledPlugins, setPluginEnabled, uninstallPlugin, scanPluginsDir, readPluginFile } from '../lib/api'

class PluginStore {
  plugins = $state<Plugin[]>([])
  loadedPlugins = $state<LoadedPlugin[]>([])
  loading = $state(false)
  error = $state<string | null>(null)

  async loadPlugins() {
    this.loading = true
    this.error = null
    try {
      this.plugins = await getAllPlugins()
    } catch (e) {
      this.error = String(e)
    } finally {
      this.loading = false
    }
  }

  async scanAndInstall(appDataDir: string) {
    this.loading = true
    this.error = null
    try {
      const installed = await scanPluginsDir(appDataDir)
      this.plugins = await getAllPlugins()
      return installed
    } catch (e) {
      this.error = String(e)
      return []
    } finally {
      this.loading = false
    }
  }

  async togglePlugin(id: string, enabled: boolean) {
    try {
      await setPluginEnabled(id, enabled)
      this.plugins = this.plugins.map((p) =>
        p.id === id ? { ...p, enabled } : p
      )
    } catch (e) {
      this.error = String(e)
    }
  }

  async removePlugin(id: string) {
    try {
      await uninstallPlugin(id)
      this.plugins = this.plugins.filter((p) => p.id !== id)
      this.loadedPlugins = this.loadedPlugins.filter((p) => p.manifest.id !== id)
    } catch (e) {
      this.error = String(e)
    }
  }

  getCustomBlocks(): CustomBlockDef[] {
    return this.loadedPlugins.flatMap((p) => p.blocks)
  }

  getPluginCommands(): { id: string; label: string; handler: () => void }[] {
    return this.loadedPlugins.flatMap((p) => p.commands)
  }

  runHooks(event: string, ...args: unknown[]) {
    for (const plugin of this.loadedPlugins) {
      const handlers = plugin.hooks.get(event)
      if (handlers) {
        for (const handler of handlers) {
          try {
            handler(...args)
          } catch (e) {
            console.error(`Plugin ${plugin.manifest.id} hook ${event} error:`, e)
          }
        }
      }
    }
  }
}

export const pluginStore = new PluginStore()
