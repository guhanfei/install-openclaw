use tauri::Emitter;
use tauri_plugin_shell::{process::CommandEvent, ShellExt};

#[derive(serde::Serialize, Clone)]
struct LogPayload {
    text: String,
    level: String,
}

#[derive(serde::Serialize)]
struct SkillInfo {
    name: String,
    description: String,
    plugin: String,
    path: String,
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

/// 执行命令，通过 Tauri 事件实时推送每行日志
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

/// 读取 ~/.openclaw/<filename>，文件不存在返回 null
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

/// 写入 ~/.openclaw/<filename>，写前自动备份（.bak）
#[tauri::command]
fn write_config(filename: String, content: String) -> Result<(), String> {
    let dir = openclaw_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&filename);
    // 如果文件已存在，先备份
    if path.exists() {
        let bak = dir.join(format!("{}.bak", filename));
        let _ = std::fs::copy(&path, &bak);
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// 列出所有插件的 skills（读取 extensions/<plugin>/skills/<skill>/SKILL.md）
#[tauri::command]
fn list_skills() -> Vec<SkillInfo> {
    let extensions_dir = openclaw_dir().join("extensions");
    let mut skills = Vec::new();

    let plugins = match std::fs::read_dir(&extensions_dir) {
        Ok(r) => r,
        Err(_) => return skills,
    };

    for plugin_entry in plugins.flatten() {
        if !plugin_entry.path().is_dir() { continue; }
        let plugin_name = plugin_entry.file_name().to_string_lossy().to_string();
        let skills_dir = plugin_entry.path().join("skills");
        if !skills_dir.is_dir() { continue; }

        let skill_dirs = match std::fs::read_dir(&skills_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for skill_entry in skill_dirs.flatten() {
            let skill_path = skill_entry.path();
            if !skill_path.is_dir() { continue; }

            // 尝试 SKILL.md 或 skill.md
            let md_path = ["SKILL.md", "skill.md"]
                .iter()
                .map(|f| skill_path.join(f))
                .find(|p| p.exists());

            if let Some(md) = md_path {
                let path_str = md.to_string_lossy().to_string();
                if let Ok(content) = std::fs::read_to_string(&md) {
                    let default_name = skill_entry.file_name().to_string_lossy().to_string();
                    let (name, description) = parse_skill_frontmatter(&content, &default_name);
                    skills.push(SkillInfo { name, description, plugin: plugin_name.clone(), path: path_str });
                }
            }
        }
    }

    skills
}

/// 读取单个 skill 的完整 Markdown 内容
#[tauri::command]
fn read_skill(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
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

/// 删除任意目录（用于插件卸载）
#[tauri::command]
fn delete_dir(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        std::fs::remove_dir_all(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 返回当前用户 HOME 目录的绝对路径
#[tauri::command]
fn get_home_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
}

/// 返回当前操作系统：macos / windows / linux
#[tauri::command]
fn get_os() -> String {
    std::env::consts::OS.to_string()
}

/// 检测 openclaw 是否正在运行（尝试连接 gateway 端口）
#[tauri::command]
fn check_openclaw_running(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

/// 检测 openclaw gateway daemon 是否已安装为系统服务
#[tauri::command]
fn check_daemon_installed() -> bool {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    match std::env::consts::OS {
        "macos" => std::path::Path::new(&home)
            .join("Library/LaunchAgents/ai.openclaw.gateway.plist")
            .exists(),
        "linux" => std::path::Path::new(&home)
            .join(".config/systemd/user/openclaw-gateway.service")
            .exists(),
        _ => false, // Windows 暂不检测文件，依赖 CLI 返回
    }
}

/// 执行命令并返回 stdout 字符串（用于需要捕获输出的场景，如 npm view）
#[tauri::command]
async fn run_command_output(app: tauri::AppHandle, cmd: String, args: Vec<String>) -> Result<String, String> {
    let shell = app.shell();
    let out = shell.command(&cmd).args(args).output().await.map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 启动 openclaw gateway（后台非阻塞）
#[tauri::command]
fn start_openclaw() -> Result<(), String> {
    std::process::Command::new("openclaw")
        .args(["gateway", "run"])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ── 内部工具函数 ────────────────────────────────────────────

fn openclaw_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home).join(".openclaw")
}

/// 从 SKILL.md 的 YAML frontmatter 提取 name 和 description
fn parse_skill_frontmatter(content: &str, default_name: &str) -> (String, String) {
    let mut name = default_name.to_string();
    let mut description = String::new();

    if !content.starts_with("---") {
        return (name, description);
    }
    let after = &content[3..];
    let end = match after.find("---") {
        Some(i) => i,
        None => return (name, description),
    };
    let frontmatter = &after[..end];

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("name:") {
            name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("description:") {
            description = v.trim().to_string();
        }
    }
    (name, description)
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
            list_skills,
            read_skill,
            delete_config_dir,
            delete_dir,
            get_home_dir,
            get_os,
            run_command_output,
            check_openclaw_running,
            check_daemon_installed,
            start_openclaw,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
