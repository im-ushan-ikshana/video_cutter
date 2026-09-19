use tauri_plugin_shell::ShellExt;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use tauri::Emitter;

pub struct ProxyState {
    pub cancel_flag: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub frame_rate: String,
    pub bit_rate: String,
    pub start_time: String,
}

#[tauri::command]
async fn get_video_metadata(app: tauri::AppHandle, path: String) -> Result<VideoMetadata, String> {
    let output = app.shell()
        .sidecar("ffprobe")
        .map_err(|e| e.to_string())?
        .args(&[
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,duration,codec_name,r_frame_rate,bit_rate,start_time",
            "-show_entries", "format=duration,bit_rate",
            "-of", "json",
            &path
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let v: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;
    let stream = v["streams"].get(0);

    let duration_str = stream
        .and_then(|s| s["duration"].as_str())
        .or_else(|| v["format"]["duration"].as_str())
        .unwrap_or("0");
    let duration = duration_str.parse::<f64>().unwrap_or(0.0);

    let width = stream.and_then(|s| s["width"].as_u64()).unwrap_or(0) as u32;
    let height = stream.and_then(|s| s["height"].as_u64()).unwrap_or(0) as u32;
    let codec = stream.and_then(|s| s["codec_name"].as_str()).unwrap_or("unknown").to_string();
    let frame_rate = stream.and_then(|s| s["r_frame_rate"].as_str()).unwrap_or("N/A").to_string();
    let bit_rate = stream
        .and_then(|s| s["bit_rate"].as_str())
        .or_else(|| v["format"]["bit_rate"].as_str())
        .unwrap_or("N/A")
        .to_string();
    let start_time = stream.and_then(|s| s["start_time"].as_str()).unwrap_or("0.000000").to_string();

    Ok(VideoMetadata {
        duration,
        width,
        height,
        codec,
        frame_rate,
        bit_rate,
        start_time,
    })
}

#[tauri::command(rename_all = "camelCase")]
async fn cut_video(app: tauri::AppHandle, input_path: String, start_time: f64, end_time: f64, target_format: String) -> Result<String, String> {
    let input = PathBuf::from(&input_path);
    let stem = input.file_stem().ok_or("Invalid input file: no file stem")?.to_string_lossy();
    let parent = input.parent().ok_or("Invalid input file: no parent directory")?;
    
    let mut args = vec![
        "-y".to_string(),
        "-ss".to_string(), start_time.to_string(),
        "-to".to_string(), end_time.to_string(),
        "-i".to_string(), input_path.clone(),
    ];

    let base_ext = match target_format.as_str() {
        "MP4_H264" => {
            args.push("-c:v".to_string()); args.push("libx264".to_string());
            args.push("-preset".to_string()); args.push("fast".to_string());
            args.push("-c:a".to_string()); args.push("aac".to_string());
            "mp4"
        },
        "MP4_HEVC" => {
            args.push("-c:v".to_string()); args.push("libx265".to_string());
            args.push("-preset".to_string()); args.push("fast".to_string());
            args.push("-vtag".to_string()); args.push("hvc1".to_string());
            args.push("-c:a".to_string()); args.push("aac".to_string());
            "mp4"
        },
        "WEBM_VP9" => {
            args.push("-c:v".to_string()); args.push("libvpx-vp9".to_string());
            args.push("-b:v".to_string()); args.push("2M".to_string());
            args.push("-c:a".to_string()); args.push("libopus".to_string());
            "webm"
        },
        _ => {
            // ORIGINAL (Fast cut, no re-encode)
            args.push("-c".to_string()); args.push("copy".to_string());
            input.extension().unwrap_or_default().to_str().unwrap_or("mp4")
        }
    };

    let mut output_path = parent.join(format!("{}_cut.{}", stem, base_ext));
    let mut counter = 1;
    while output_path.exists() {
        output_path = parent.join(format!("{}_cut ({}).{}", stem, counter, base_ext));
        counter += 1;
    }

    args.push(output_path.to_string_lossy().to_string());

    let output = app.shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg failed: {}", stderr));
    }

    Ok(output_path.to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStats {
    pub file_count: usize,
    pub total_bytes: u64,
}

fn get_proxy_dir() -> PathBuf {
    std::env::temp_dir().join("video_cutter_proxies")
}

#[tauri::command]
fn cancel_proxy(state: tauri::State<'_, ProxyState>) {
    state.cancel_flag.store(true, Ordering::Relaxed);
}

#[tauri::command(rename_all = "camelCase")]
async fn generate_proxy(app: tauri::AppHandle, state: tauri::State<'_, ProxyState>, input_path: String, quality: u32) -> Result<String, String> {
    let input = PathBuf::from(&input_path);
    let stem = input.file_stem().ok_or("Invalid input file: no file stem")?.to_string_lossy();
    
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input_path.hash(&mut hasher);
    let path_hash = hasher.finish();

    let temp_dir = get_proxy_dir();
    if !temp_dir.exists() {
        let _ = std::fs::create_dir_all(&temp_dir);
    }
    // Automatically prune old or orphaned proxies before starting
    cleanup_proxies(None);

    let final_output_path = temp_dir.join(format!("{}_{:x}_proxy_{}.mp4", stem, path_hash, quality));
    let temp_output_path = temp_dir.join(format!("~tmp_{}_{:x}_proxy_{}.mp4", stem, path_hash, quality));

    if final_output_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&final_output_path) {
            if metadata.len() > 0 {
                return Ok(final_output_path.to_string_lossy().to_string());
            }
        }
        let _ = std::fs::remove_file(&final_output_path);
    }

    state.cancel_flag.store(false, Ordering::Relaxed);
    // Force even dimensions for libx264 compatibility by using trunc(val/2)*2
    let scale = format!("scale=trunc(iw*{0}/2)*2:trunc(ih*{0}/2)*2", quality as f32 / 100.0);

    let (mut rx, child) = app.shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(&[
            "-y",
            "-i", &input_path,
            "-vf", &scale,
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-crf", "28",
            "-an",
            temp_output_path.to_str().unwrap()
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    while let Some(event) = rx.recv().await {
        if state.cancel_flag.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = std::fs::remove_file(&temp_output_path);
            return Err("CANCELLED".into());
        }

        match event {
            tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                let line_str = String::from_utf8_lossy(&line);
                if let Some(time_idx) = line_str.find("time=") {
                    let time_str = &line_str[time_idx + 5..];
                    if let Some(space_idx) = time_str.find(" ") {
                        let t = &time_str[..space_idx];
                        let parts: Vec<&str> = t.split(':').collect();
                        if parts.len() == 3 {
                            let h: f64 = parts[0].parse().unwrap_or(0.0);
                            let m: f64 = parts[1].parse().unwrap_or(0.0);
                            let s: f64 = parts[2].parse().unwrap_or(0.0);
                            let total_seconds = h * 3600.0 + m * 60.0 + s;
                            let _ = app.emit("proxy_progress", total_seconds);
                        }
                    }
                }
            }
            tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                if payload.code != Some(0) {
                    let _ = std::fs::remove_file(&temp_output_path);
                    return Err(format!("FFmpeg failed with exit code: {}", payload.code.unwrap_or(-1)));
                }
            }
            tauri_plugin_shell::process::CommandEvent::Error(err) => {
                let _ = std::fs::remove_file(&temp_output_path);
                return Err(err);
            }
            _ => {}
        }
    }

    // Rename the temporary file to the final output file
    if let Err(e) = std::fs::rename(&temp_output_path, &final_output_path) {
        return Err(format!("Failed to finalize proxy file: {}", e));
    }

    // Keep cache pruned while retaining the newly generated proxy file
    cleanup_proxies(Some(&final_output_path));

    Ok(final_output_path.to_string_lossy().to_string())
}

