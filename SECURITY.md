# Security Policy

## Reporting a Vulnerability

Use GitHub's private vulnerability reporting for this repository. If that route is unavailable, email **security@900labs.com**.

Please include:
- A description of the vulnerability
- Steps to reproduce
- The operating system and version you tested on
- The 900Notes version found in Settings > About

Reports are reviewed as maintainer availability permits. We will share status and disclosure timing when we can do so safely.

## Scope

- 900Notes desktop application (this repository)
- The SQLite database layer
- The Tauri IPC boundary
- The ProseMirror editor sanitization layer

## Out of Scope

- Vulnerabilities in third-party dependencies (report upstream)
- Issues that require physical access to the user's machine
- Social engineering attacks

## Disclosure

We follow responsible disclosure. Once a fix is released, we will publish a security advisory on GitHub.
