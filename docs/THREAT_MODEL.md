# 900Notes Threat Model

**Version**: 1.0  
**Date**: 2026-06-29  
**Status**: Active

## 1. Overview

900Notes is a local-first knowledge base and note-taking application built with Tauri (Rust backend + Svelte frontend). All data is stored locally in a SQLite database. Optional features include LAN-based sync (mDNS), CRDT-based conflict-free merging (Automerge), encrypted share bundles, database encryption at rest, and team workspaces.

This document formalizes the security posture, identifies threats, and describes mitigations.

## 2. Assets

| Asset | Description | Location |
|-------|-------------|----------|
| Page content | User's notes, titles, ProseMirror JSON | SQLite `pages` table |
| Tags & properties | Organizational metadata | SQLite `tags`, `page_tags`, `page_properties` |
| Attachments | Files (images, PDFs, audio) | SQLite `attachments` table (BLOBs) |
| Settings | User preferences, theme, language | SQLite `settings` table |
| Sync state | Automerge CRDT document | SQLite `sync_state` table |
| Encryption keys | Derived from passphrase, not stored | In-memory only |
| Workspace registry | List of workspaces + DB paths | `workspaces.json` in app data dir |

## 3. Trust Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    User's Machine                        │
│  ┌─────────────┐    ┌─────────────┐    ┌──────────────┐ │
│  │  Svelte UI  │◄──►│ Tauri IPC   │◄──►│  Rust Core   │ │
│  │  (WebView)  │    │  Boundary   │    │  (SQLite)    │ │
│  └─────────────┘    └─────────────┘    └──────┬───────┘ │
│                                               │         │
│  ┌────────────────────────────────────────────┘         │
│  │  File System (app_data_dir)                          │
│  │  ├── 900notes.db (or .enc + .meta)                   │
│  │  ├── workspaces.json                                 │
│  │  └── <workspace-id>.db                               │
│  └──────────────────────────────────────────────────────│
│                                                         │
│  ┌─────────────┐         ┌─────────────┐               │
│  │  LAN Sync   │◄──────►│  Peer Device │               │
│  │  (mDNS)     │   UDP    │  (900Notes)  │               │
│  └─────────────┘         └─────────────┘               │
└─────────────────────────────────────────────────────────┘
```

### Trust Boundaries

1. **Tauri IPC Boundary**: Between the Svelte WebView and Rust backend. All commands are explicitly registered; no arbitrary code execution from the frontend.
2. **File System Boundary**: Data at rest on disk. Protected by OS-level permissions and optional AES-256-GCM encryption.
3. **LAN Sync Boundary**: Network communication with peer devices. Unencrypted UDP, limited to local network.

## 4. Threat Actors

| Actor | Capability | Motivation |
|-------|-----------|------------|
| **Local attacker** | Physical access to device, can read disk | Data theft, surveillance |
| **Malware** | Runs as user, can read files, memory | Data exfiltration |
| **Network attacker (LAN)** | Can intercept/modify LAN traffic | Eavesdrop on sync data |
| **Supply chain** | Compromised dependency | Code execution, data theft |
| **Insider (team workspace)** | Has share bundle + passphrase | Access shared pages |

## 5. Threat Analysis

### 5.1 Data at Rest (T1)

**Threat**: Attacker gains access to the SQLite database file.  
**Impact**: Full disclosure of all notes, tags, attachments.  
**Likelihood**: Medium (stolen laptop, disk imaging, cloud backup access).  
**Mitigation**: 
- AES-256-GCM database encryption (Sprint 14). Passphrase not stored.
- Without passphrase, encrypted `.enc` file is indistinguishable from random bytes.
- **Residual risk**: WAL/SHM files are not encrypted while the database is in use. Plain DB exists while unlocked.

**Severity**: HIGH (without encryption) → MEDIUM (with encryption)

### 5.2 LAN Sync Eavesdropping (T2)

**Threat**: Network attacker intercepts sync traffic between peers.  
**Impact**: Disclosure of page content being synced.  
**Likelihood**: Low (requires being on same LAN).  
**Mitigation**:
- Sync is opt-in and only activates when user explicitly starts it.
- mDNS discovery is limited to local network.
- **Residual risk**: Sync traffic is not encrypted. Sprint 16+ should add TLS or Noise Protocol for sync.

**Severity**: MEDIUM

### 5.3 Malware on Device (T3)

**Threat**: Malware running as the user reads the unlocked database or extracts the passphrase from memory.  
**Impact**: Full data disclosure.  
**Likelihood**: Low-Medium.  
**Mitigation**:
- Passphrase is held in memory only for the minimum duration needed.
- Database is re-encrypted and plaintext file is deleted when the app closes via `RunEvent::ExitRequested` handler.
- On startup with encryption enabled, an in-memory placeholder DB is used until the user unlocks with their passphrase.
- **Residual risk**: While the app is running with an unlocked encrypted database, the plain DB file exists on disk. Malware with file access can read it.

**Severity**: HIGH (while unlocked) — accepted risk for local-first apps.

### 5.4 Tauri IPC Injection (T4)

**Threat**: Malicious input from the WebView triggers unintended Rust commands.  
**Impact**: Arbitrary database operations, file access.  
**Likelihood**: Low.  
**Mitigation**:
- All Tauri commands are explicitly registered in `generate_handler!`.
- No `eval` or dynamic command dispatch.
- Input validation on all command parameters.
- ProseMirror content is structured JSON, not arbitrary HTML.

**Severity**: LOW

### 5.5 ProseMirror XSS (T5)

**Threat**: Malicious content in notes renders as executable HTML in the editor.  
**Impact**: Code execution in WebView context.  
**Likelihood**: Low.  
**Mitigation**:
- ProseMirror uses a schema-based document model. Only whitelisted node types and marks are rendered.
- HTML export uses `escape_html()` for all text content.
- No `v-html` or raw HTML rendering in the editor.

**Severity**: LOW

### 5.6 Share Bundle Brute-Force (T6)

**Threat**: Attacker obtains an encrypted share bundle and brute-forces the passphrase.  
**Impact**: Disclosure of shared pages.  
**Likelihood**: Medium (if weak passphrase).  
**Mitigation**:
- AES-256-GCM with per-bundle salt and nonce.
- Key derivation uses iterative SHA-256 with 100,000 iterations and CSPRNG-generated salts (hardened from previous single-pass SHA-256).
- **Residual risk**: Not a proper KDF (PBKDF2/Argon2). Still vulnerable to GPU brute-force with weak passphrases, but significantly harder than before.

**Severity**: MEDIUM — will be improved with PBKDF2 in a future sprint.

### 5.7 Workspace Registry Tampering (T7)

**Threat**: Attacker modifies `workspaces.json` to point a workspace to a malicious database.  
**Impact**: Data injection, potential confusion attacks.  
**Likelihood**: Low.  
**Mitigation**:
- Workspaces are user-created. The registry is not automatically processed from external sources.
- Database files are opened read-write; a malicious DB could contain crafted data but cannot execute code.

**Severity**: LOW

## 6. Security Controls Summary

| Control | Status | Sprint |
|---------|--------|--------|
| Database encryption at rest (AES-256-GCM) | ✅ Implemented | 14 |
| Encrypted share bundles | ✅ Implemented | 13 |
| Encrypted workspace export | ✅ Implemented | 14 |
| Passphrase management (enable/change/disable) | ✅ Implemented | 14 |
| Secure delete (overwrite before delete) | ✅ Implemented | 15 |
| ProseMirror schema validation | ✅ Implemented | 1-4 |
| Tauri IPC command whitelisting | ✅ Implemented | 1 |
| LAN sync (opt-in, mDNS) | ✅ Implemented | 12 |
| Re-encrypt on shutdown (plaintext DB cleanup) | ✅ Implemented | Audit |
| Encryption unlock gate (frontend) | ✅ Implemented | Audit |
| Content Security Policy (CSP) | ✅ Implemented | Audit |
| Sync message size cap (100MB) | ✅ Implemented | Audit |
| DB transaction wrapping (atomicity) | ✅ Implemented | Audit |
| Revision pruning (last 50 per page) | ✅ Implemented | Audit |
| Cargo audit in CI | ✅ Implemented | Audit |
| Iterative key derivation (100K rounds) | ✅ Implemented | Audit |
| CSPRNG for salt/nonce (getrandom) | ✅ Implemented | Audit |
| Encrypted sync transport | ❌ Not implemented | Future |
| PBKDF2/Argon2 key derivation | ❌ Not implemented | Future |
| Auto-lock timeout | ❌ Not implemented | Future |
| Recovery key mechanism | ❌ Not implemented | Future |

## 7. Assumptions

1. The user's machine is not compromised at the OS/kernel level.
2. The user chooses a strong passphrase for encryption (≥12 characters, mixed case, numbers, symbols).
3. Tauri and its dependencies are trusted (supply chain risk is accepted).
4. Physical access to a powered-off, encrypted machine does not yield data.
5. The user understands that enabling encryption is irreversible without the passphrase.

## 8. Future Work

- **PBKDF2/Argon2**: Replace SHA-256 key derivation with a proper KDF.
- **Encrypted sync transport**: Add TLS or Noise Protocol for LAN sync.
- **Auto-lock**: Re-lock the database after a period of inactivity.
- **Recovery key**: Generate a recovery key during encryption setup.
- **WAL encryption**: Encrypt WAL/SHM files or switch to journal mode that doesn't create sidecar files.
- **Secure memory zeroing**: Zero passphrase buffers in memory after use.
