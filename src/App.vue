<template>
  <div class="app-container">
    <div class="header">
      <h1>NeCd</h1>
      <span class="version">v1.1.2</span>
    </div>
    
    <div class="main-content">
      <!-- 左パネル（設定） -->
      <div class="settings-panel">
        <!-- URL入力 -->
        <div class="input-group">
          <input 
            type="text" 
            v-model="url" 
            placeholder="ダウンロード対象のURL" 
            class="url-input"
            @keyup.enter="executeDownloadHandler"
          >
          <button @click="pasteUrl" class="paste-button" title="クリップボードから貼り付け">
            📋
          </button>
        </div>

        <!-- 保存先フォルダ -->
        <div class="input-group">
          <input 
            type="text" 
            v-model="outputDirectory" 
            placeholder="保存先フォルダ" 
            class="directory-input"
            readonly
          >
          <button @click="selectDirectoryHandler" class="select-button">
            フォルダ選択
          </button>
        </div>

        <!-- 形式と品質 -->
        <div class="format-quality-group">
          <select v-model="format" @change="handleFormatChange" class="format-select">
            <option value="mp4">mp4</option>
            <option value="mp3">mp3</option>
            <option value="mkv">mkv</option>
            <option value="opus">opus</option>
            <option value="flac">flac</option>
          </select>
          
          <select v-model="quality" class="quality-select">
            <option v-if="isVideoFormat" value="auto">自動</option>
            <option v-if="isVideoFormat" value="2160">4K</option>
            <option v-if="isVideoFormat" value="1440">2K</option>
            <option v-if="isVideoFormat" value="1080">Full HD</option>
            <option v-if="isVideoFormat" value="720">HD</option>
            <option v-if="isMP3Format" value="auto">自動</option>
            <option v-if="isMP3Format" value="320k">320kbps</option>
            <option v-if="isMP3Format" value="256k">256kbps</option>
            <option v-if="isMP3Format" value="192k">192kbps</option>
            <option v-if="isMP3Format" value="128k">128kbps</option>
            <option v-if="!isVideoFormat && !isMP3Format" value="auto">自動</option>
          </select>
        </div>

        <!-- Cookie設定 -->
        <div class="cookie-group">
          <select v-model="cookieSource" @change="handleCookieSourceChange" class="cookie-select">
            <option value="none">Cookieなし</option>
            <option value="file">ファイル</option>
            <option value="firefox">Firefox</option>
          </select>
          
          <div v-if="cookieSource === 'file'" class="cookie-file-group">
            <input 
              type="text" 
              v-model="cookieFilePath" 
              placeholder="Cookieファイル(.txt)" 
              class="cookie-input"
              readonly
            >
            <button @click="selectCookieFileHandler" class="select-button">
              ファイル選択
            </button>
          </div>
        </div>

        <!-- 同時接続数 -->
        <div class="input-group">
          <label for="concurrentConnections">同時接続数</label>
          <input 
            type="number" 
            v-model="concurrentConnections" 
            id="concurrentConnections"
            min="1" 
            max="16" 
            class="connections-input"
            placeholder="同時接続数 (0~16)"
          >
        </div>

        <!-- オプション -->
        <div class="options-group">
          <h3>オプション</h3>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="chapterEmbed">
            <span>チャプターを埋め込む</span>
          </label>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="playlistMode">
            <span>プレイリストモード</span>
          </label>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="thumbnailEmbed" @change="handleThumbnailChange">
            <span>サムネイルを埋め込む</span>
          </label>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="thumbnailCrop" :disabled="!thumbnailEmbed">
            <span>サムネイルをクロッピング</span>
          </label>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="compatibilityMode">
            <span>互換性重視</span>
          </label>
          
          <label class="checkbox-label">
            <input type="checkbox" v-model="hdrMode">
            <span>HDRを優先する</span>
          </label>
        </div>

        <!-- プログレスバー -->
        <div class="progress-container">
          <div class="progress-bar">
            <div 
              class="progress-fill" 
              :style="{ width: progress + '%' }"
              :class="{ 'error': isError }"
            ></div>
          </div>
        </div>

        <!-- 実行ボタン -->
        <button 
          @click="executeDownloadHandler" 
          :disabled="isDownloading" 
          class="download-button"
          :class="{ 'downloading': isDownloading }"
        >
          {{ isDownloading ? '実行中...' : '実行' }}
        </button>
        <div class="input-group">
          <button @click="installYtDlpHandler" v-if="!ytDlpInstalled" class="select-button">yt-dlp自動インストール</button>
          <span v-if="ytDlpInstallMessage">{{ ytDlpInstallMessage }}</span>
          <button @click="openOutputDirectory" class="select-button">保存先フォルダを開く</button>
        </div>
      </div>

      <!-- 右パネル（ログ） -->
      <div class="log-panel">
        <div class="log-header">
          <h3>📃 ダウンロードログ</h3>
        </div>
        <div class="log-content" ref="logContent">
          <div v-for="(log, index) in logs" :key="index" class="log-entry" :class="{ 'error': log.isError }">
            {{ log.message }}
          </div>
          <div v-if="logs.length === 0" class="log-placeholder">
            ここにログが表示されます
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, onUnmounted, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import {
  readClipboard,
  selectDirectory,
  selectFile,
  getDefaultDownloadDirectory,
  checkYtDlpInstalled,
  installYtDlp,
  openDirectory,
  executeDownload,
  saveSettings,
  loadSettings
} from './api'

