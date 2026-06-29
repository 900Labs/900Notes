# Security Policy

## Reporting a Vulnerability

To report a vulnerability, email **security@900labs.com**.

Please include:
- A description of the vulnerability
- Steps to reproduce
- The operating system and version you tested on
- The 900Notes version (found in Settings → About)

We will acknowledge receipt within 48 hours and provide a timeline for a fix.

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
