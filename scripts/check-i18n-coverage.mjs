#!/usr/bin/env node
import fs from 'node:fs'

const sourcePath = new URL('../src/i18n/index.ts', import.meta.url)
const source = fs.readFileSync(sourcePath, 'utf8')

const localeCodes = [...source.matchAll(/\{ code: '([^']+)'/g)].map((match) => match[1])

function extractBlock(locale) {
  const startToken = `\n  ${locale}: {`
  const start = source.indexOf(startToken)
  if (start === -1) {
    throw new Error(`Missing translation block for locale "${locale}"`)
  }

  const blockStart = start + startToken.length
  const end = source.indexOf('\n  },', blockStart)
  if (end === -1) {
    throw new Error(`Could not find end of translation block for locale "${locale}"`)
  }

  return source.slice(blockStart, end)
}

function keysFor(locale) {
  return [...extractBlock(locale).matchAll(/^\s+'([^']+)':/gm)].map((match) => match[1])
}

if (localeCodes.length === 0) {
  throw new Error('No locale codes found')
}

const allKeys = new Map(localeCodes.map((locale) => [locale, keysFor(locale)]))
const englishKeys = allKeys.get('en')

if (!englishKeys) {
  throw new Error('English translation block is required')
}

let failed = false

for (const locale of localeCodes) {
  if (locale === 'en') continue

  const localeKeys = allKeys.get(locale) ?? []
  const localeKeySet = new Set(localeKeys)
  const englishKeySet = new Set(englishKeys)
  const missing = englishKeys.filter((key) => !localeKeySet.has(key))
  const extra = localeKeys.filter((key) => !englishKeySet.has(key))

  if (missing.length > 0 || extra.length > 0) {
    failed = true
    console.error(`${locale}: ${missing.length} missing, ${extra.length} extra`)
    for (const key of missing) console.error(`  missing ${key}`)
    for (const key of extra) console.error(`  extra ${key}`)
  }
}

if (failed) {
  process.exit(1)
}

console.log(`i18n coverage passed: ${localeCodes.length} locales, ${englishKeys.length} keys`)