// リアクティブな状態
const url = ref('')
const outputDirectory = ref('')
const format = ref('mp4')
const quality = ref('auto')
const cookieSource = ref('none')
const cookieFilePath = ref('')
const concurrentConnections = ref('3')
const chapterEmbed = ref(false)
const playlistMode = ref(false)
const thumbnailEmbed = ref(false)
const thumbnailCrop = ref(false)
const compatibilityMode = ref(false)
const hdrMode = ref(false)

const isDownloading = ref(false)
const progress = ref(0)
const isError = ref(false)
const logs = ref<Array<{ message: string; isError: boolean }>>([])
const logContent = ref<HTMLElement>()

// yt-dlpインストール状態
const ytDlpInstalled = ref(true)
const ytDlpInstallMessage = ref('')

// イベントリスナーの参照
const unsubscribe = ref<null | (() => void)>(null)

// 計算プロパティ
const isVideoFormat = computed(() => {
  return ['mp4', 'mkv'].includes(format.value)
})

const isMP3Format = computed(() => {
  return ['mp3'].includes(format.value)
})

const saveCurrentSettings = async () => {
  try {
    const settings = {
      outputDirectory: outputDirectory.value,
      format: format.value,
      quality: quality.value,
      cookieSource: cookieSource.value,
      cookieFilePath: cookieFilePath.value,
      concurrentConnections: concurrentConnections.value.toString(),
      chapterEmbed: chapterEmbed.value,
      playlistMode: playlistMode.value,
      thumbnailEmbed: thumbnailEmbed.value,
      thumbnailCrop: thumbnailCrop.value,
      compatibilityMode: compatibilityMode.value,
      hdrMode: hdrMode.value
    }
    await saveSettings(settings)
  } catch (e) {
    addLog('❌ 設定の保存に失敗しました', true)
  }
}

// url以外の設定値をまとめて監視
watch(
  [
    outputDirectory, format, quality, cookieSource, cookieFilePath,
    concurrentConnections, chapterEmbed, playlistMode, thumbnailEmbed,
    thumbnailCrop, compatibilityMode, hdrMode
  ],
  saveCurrentSettings,
  { deep: true }
)

// リアルタイムログ受信の設定
const setupRealTimeLogging = async () => {
  unsubscribe.value = await listen('download-log', (event) => {
    const log = event.payload as { message: string; is_error: boolean; progress?: number }
    
    // プログレス情報の処理
    if (log.progress !== undefined) {
      progress.value = log.progress
    }
    
    // プログレスメッセージの解析（フォールバック）
    if (log.message.startsWith('[DOWNLOADING]:')) {
      const match = log.message.match(/\[DOWNLOADING\]:\s*([\d.]+)%/)
      if (match) {
        progress.value = parseFloat(match[1])
      }
      // プログレスメッセージはログに追加しない
      return
    }
    
    // ログを追加
    addLog(log.message, log.is_error)
    
    // エラー状態を更新
    if (log.is_error) {
      isError.value = true
    }
  })
}

// メソッド
const pasteUrl = async () => {
  try {
    url.value = await readClipboard()
    addLog('📋 クリップボードからURLを貼り付けました')
  } catch (e) {
    addLog('❌ クリップボードの読み込みに失敗しました', true)
  }
}

