# 900Notes Privacy Model

**Version**: 1.0  
**Date**: 2026-06-29  
**Status**: Active

## 1. Philosophy

900Notes is a **local-first** application. Your data lives on your machine. We do not run servers, we do not collect telemetry, and we do not have accounts. This document describes exactly what data exists, where it goes, and what stays private.

## 2. Data Inventory

### 2.1 What We Store

All data is stored locally on your machine in the application data directory:

| Data | Format | Location | Encrypted? |
|------|--------|----------|------------|
| Pages (title, content, metadata) | SQLite | `900notes.db` | Optional (Sprint 14) |
| Tags | SQLite | `900notes.db` | Optional |
| Page properties | SQLite | `900notes.db` | Optional |
| Attachments (images, PDFs, audio) | SQLite BLOBs | `900notes.db` | Optional |
| Page revisions (history) | SQLite | `900notes.db` | Optional |
| Links between pages | SQLite | `900notes.db` | Optional |
| Templates | SQLite | `900notes.db` | Optional |
| Saved searches & smart folders | SQLite | `900notes.db` | Optional |
| Settings (theme, language, font) | SQLite | `900notes.db` | Optional |
| Sync state (Automerge CRDT) | SQLite BLOB | `900notes.db` | Optional |
| Sync queue | SQLite | `900notes.db` | Optional |
| Audio notes | SQLite BLOBs | `900notes.db` | Optional |
| Workspace registry | JSON | `workspaces.json` | No |
| Encryption metadata | JSON | `900notes.db.meta` | N/A (salt + nonce only) |

### 2.2 What We Do NOT Store

- **No accounts**: No username, email, or password is required.
- **No telemetry**: We do not collect usage analytics, crash reports, or performance data.
- **No cloud storage**: Notes are never uploaded to any server.
- **No tracking**: No cookies, no tracking pixels, no advertising SDKs.
- **No login tokens**: No authentication tokens are stored or transmitted.

## 3. Data Flow

### 3.1 Normal Operation (No Sync)

```
User types in editor
       │
       ▼
Svelte UI (WebView)
       │  Tauri IPC (local, in-process)
       ▼
Rust Backend
       │  SQLite query
       ▼
900notes.db (local disk)
```

**No data leaves the machine.** All processing is local.

### 3.2 LAN Sync (Opt-In)

When the user explicitly enables sync:

```
900notes.db (Machine A)
       │
       ├──► CRDT Document (Automerge)
       │         │
       │         ▼
       │    mDNS Discovery (LAN, UDP)
       │         │
       │         ▼
       │    Peer Machine B (900Notes)
       │         │
       │         ▼
       │    900notes.db (Machine B)
       │
       └──► Sync Queue (tracks pending ops)
```

**Data stays on the LAN.** Sync uses mDNS (UDP multicast) for discovery on the local network and encrypted TCP handshakes for page exchange. No data is sent to any external server.

**Privacy implications**:
- Other devices on the same LAN can see that 900Notes is running (mDNS announcement).
- Page content is encrypted in transit with AES-256-GCM using the pairing secret entered when sync is started.
- Only devices running 900Notes with sync enabled and the same pairing secret can complete a sync exchange.
- Weak or reused pairing secrets reduce confidentiality; use a long, unique secret for each trusted device group.

### 3.3 Share Bundles (Export/Import)

```
User selects pages + passphrase
       │
       ▼
Pages → JSON → AES-256-GCM encrypt → .json file (disk)
       │
       ▼
User shares file out-of-band (email, USB, etc.)
       │
       ▼
Recipient: file + passphrase → decrypt → import to their DB
```

**The share bundle is encrypted.** Anyone with the file but not the passphrase cannot read it. The passphrase is shared out-of-band by the user.

### 3.4 HTML Export

```
Page content → ProseMirror → HTML → .html file (disk)
```

**HTML export is not encrypted.** The exported HTML file is plain text. Do not export sensitive pages as HTML if you don't want them readable.

### 3.5 Encrypted Workspace Export

```
All pages → JSON → AES-256-GCM encrypt → base64 → .enc file (disk)
```

**The export is encrypted.** Requires the passphrase to import.

## 4. What Stays Local

- ✅ All page content
- ✅ All tags and properties
- ✅ All attachments
- ✅ All settings
- ✅ All search history
- ✅ All page revisions
- ✅ Encryption passphrases (never written to disk)
- ✅ Workspace registry

