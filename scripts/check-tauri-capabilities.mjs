import { readFile } from 'node:fs/promises'

const capabilityPath = new URL('../src-tauri/capabilities/default.json', import.meta.url)
const capability = JSON.parse(await readFile(capabilityPath, 'utf8'))
const permissions = new Set(capability.permissions ?? [])
const requiredPermissions = ['core:window:allow-destroy']
const missing = requiredPermissions.filter((permission) => !permissions.has(permission))

if (missing.length > 0) {
  console.error(`Missing required Tauri permissions: ${missing.join(', ')}`)
  process.exit(1)
}

console.log(`Tauri capability check passed (${requiredPermissions.length} required permission)`)