const selectDirectoryHandler = async () => {
  try {
    const selected = await selectDirectory(outputDirectory.value)
    if (selected) {
      outputDirectory.value = selected
      addLog('📁 保存先フォルダを選択しました: ' + selected)
    } else {
      addLog('📁 フォルダ選択がキャンセルされました')
    }
  } catch (e) {
    addLog('❌ フォルダ選択に失敗しました', true)
  }
}

const selectCookieFileHandler = async () => {
  try {
    const selected = await selectFile()
    if (selected) {
      cookieFilePath.value = selected
      addLog('🍪 Cookieファイルを選択しました: ' + selected)
    } else {
      addLog('🍪 Cookieファイル選択がキャンセルされました')
    }
  } catch (e) {
    addLog('❌ Cookieファイル選択に失敗しました', true)
  }
}

const installYtDlpHandler = async () => {
  ytDlpInstallMessage.value = 'インストール中...'
  try {
    const msg = await installYtDlp()
    ytDlpInstallMessage.value = msg
    ytDlpInstalled.value = true
    addLog('✅ yt-dlpのインストールが完了しました')
  } catch (e) {
    ytDlpInstallMessage.value = '❌ インストールに失敗しました'
    addLog('❌ yt-dlpのインストールに失敗しました', true)
  }
}

const openOutputDirectory = async () => {
  try {
    await openDirectory(outputDirectory.value)
    addLog('📂 保存先フォルダを開きました')
  } catch (e) {
    addLog('❌ フォルダを開けませんでした', true)
  }
}

const handleFormatChange = () => {
  if (isVideoFormat.value) {
    quality.value = 'auto'
  } else {
    quality.value = 'auto'
  }
}

const handleCookieSourceChange = () => {
  if (cookieSource.value !== 'file') {
    cookieFilePath.value = ''
  }
}

const handleThumbnailChange = () => {
  if (!thumbnailEmbed.value) {
    thumbnailCrop.value = false
  }
}

const loadSavedSettings = async () => {
  try {
    const settings = await loadSettings()
    
    // 設定を適用
    if (settings.outputDirectory) {
      outputDirectory.value = settings.outputDirectory
    }
    if (settings.format) {
      format.value = settings.format
    }
    if (settings.quality) {
      quality.value = settings.quality
    }
    if (settings.cookieSource) {
      cookieSource.value = settings.cookieSource
    }
    if (settings.cookieFilePath) {
      cookieFilePath.value = settings.cookieFilePath
    }
    if (settings.concurrentConnections) {
      concurrentConnections.value = settings.concurrentConnections
    }
    if (settings.chapterEmbed !== undefined) {
      chapterEmbed.value = settings.chapterEmbed
    }
    if (settings.playlistMode !== undefined) {
      playlistMode.value = settings.playlistMode
    }
    if (settings.thumbnailEmbed !== undefined) {
      thumbnailEmbed.value = settings.thumbnailEmbed
    }
    if (settings.thumbnailCrop !== undefined) {
      thumbnailCrop.value = settings.thumbnailCrop
    }
    if (settings.compatibilityMode !== undefined) {
      compatibilityMode.value = settings.compatibilityMode
    }
    if (settings.hdrMode !== undefined) {
      hdrMode.value = settings.hdrMode
    }
    
    addLog('✅ すべての設定を読み込みました')
  } catch (e) {
    addLog('❌ 設定の読み込みに失敗しました: ' + e, true)
  }
}

const addLog = (message: string, isError = false) => {
  logs.value.push({ message, isError })
  scrollToBottom()
}

const scrollToBottom = async () => {
  await nextTick()
  if (logContent.value) {
    logContent.value.scrollTop = logContent.value.scrollHeight
  }
}