fn cleanup_proxies(keep_file: Option<&Path>) {
    let temp_dir = get_proxy_dir();
    if !temp_dir.exists() {
        return;
    }

    let max_retention_seconds = 2.0 * 3600.0; // 2 hours max retention
    let max_cache_bytes: u64 = 500 * 1024 * 1024; // 500 MB max budget
    let target_trim_bytes: u64 = 250 * 1024 * 1024; // Prune to 250 MB if exceeded

    let mut valid_files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            
            // Delete orphaned incomplete files immediately
            if file_name.starts_with("~tmp_") {
                let _ = std::fs::remove_file(&path);
                continue;
            }

            // Never delete the currently active proxy file
            if let Some(keep) = keep_file {
                if path == keep {
                    continue;
                }
            }

            if let Ok(metadata) = entry.metadata() {
                let len = metadata.len();
                let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                // Age-based eviction
                if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                    if age.as_secs_f64() > max_retention_seconds {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                }

                total_size += len;
                valid_files.push((path, len, modified));
            }
        }
    }

    // Size-based eviction: delete oldest modified first until within target
    if total_size > max_cache_bytes {
        valid_files.sort_by_key(|f| f.2);
        for (path, len, _) in valid_files {
            if total_size <= target_trim_bytes {
                break;
            }
            if std::fs::remove_file(&path).is_ok() {
                total_size = total_size.saturating_sub(len);
            }
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
fn get_proxy_stats() -> Result<ProxyStats, String> {
    let temp_dir = get_proxy_dir();
    if !temp_dir.exists() {
        return Ok(ProxyStats { file_count: 0, total_bytes: 0 });
    }

    let mut file_count = 0usize;
    let mut total_bytes = 0u64;

    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                file_count += 1;
                if let Ok(metadata) = path.metadata() {
                    total_bytes += metadata.len();
                }
            }
        }
    }

    Ok(ProxyStats { file_count, total_bytes })
}

#[tauri::command(rename_all = "camelCase")]
fn clear_proxy_cache() -> Result<u64, String> {
    let temp_dir = get_proxy_dir();
    if !temp_dir.exists() {
        return Ok(0);
    }

    let mut freed_bytes = 0u64;
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    freed_bytes += metadata.len();
                }
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    Ok(freed_bytes)
}

#[tauri::command]
fn open_devtools(_window: tauri::WebviewWindow) {
    #[cfg(debug_assertions)]
    {
        _window.open_devtools();
    }
}

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|_app| {
            // Run proxy cache cleanup on startup in the background
            std::thread::spawn(|| {
                cleanup_proxies(None);
            });
            Ok(())
        })
        .manage(ProxyState { cancel_flag: Arc::new(AtomicBool::new(false)) })
        .invoke_handler(tauri::generate_handler![
            get_video_metadata,
            cut_video,
            generate_proxy,
            cancel_proxy,
            get_proxy_stats,
            clear_proxy_cache,
            open_devtools
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
