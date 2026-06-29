# Documentation Index

A sorted guide to public docs, runbooks, and references for 900Notes.

## Build and Use

- [Installation & Build](../README.md#installation) — prerequisites, build from source, run in dev mode
- [Architecture Overview](ARCHITECTURE.md) — system design, data flow, data model, offline model
- [Roadmap](ROADMAP.md) — MVP scope and post-MVP phases
- [Sprint Plan](SPRINT_PLAN.md) — 20 sprints across 7 post-MVP phases

## Developer References

- [API Reference](API.md) — complete Tauri command reference (25 commands)
- [Development Guide](DEVELOPMENT.md) — local setup, project structure, coding standards, testing
- [Database Schema](DATABASE.md) — SQLite tables, triggers, FTS5 index, migrations
- [Editor Schema](EDITOR.md) — ProseMirror schema, block types, marks, wiki links
- [i18n Guide](I18N.md) — add languages, locale formatting, RTL support
- [Quality Gate](QUALITY_GATE.md) — required pre-merge validation baseline

## Governance

- [Contributing](../CONTRIBUTING.md) — setup, coding standards, PR process
- [Security Policy](../SECURITY.md) — vulnerability reporting and scope
- [ADR Template](adr/TEMPLATE.md) — architecture decision record template
- [Sprint Review Log](SPRINT_REVIEW_LOG.md) — build → review cycle results per sprint

## Planned Documentation (Post-MVP)

- Deployment Guide — platform builds, data locations, distribution
- Release Runbook — tagged release flow, artifacts, checksums
- Public Release Checklist — repository visibility, privacy, documentation readiness
- Threat Model — formal threat model (Sprint 15)
- Privacy Model — formal privacy model (Sprint 15)
- Plugin API — plugin development guide (Sprint 19)
- Sync Protocol — CRDT sync protocol specification (Sprint 12)
