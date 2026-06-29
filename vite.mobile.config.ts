import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  root: 'src/mobile',
  server: {
    host: host || false,
    port: 1421,
    strictPort: true,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1422,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    outDir: 'dist-mobile',
    emptyOutDir: true,
  },
})