const executeDownloadHandler = async () => {
  if (!url.value.trim()) {
    addLog('❌ URLを入力してください', true)
    return
  }
  if (!outputDirectory.value) {
    addLog('❌ 保存先フォルダを選択してください', true)
    return
  }
  // yt-dlpのインストール確認
  try {
    ytDlpInstalled.value = await checkYtDlpInstalled()
    if (!ytDlpInstalled.value) {
      addLog('❌ yt-dlpがインストールされていません。', true)
      return
    }
    addLog('✅ yt-dlpがインストールされています')
  } catch (e) {
    addLog('❌ yt-dlpの確認に失敗しました', true)
    return
  }
  isDownloading.value = true
  progress.value = 0
  isError.value = false
  logs.value = []
  addLog('⏳ 開始しています...')
  try {
    const result = await executeDownload({
      url: url.value,
      format: format.value,
      quality: quality.value,
      outputDirectory: outputDirectory.value,
      cookieSource: cookieSource.value,
      cookieFilePath: cookieFilePath.value || null,
      concurrentConnections: concurrentConnections.value.toString(),
      playlistMode: playlistMode.value,
      thumbnailEmbed: thumbnailEmbed.value,
      thumbnailCrop: thumbnailCrop.value,
      chapterEmbed: chapterEmbed.value,
      compatibilityMode: compatibilityMode.value,
      hdrMode: hdrMode.value
    })
    if (result.is_error) {
      isError.value = true
      progress.value = 0
      addLog(result.message, true)
    } else {
      progress.value = 100
      addLog(result.message)
    }
  } catch (e) {
    isError.value = true
    progress.value = 0
    addLog(`❌ 予期せぬエラーが発生しました: ${e}`, true)
  } finally {
    isDownloading.value = false
  }
}

// 初期化
onMounted(async () => {
  try {
    // 1. 保存された設定を読み込み
    await loadSavedSettings()
    
    // 2. 出力ディレクトリが設定されていない場合のみデフォルトを設定
    if (!outputDirectory.value) {
      outputDirectory.value = await getDefaultDownloadDirectory()
      addLog('📁 デフォルトの保存先フォルダを設定しました: ' + outputDirectory.value)
    }
    
    // 3. yt-dlpのインストール確認
    ytDlpInstalled.value = await checkYtDlpInstalled()
    
    // 4. リアルタイムログ受信を設定
    await setupRealTimeLogging()
    
    addLog('🚀 アプリケーションの初期化が完了しました')
  } catch (e) {
    addLog('❌ 初期化中にエラーが発生しました: ' + e, true)
  }
})

// クリーンアップ
onUnmounted(() => {
  if (unsubscribe.value) {
    unsubscribe.value()
  }
})
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 12px;
  background-color: #f5f5f5;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.header h1 {
  margin: 0;
  font-size: 28px;
  font-weight: bold;
  color: #333;
}

.version {
  font-size: 12px;
  color: #666;
}

.main-content {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.settings-panel {
  width: 420px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.log-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 2px solid #e3f2fd;
  border-radius: 10px;
  background-color: white;
  overflow: hidden;
}

.log-header {
  padding: 12px;
  background-color: #f8f9fa;
  border-bottom: 1px solid #e9ecef;
}

.log-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: bold;
}

.log-content {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
}

.log-entry {
  margin-bottom: 4px;
  word-wrap: break-word;
}

.log-entry.error {
  color: #d32f2f;
}

.log-placeholder {
  color: #666;
  font-style: italic;
}

.input-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.url-input, .directory-input, .cookie-input, .connections-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.url-input:focus, .directory-input:focus, .cookie-input:focus, .connections-input:focus {
  outline: none;
  border-color: #2196f3;
  box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.2);
}

.paste-button, .select-button {
  padding: 8px 12px;
  background-color: #2196f3;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  white-space: nowrap;
}

.paste-button:hover, .select-button:hover {
  background-color: #1976d2;
}

.paste-button:disabled, .select-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.format-quality-group {
  display: flex;
  gap: 8px;
}

.format-select, .quality-select, .cookie-select {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  background-color: white;
}

.cookie-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cookie-file-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.options-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.options-group h3 {
  margin: 0 0 8px 0;
  font-size: 15px;
  font-weight: bold;
  color: #333;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.checkbox-label input[type="checkbox"]:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.progress-container {
  margin: 8px 0;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background-color: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: #4caf50;
  transition: width 0.3s ease;
}

.progress-fill.error {
  background-color: #f44336;
}

.download-button {
  padding: 16px;
  background-color: #2196f3;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 16px;
  font-weight: bold;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.download-button:hover:not(:disabled) {
  background-color: #1976d2;
}

.download-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.download-button.downloading {
  background-color: #ff9800;
}

/* スクロールバーのスタイル */
.settings-panel::-webkit-scrollbar,
.log-content::-webkit-scrollbar {
  width: 8px;
}

.settings-panel::-webkit-scrollbar-track,
.log-content::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.settings-panel::-webkit-scrollbar-thumb,
.log-content::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

.settings-panel::-webkit-scrollbar-thumb:hover,
.log-content::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>