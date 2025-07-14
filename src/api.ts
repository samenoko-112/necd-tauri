import { invoke } from '@tauri-apps/api/core'

export type DownloadOptions = {
  url: string
  format: string
  quality: string
  outputDirectory: string
  cookieSource: string
  cookieFilePath: string | null
  concurrentConnections: string
  playlistMode: boolean
  thumbnailEmbed: boolean
  thumbnailCrop: boolean
  chapterEmbed: boolean
  compatibilityMode: boolean
  hdrMode: boolean
}

export type DownloadProgress = {
  message: string
  progress?: number
  is_error: boolean
}

export async function readClipboard(): Promise<string> {
  return await invoke('read_clipboard')
}

export async function selectDirectory(defaultPath?: string): Promise<string | null> {
  return await invoke('select_directory', { defaultPath })
}

export async function selectFile(): Promise<string | null> {
  return await invoke('select_file', {
    filters: [{ name: 'Cookie Files', extensions: ['txt'] }]
  })
}

export async function getDefaultDownloadDirectory(): Promise<string> {
  return await invoke('get_default_download_directory')
}

export async function checkYtDlpInstalled(): Promise<boolean> {
  return await invoke('check_yt_dlp_installed')
}

export async function installYtDlp(): Promise<string> {
  return await invoke('install_yt_dlp')
}

export async function openDirectory(path: string): Promise<void> {
  return await invoke('open_directory', { path })
}

function toSnakeCase(obj: any): any {
  if (Array.isArray(obj)) {
    return obj.map(toSnakeCase)
  } else if (obj && typeof obj === 'object') {
    return Object.fromEntries(
      Object.entries(obj).map(([k, v]) => [
        k.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`),
        toSnakeCase(v)
      ])
    )
  }
  return obj
}

export async function executeDownload(options: DownloadOptions): Promise<DownloadProgress> {
  const snakeOptions = toSnakeCase(options)
  return await invoke('execute_download', { options: snakeOptions })
}

export async function saveSettings(settings: any): Promise<void> {
  return await invoke('save_settings', { settings })
}

export async function loadSettings(): Promise<any> {
  return await invoke('load_settings')
} 