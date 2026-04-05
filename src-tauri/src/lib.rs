use tauri::Emitter;
use tauri_plugin_shell::{process::CommandEvent, ShellExt};

#[derive(serde::Serialize, Clone)]
struct LogPayload {
    text: String,
    level: String, // "info" | "error" | "success"
}

/// 检测单个命令是否可用，返回版本字符串或空字符串
#[tauri::command]
async fn check_command(app: tauri::AppHandle, cmd: String) -> String {
    let shell = app.shell();
    match shell.command(&cmd).args(["--version"]).output().await {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}

/// 执行命令并通过 Tauri 事件实时推送每行日志，命令完成后返回
#[tauri::command]
async fn run_command_streaming(
    app: tauri::AppHandle,
    cmd: String,
    args: Vec<String>,
) -> Result<(), String> {
    let shell = app.shell();
    let (mut rx, _child) = shell
        .command(&cmd)
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                let line = String::from_utf8_lossy(&bytes).trim().to_string();
                if !line.is_empty() {
                    let _ = app.emit("log", LogPayload { text: line, level: "info".into() });
                }
            }
            CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes).trim().to_string();
                if !line.is_empty() {
                    let _ = app.emit("log", LogPayload { text: line, level: "error".into() });
                }
            }
            CommandEvent::Terminated(status) => {
                let code = status.code.unwrap_or(-1);
                if code != 0 {
                    return Err(format!("命令异常退出，退出码: {}", code));
                }
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

/// 读取 ~/.openclaw/<filename>，返回 JSON 字符串（文件不存在返回 null）
#[tauri::command]
fn read_config(filename: String) -> Result<Option<String>, String> {
    let path = openclaw_dir().join(&filename);
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// 写入 ~/.openclaw/<filename>
#[tauri::command]
fn write_config(filename: String, content: String) -> Result<(), String> {
    let dir = openclaw_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(&filename), content).map_err(|e| e.to_string())
}

/// 删除 ~/.openclaw 整个目录
#[tauri::command]
fn delete_config_dir() -> Result<(), String> {
    let dir = openclaw_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 返回当前操作系统名称：macos / windows / linux
#[tauri::command]
fn get_os() -> String {
    std::env::consts::OS.to_string()
}

fn openclaw_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home).join(".openclaw")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_command,
            run_command_streaming,
            read_config,
            write_config,
            delete_config_dir,
            get_os,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