## 5. What Can Leave the Machine

Only when the user explicitly takes action:

| Action | What leaves | Where it goes | Encrypted? |
|--------|-------------|---------------|------------|
| Enable LAN sync | Page content (sync handshake) | LAN peers | Yes (AES-256-GCM with pairing secret) |
| Export share bundle | Selected pages | File on disk | Yes (AES-256-GCM) |
| Export HTML | Single page | File on disk | No |
| Export encrypted workspace | All pages | File on disk | Yes (AES-256-GCM) |
| Export workspace (plain) | All pages | File on disk | No |

**No data ever leaves the machine automatically.** Every export/sync action is user-initiated.

## 6. Encryption Details

### Database at Rest (Sprint 14)

- **Algorithm**: AES-256-GCM
- **Key derivation**: Iterative SHA-256(passphrase + 32-byte salt), 100,000 rounds
- **Storage**: Encrypted DB in `.enc` file, salt + nonce in `.meta` JSON file
- **Passphrase**: Not stored anywhere. Held in memory only while the database is unlocked.
- **Failure mode**: If the passphrase is lost, data is irrecoverable.

### Share Bundles (Sprint 13)

- **Algorithm**: AES-256-GCM
- **Key derivation**: Iterative SHA-256(passphrase + 32-byte salt), 100,000 rounds
- **Per-bundle**: Unique salt and nonce for every export
- **Passphrase**: Chosen by user at export time, shared out-of-band

### Limitations

1. **WAL files**: While the database is in use, Write-Ahead Log (WAL) files contain unencrypted data. These are deleted when the database is closed or checkpointed.
2. **Plain DB while unlocked**: When encryption is enabled and the database is unlocked, the plain `.db` file exists on disk. It is removed when the app closes (if encryption is re-applied).
3. **Memory**: The passphrase and decrypted data exist in RAM while the app is running. Memory dumps could expose them.
4. **Key derivation**: SHA-256 is not a proper KDF. Future versions will use PBKDF2 or Argon2.

## 7. User Controls

| Control | How | Effect |
|---------|-----|--------|
| Enable encryption | Settings → Security → Enable | Database encrypted at rest |
| Disable encryption | Settings → Security → Disable | Database decrypted, stored in plain text |
| Change passphrase | Settings → Security → Change Passphrase | Re-encrypts with new passphrase |
| Disable sync | Settings → Sync → Stop | No data leaves the machine |
| Delete data | Settings → Data → Export then delete, or delete pages | Data removed from DB |
| Secure delete | Per-page or bulk | Overwrites content before deletion (Sprint 15) |
| Export data | Settings → Data → Export | Full workspace export to file |
| Encrypted export | Settings → Security → Export Encrypted | Encrypted full workspace export |

## 8. Third-Party Dependencies

900Notes uses the following open-source dependencies. None of them transmit data externally:

| Dependency | Purpose | Network access? |
|------------|---------|-----------------|
| Tauri | App framework (WebView + Rust) | No (unless app uses network features) |
| rusqlite | SQLite database | No |
| Automerge | CRDT for sync | No (in-process) |
| aes-gcm | Encryption | No |
| sha2 | Hashing | No |
| mdns-sd | LAN device discovery | Yes (LAN only, UDP multicast) |
| serde / serde_json | Serialization | No |
| uuid | ID generation | No |
| chrono | Timestamps | No |

PDF export is implemented by a small built-in text PDF writer in `src-tauri/src/services/pdf.rs`; it does not require a third-party PDF crate.

**No analytics SDKs. No advertising SDKs. No tracking libraries.**

## 9. Compliance Notes

- **GDPR**: 900Notes does not process personal data on any server. All data is under the user's control. No data subject access requests are needed because there is no data controller.
- **CCPA**: No data is sold or shared with third parties.
- **HIPAA**: 900Notes is not HIPAA-certified. If storing health information, users should enable encryption and use strong passphrases.

## 10. Summary

900Notes is designed for privacy by default:

- **Local-first**: Your data never leaves your machine unless you explicitly export or sync it.
- **No accounts, no servers, no telemetry**: We have nothing to leak.
- **Optional encryption**: AES-256-GCM encryption at rest for sensitive data.
- **User control**: Every data-sharing action is explicit and user-initiated.
