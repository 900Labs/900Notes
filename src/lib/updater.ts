import { check, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export type UpdateStatus =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'available'; version: string; body: string }
  | { state: 'not-available' }
  | { state: 'downloading'; progress: number }
  | { state: 'restarting' }
  | { state: 'done' }
  | { state: 'error'; message: string }

export interface PendingUpdate {
  version: string
  body: string
}

export async function checkForUpdate(): Promise<{ status: UpdateStatus; update: PendingUpdate | null }> {
  try {
    const update = await check()
    if (update) {
      return {
        status: {
          state: 'available',
          version: update.version,
          body: update.body ?? '',
        },
        update: { version: update.version, body: update.body ?? '' },
      }
    }
    return { status: { state: 'not-available' }, update: null }
  } catch (error) {
    return {
      status: { state: 'error', message: error instanceof Error ? error.message : String(error) },
      update: null,
    }
  }
}

export async function downloadAndInstallUpdate(
  update: PendingUpdate,
  onProgress: (progress: number) => void,
): Promise<UpdateStatus> {
  try {
    const latest = await check()
    if (!latest) {
      return { state: 'not-available' }
    }

    let totalContentLength = 0
    let downloaded = 0

    await latest.downloadAndInstall((event: DownloadEvent) => {
      if (event.event === 'Started' && event.data.contentLength) {
        totalContentLength = event.data.contentLength
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength
        if (totalContentLength > 0) {
          onProgress(Math.round((downloaded / totalContentLength) * 100))
        }
      }
    })

    await relaunch()
    return { state: 'done' }
  } catch (error) {
    return { state: 'error', message: error instanceof Error ? error.message : String(error) }
  }
}
