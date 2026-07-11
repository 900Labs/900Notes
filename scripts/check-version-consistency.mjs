import { readFileSync } from 'node:fs'

const packageJson = JSON.parse(readFileSync('package.json', 'utf8'))
const packageLock = JSON.parse(readFileSync('package-lock.json', 'utf8'))
const desktop = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'))
const mobile = JSON.parse(readFileSync('src-tauri/tauri.mobile.conf.json', 'utf8'))
const cargo = readFileSync('src-tauri/Cargo.toml', 'utf8')
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1]

const versions = new Map([
  ['package.json', packageJson.version],
  ['package-lock.json', packageLock.version],
  ['package-lock root', packageLock.packages?.['']?.version],
  ['Cargo.toml', cargoVersion],
  ['tauri.conf.json', desktop.version],
  ['tauri.mobile.conf.json', mobile.version],
])
const expected = packageJson.version
const mismatches = [...versions].filter(([, version]) => version !== expected)
if (mismatches.length > 0) {
  for (const [file, version] of mismatches) {
    console.error(`${file}: expected ${expected}, found ${version ?? 'missing'}`)
  }
  process.exit(1)
}
console.log(`version consistency passed: ${expected}`)
