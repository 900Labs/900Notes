# ADR-0003: Use SQLite with FTS5 for Data Storage and Search

- **Status**: accepted
- **Date**: 2026-06-28
- **Deciders**: 900 Labs team
- **Context**: MVP

---

## Context

900Notes needs a local data store that supports structured data (pages, tags, links), hierarchical relationships (page tree), and full-text search. The store must work offline, require no server process, and be embedded in the application binary.

Constraints:
- Must work offline with no server process
- Must support full-text search across page titles and content
- Must handle hierarchical data (nested pages)
- Must be embeddable (no external database server)
- Must be reliable and ACID-compliant
- Single-file storage for easy backup and portability

## Decision

Use **SQLite** with the **FTS5** extension for data storage and full-text search. SQLite is embedded via the `rusqlite` crate with the `bundled` feature (SQLite is compiled into the binary).

## Consequences

### Positive
- Single-file database (`900notes.db`) - trivial to backup, copy, and export
- ACID-compliant with WAL journal mode for concurrent reads
- FTS5 provides high-quality full-text search with Unicode tokenization
- Triggers keep the FTS index in sync automatically
- No server process, no configuration, no network
- Mature, battle-tested, used by virtually every application
- `rusqlite` with `bundled` feature means no system SQLite dependency

### Negative
- SQLite is not designed for concurrent writes from multiple processes (fine for a single-user desktop app)
- FTS5 tokenizer (`unicode61`) may not handle all languages perfectly (e.g., CJK languages need custom tokenizers - post-MVP)
- No built-in encryption (post-MVP: Sprint 14 will add AES-256 encryption at the application layer)

### Neutral
- The database file location varies by platform (see DEVELOPMENT.md)
- Schema migrations are handled in code (no separate migration tool needed for MVP)

## Alternatives Considered

1. **JSON files** - Rejected because full-text search would require loading all files into memory. No relational queries for the page tree or link graph.

2. **PostgreSQL embedded** - Rejected because PostgreSQL cannot be embedded. It requires a server process, which violates the offline-first, no-server constraint.

3. **DuckDB** - Considered for its analytical capabilities, but rejected because it's optimized for OLAP workloads. SQLite is better suited for OLTP (point queries, single-row updates).

4. **LMDB / sled** - Rejected because they are key-value stores, not relational databases. Full-text search and hierarchical queries would require significant custom code.

5. **Tantivy (search-only)** - Considered for search, but rejected to avoid maintaining two data stores. FTS5 is sufficient for the MVP and keeps everything in a single file.

## References

- [SQLite documentation](https://www.sqlite.org/docs.html)
- [FTS5 documentation](https://www.sqlite.org/fts5.html)
- [rusqlite crate](https://crates.io/crates/rusqlite)
