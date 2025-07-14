use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;
use std::env;
use std::fs;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub url: String,
    pub format: String,
    pub quality: String,
    pub output_directory: String,
    pub cookie_source: String,
    pub cookie_file_path: Option<String>,
    pub concurrent_connections: String,
    pub playlist_mode: bool,
    pub thumbnail_embed: bool,
    pub thumbnail_crop: bool,
    pub chapter_embed: bool,
    pub compatibility_mode: bool,
    pub hdr_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub message: String,
    pub progress: Option<f64>,
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RealTimeLog {
    pub message: String,
    pub is_error: bool,
    pub progress: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[tauri::command]
async fn execute_download(window: tauri::Window, options: DownloadOptions) -> Result<DownloadProgress, String> {
    // Windows環境での文字エンコーディング設定
    #[cfg(target_os = "windows")]
    {
        env::set_var("PYTHONIOENCODING", "utf-8");
        env::set_var("PYTHONLEGACYWINDOWSSTDIO", "utf-8");
        env::set_var("PYTHONUTF8", "1");
    }
    
    let mut command = TokioCommand::new("yt-dlp");
    
    // 基本オプション
    command.args(&[
        "--newline",
        &options.url,
        "--embed-metadata",
        "--add-metadata",
        "--default-search", "ytsearch",
        "--progress-template", "[DOWNLOADING]:%(progress._percent_str)s",
        "--add-header", "Accept-Language:ja-JP",
        "--extractor-args", "youtube:lang=ja",
        "--no-warnings",
        "--encoding", "utf-8"
    ]);

    // Cookie設定
    match options.cookie_source.as_str() {
        "file" => {
            if let Some(cookie_path) = options.cookie_file_path {
                command.args(&["--cookies", &cookie_path]);
            }
        }
        "firefox" => {
            command.args(&["--cookies-from-browser", "firefox"]);
        }
        _ => {}
    }

    // 出力形式と品質設定
    match options.format.as_str() {
        "mp4" => {
            command.args(&["--merge-output-format", "mp4"]);
            if options.quality != "auto" {
                if options.compatibility_mode {
                    command.args(&["-f", &format!("bestvideo[height<={}][vcodec^=avc1]+bestaudio[ext=m4a]/best[height<={}][vcodec^=avc1]/best[height<={}]", options.quality, options.quality, options.quality)]);
                } else {
                    command.args(&["-f", &format!("bestvideo[height<={}]+bestaudio[ext=m4a]/best[height<={}]", options.quality, options.quality)]);
                }
            } else {
                if options.compatibility_mode {
                    command.args(&["-f", "bestvideo[vcodec^=avc1]+bestaudio[ext=m4a]/best[vcodec^=avc1]/best"]);
                } else {
                    command.args(&["-f", "bestvideo+bestaudio[ext=m4a]/best"]);
                }
            }
        }
        "mkv" => {
            command.args(&["--merge-output-format", "mkv"]);
            if options.quality != "auto" {
                if options.compatibility_mode {
                    command.args(&["-f", &format!("bestvideo[height<={}][vcodec^=avc1]+bestaudio[ext=m4a]/best[height<={}][vcodec^=avc1]/best[height<={}]", options.quality, options.quality, options.quality)]);
                } else {
                    command.args(&["-f", &format!("bestvideo[height<={}]+bestaudio[ext=m4a]/best[height<={}]", options.quality, options.quality)]);
                }
            } else {
                if options.compatibility_mode {
                    command.args(&["-f", "bestvideo[vcodec^=avc1]+bestaudio[ext=m4a]/best[vcodec^=avc1]/best"]);
                } else {
                    command.args(&["-f", "bestvideo+bestaudio[ext=m4a]/best"]);
                }
            }
        }
        "mp3" => {
            command.args(&["-f", "bestaudio", "-x", "--audio-format", "mp3"]);
            if options.quality != "auto" {
                command.args(&["--audio-quality", &options.quality]);
            } else {
                command.args(&["--audio-quality", "0"]);
            }
        }
        _ => {
            command.args(&["-f", "bestaudio", "-x", "--audio-format", &options.format, "--audio-quality", "0"]);
        }
    }

    // HDR設定
    if options.hdr_mode {
        command.args(&["--format-sort", "hdr,res,codec,ext,size"]);
    }

    // 追加オプション
    if options.chapter_embed {
        command.args(&["--embed-chapters", "--add-chapters"]);
    }

    if options.concurrent_connections != "" && options.concurrent_connections != "0" {
        command.args(&["-N", &options.concurrent_connections]);
    }

    // 出力パス設定
    if options.playlist_mode {
        command.args(&["-o", &format!("{}/%(playlist_title)s/%(playlist_index)03d_%(title).100s.%(ext)s", options.output_directory)]);
    } else {
        command.args(&["-o", &format!("{}/%(title).100s.%(ext)s", options.output_directory)]);
    }

    // サムネイル設定
    if options.thumbnail_embed {
        command.args(&["--embed-thumbnail", "--convert-thumbnails", "jpg"]);
        if options.thumbnail_crop {
            command.args(&["--ppa", "ThumbnailsConvertor:-qmin 1 -q:v 1 -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\""]);
        }
    }

    // コマンド実行（リアルタイム出力）
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONLEGACYWINDOWSSTDIO", "utf-8")
        .spawn()
        .map_err(|e| format!("コマンド実行エラー: {}", e))?;

    let stdout = child.stdout.take().ok_or("stdout取得エラー")?;
    let stderr = child.stderr.take().ok_or("stderr取得エラー")?;

    // リアルタイムログ送信用のタスク
    let window_handle = window.clone();
    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            // プログレス情報の解析
            let progress = if line.contains("[DOWNLOADING]:") {
                if let Some(percent_str) = line.split("[DOWNLOADING]:").nth(1) {
                    if let Some(percent) = percent_str.trim().strip_suffix('%') {
                        percent.parse::<f64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            let log = RealTimeLog {
                message: line,
                is_error: false,
                progress,
            };
            let _ = window_handle.emit("download-log", log);
        }
    });

    let window_handle2 = window.clone();
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            // stderrからのプログレス情報も解析
            let progress = if line.contains("[DOWNLOADING]:") {
                if let Some(percent_str) = line.split("[DOWNLOADING]:").nth(1) {
                    if let Some(percent) = percent_str.trim().strip_suffix('%') {
                        percent.parse::<f64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            let log = RealTimeLog {
                message: line,
                is_error: true,
                progress,
            };
            let _ = window_handle2.emit("download-log", log);
        }
    });

    // 両方のハンドルを待つ
    let (stdout_result, stderr_result) = tokio::join!(stdout_handle, stderr_handle);
    
    // エラーハンドリング
    if let Err(e) = stdout_result {
        eprintln!("stdout読み取りエラー: {}", e);
    }
    if let Err(e) = stderr_result {
        eprintln!("stderr読み取りエラー: {}", e);
    }

    // プロセス終了を待つ
    let status = child.wait().await.map_err(|e| format!("プロセス待機エラー: {}", e))?;

    if status.success() {
        let final_log = RealTimeLog {
            message: "✅ ダウンロードが完了しました".to_string(),
            is_error: false,
            progress: Some(100.0),
        };
        let _ = window.emit("download-log", final_log);
        
        Ok(DownloadProgress {
            message: "✅ 正常に完了しました".to_string(),
            progress: Some(1.0),
            is_error: false,
        })
    } else {
        let final_log = RealTimeLog {
            message: "❌ ダウンロードに失敗しました".to_string(),
            is_error: true,
            progress: Some(0.0),
        };
        let _ = window.emit("download-log", final_log);
        
        Ok(DownloadProgress {
            message: "❌ エラーが発生しました".to_string(),
            progress: Some(0.0),
            is_error: true,
        })
    }
}

#[tauri::command]
async fn get_default_download_directory() -> Result<String, String> {
    // デフォルトのダウンロードディレクトリを取得
    #[cfg(target_os = "windows")]
    {
        use std::env;
        if let Ok(user_profile) = env::var("USERPROFILE") {
            Ok(format!("{}\\Downloads", user_profile))
        } else {
            Ok(".".to_string())
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::env;
        if let Ok(home) = env::var("HOME") {
            Ok(format!("{}/Downloads", home))
        } else {
            Ok(".".to_string())
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::env;
        if let Ok(home) = env::var("HOME") {
            Ok(format!("{}/Downloads", home))
        } else {
            Ok(".".to_string())
        }
    }
}

#[tauri::command]
async fn check_yt_dlp_installed() -> Result<bool, String> {
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn install_yt_dlp() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: pipを使用してインストール
        match Command::new("pip").args(&["install", "--upgrade", "yt-dlp"]).output() {
            Ok(output) => {
                if output.status.success() {
                    Ok("yt-dlpのインストールが完了しました".to_string())
                } else {
                    Err(format!("yt-dlpのインストールに失敗しました: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Err(format!("pipコマンドの実行に失敗しました: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Unix系: curlを使用してインストール
        let install_script = r#"
            if command -v curl >/dev/null 2>&1; then
                curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /tmp/yt-dlp
                chmod +x /tmp/yt-dlp
                sudo mv /tmp/yt-dlp /usr/local/bin/yt-dlp
                echo "yt-dlpのインストールが完了しました"
            else
                echo "curlがインストールされていません"
                exit 1
            fi
        "#;
        
        match Command::new("sh").arg("-c").arg(install_script).output() {
            Ok(output) => {
                if output.status.success() {
                    Ok("yt-dlpのインストールが完了しました".to_string())
                } else {
                    Err(format!("yt-dlpのインストールに失敗しました: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            Err(e) => Err(format!("インストールスクリプトの実行に失敗しました: {}", e)),
        }
    }
}

#[tauri::command]
async fn read_clipboard() -> Result<String, String> {
    // Windowsの場合
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        match Command::new("powershell")
            .args(&["-command", "Get-Clipboard"])
            .output()
        {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(text)
            }
            Err(_) => Err("クリップボードの読み込みに失敗しました".to_string()),
        }
    }
    
    // macOSの場合
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        match Command::new("pbpaste").output() {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(text)
            }
            Err(_) => Err("クリップボードの読み込みに失敗しました".to_string()),
        }
    }
    
    // Linuxの場合
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // xclipを試す
        match Command::new("xclip").args(&["-o", "-selection", "clipboard"]).output() {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(text)
            }
            Err(_) => {
                // xclipが失敗したらxselを試す
                match Command::new("xsel").args(&["--output", "--clipboard"]).output() {
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        Ok(text)
                    }
                    Err(_) => Err("クリップボードの読み込みに失敗しました".to_string()),
                }
            }
        }
    }
}

#[tauri::command]
async fn select_directory(default_path: Option<String>) -> Result<Option<String>, String> {
    let dialog = if let Some(path) = default_path {
        rfd::AsyncFileDialog::new().set_directory(PathBuf::from(path)).pick_folder().await
    } else {
        rfd::AsyncFileDialog::new().pick_folder().await
    };
    Ok(dialog.map(|handle| handle.path().display().to_string()))
}

#[tauri::command]
async fn select_file(filters: Vec<FileFilter>) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    for filter in filters {
        dialog = dialog.add_filter(&filter.name, &filter.extensions);
    }
    let result = dialog.pick_file().await;
    Ok(result.map(|handle| handle.path().display().to_string()))
}

#[tauri::command]
async fn open_directory(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("ディレクトリを開けませんでした: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("ディレクトリを開けませんでした: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("ディレクトリを開けませんでした: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn save_settings(settings: serde_json::Value) -> Result<(), String> {
    let app_dir = dirs::config_dir()
        .ok_or("設定ディレクトリを取得できませんでした")?
        .join("necd-tauri");
    
    // ディレクトリが存在しない場合は作成
    fs::create_dir_all(&app_dir).map_err(|e| format!("設定ディレクトリを作成できませんでした: {}", e))?;
    
    let settings_path = app_dir.join("settings.json");
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("設定のシリアライズに失敗しました: {}", e))?;
    
    fs::write(&settings_path, settings_json)
        .map_err(|e| format!("設定の保存に失敗しました: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn load_settings() -> Result<serde_json::Value, String> {
    let app_dir = dirs::config_dir()
        .ok_or("設定ディレクトリを取得できませんでした")?
        .join("necd-tauri");
    
    let settings_path = app_dir.join("settings.json");
    
    if !settings_path.exists() {
        // デフォルト設定を返す
        return Ok(serde_json::json!({
            "outputDirectory": "",
            "format": "mp4",
            "quality": "auto",
            "cookieSource": "none",
            "cookieFilePath": "",
            "concurrentConnections": "3",
            "chapterEmbed": false,
            "playlistMode": false,
            "thumbnailEmbed": false,
            "thumbnailCrop": false,
            "compatibilityMode": false,
            "hdrMode": false
        }));
    }
    
    let settings_content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("設定ファイルの読み込みに失敗しました: {}", e))?;
    
    let settings: serde_json::Value = serde_json::from_str(&settings_content)
        .map_err(|e| format!("設定のデシリアライズに失敗しました: {}", e))?;
    
    Ok(settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            execute_download,
            get_default_download_directory,
            check_yt_dlp_installed,
            install_yt_dlp,
            read_clipboard,
            select_directory,
            select_file,
            open_directory,
            save_settings,
            load_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
