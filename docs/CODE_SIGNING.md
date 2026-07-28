# Code Signing

Status: Workflow scaffolded in `.github/workflows/release.yml`. Both signing paths are currently dormant - no credentials are configured. The release workflow produces unsigned artifacts identical to the manual release gate, and the `README.md` checksum-verification flow remains the trust mechanism for users.

Prerequisites to activate each path:

- macOS requires an Apple Developer Program membership ($99/yr). Notarization has no substitute - ad-hoc signing (`codesign -s -`) does not satisfy Gatekeeper and is not used. Enroll at developer.apple.com, then configure the six `APPLE_*` secrets below.
- Windows requires a code signing certificate (OV is sufficient for CI). Acquire one from a CA, then configure the two `WINDOWS_*` secrets below.

Neither path depends on the other. Activate them independently whenever each credential is available.

This covers macOS (Developer ID Application certificate + notarization) and Windows (code signing certificate). Linux package signing uses distribution-specific keys (GPG for DEB/RPM repositories) and is out of scope here.

## How the workflow branches

Each platform job runs in the `release` environment (Settings > Environments > release) and detects whether its credentials are present at runtime. It then takes one of two paths:

- Credentials present: builds, signs, notarizes (macOS only), and labels the artifact `signed`.
- Credentials absent: builds unsigned and labels the artifact `unsigned`.

The `release` environment can be gated behind a required reviewer (Environments > release > Required reviewers) so signing keys are only injected after explicit approval. Contributors can run the workflow from a branch without secrets and still get usable unsigned installers for manual testing.

## Required environment secrets

Create a `release` environment (Settings > Environments > New environment, name it `release`) and configure these secrets there. Environment secrets are only visible to jobs that declare `environment: release`, which limits the blast radius of each key. Never commit certificate material to the repository. The public release gate scans tracked files for private keys and high-confidence secret strings.

### macOS

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE` | Base64-encoded export of the Developer ID Application certificate as a `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password protecting the `.p12` export |
| `APPLE_SIGNING_IDENTITY` | Identity name, e.g. `Developer ID Application: 900 Labs (TEAMID)` |
| `APPLE_ID` | Apple ID of the notarization account |
| `APPLE_PASSWORD` | App-specific password created at appleid.apple.com (not the account password) |
| `APPLE_TEAM_ID` | Developer Program team ID |

Tauri imports the `.p12` into a temporary keychain, signs the `.app` and `.dmg`, and submits them to Apple's notary service via `notarytool`. Stapling happens automatically when notarization succeeds.

### Windows

| Secret | Value |
|--------|-------|
| `WINDOWS_CERTIFICATE` | Base64-encoded `.pfx` code signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password protecting the `.pfx` |

The Windows job imports the certificate into the runner's CurrentUser store, writes a temporary Tauri config overlay that points at the certificate thumbprint, and builds with `--config`. Tauri signs the main executable and the NSIS/MSI installers with `signtool`, using the DigiCert timestamp server.

## Preparing credentials

### Export the Apple Developer ID certificate

Run on a Mac enrolled in the developer account with the private key installed:

```bash
# 1. In Keychain Access, export the "Developer ID Application: ..." identity as a .p12.
# 2. Base64-encode it for the repository secret:
base64 -i developer-id.p12 -o developer-id.b64
```

Create an app-specific password for notarization at https://appleid.apple.com under Sign-In and Security > App-Specific Passwords. The team ID is shown under Membership > Membership Details in the developer portal.

### Export the Windows certificate

```powershell
# 1. In certmgr.msc, export the code signing certificate with its private key as a .pfx.
# 2. Base64-encode it for the repository secret:
certutil -encode codesign.pfx codesign.b64
# The base64 payload is the content between the BEGIN/END markers.
```

OV certificates can be exported and reused across CI runners. EV certificates that require a hardware token cannot be used in this workflow without a remote signing service; use an OV certificate for CI.

## Triggering a signed release

1. Confirm the secrets above are set in the `release` environment.
2. Tag the reviewed commit:

```bash
git tag -a v1.7.0 -m "900Notes 1.7.0"
git push origin v1.7.0
```

3. The `Release` workflow runs once per tag. The `release` environment pauses for reviewer approval if required reviewers are configured. Download the `900notes-macos-signed`, `900notes-windows-signed`, and `900notes-linux-unsigned` artifacts.
4. Record the published SHA-256 checksums (each artifact bundle includes a `SHA256SUMS` file) in the release notes.

Manual overrides are available via Actions > Release > Run workflow, with an optional `ref` input.

## Verification

After a signed build, confirm the signatures before publishing:

```bash
# macOS: ad-hoc or identity signature
codesign -dv --verbose=4 900Notes.app
spctl -a -vvv -t install 900Notes.app   # should report "notarized"

# Windows: use signtool from the Windows SDK
signtool verify /pa /v 900Notes-Setup.exe
```

## Out of scope

- Linux distribution signing (GPG keys for APT/YUM repositories) is handled separately at publish time.

## App auto-updater

The Tauri updater is wired into the app. It is independent of Apple and Windows code signing: users get seamless in-app updates over GitHub Releases without needing a paid developer account.

**How it works**: The app checks the endpoint configured in `tauri.conf.json` (a `latest.json` manifest hosted on GitHub Releases). When a newer version exists, the user sees it in Settings > About, downloads it, and the app installs and relaunches. Each update bundle is signed with a minisign keypair that the updater verifies before installing, so only maintainers who hold the private key can publish updates.

**Keypair**: Already generated. The public key is embedded in `src-tauri/tauri.conf.json`. The private key lives at `~/.tauri/900notes_updater.key` on the maintainer machine (empty password). Store it as the `TAURI_SIGNING_PRIVATE_KEY` environment secret in the `release` environment so CI can sign release bundles. Do not lose the private key: without it, no future updates can be signed, and the updater will reject them.

**To activate**:

1. Create the `release` environment if it does not exist yet (Settings > Environments > New environment).

2. Store the private key content as a secret named `TAURI_SIGNING_PRIVATE_KEY` in the `release` environment. Read it from the key file:

   ```bash
   cat ~/.tauri/900notes_updater.key
   ```

3. If the key has a password, also set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. The generated key uses an empty password, so this secret can be omitted.

4. The `Release` workflow reads the secret from the `release` environment. Each platform job builds with `createUpdaterArtifacts: true`, so Tauri emits `.tar.gz`/`.nsis.zip` update bundles and their `.sig` files. The `publish` job then assembles `latest.json` and creates a GitHub Release with all artifacts attached.

**To generate a new keypair** (only if the private key is lost or compromised):

```bash
cargo tauri signer generate -w ~/.tauri/900notes_updater.key -p "yourpassword" --ci
```

Replace the `pubkey` in `src-tauri/tauri.conf.json` with the new public key and update the `release` environment secret.
