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

/// 用系统文件管理器打开指定目录（macOS: Finder / Windows: Explorer / Linux: xdg-open）
#[tauri::command]
fn open_dir(path: String) -> Result<(), String> {
    let cmd = match std::env::consts::OS {
        "macos"   => "open",
        "windows" => "explorer",
        _         => "xdg-open",
    };
    std::process::Command::new(cmd)
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 启动 openclaw gateway（流式日志 + 检测到 ready 后返回，进程继续后台运行）
/// 最多等 15 秒；检测到 "listening on" 时立即返回
#[tauri::command]
async fn start_openclaw(app: tauri::AppHandle) -> Result<(), String> {
    let shell = app.shell();
    let (mut rx, _child) = shell
        .command("openclaw")
        .args(["gateway", "run"])
        .spawn()
        .map_err(|e| e.to_string())?;

    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(15);

    loop {
        let result = tokio::time::timeout_at(deadline, rx.recv()).await;
        match result {
            Err(_) => break, // 超时，进程仍在运行，直接返回
            Ok(None) => break, // channel 关闭
            Ok(Some(event)) => match event {
                CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes) => {
                    let line = String::from_utf8_lossy(&bytes).trim().to_string();
                    if !line.is_empty() {
                        let _ = app.emit("log", LogPayload { text: line.clone(), level: "info".into() });
                        if line.contains("listening on") {
                            break; // 网关已就绪，返回控制权；进程继续运行
                        }
                    }
                }
                CommandEvent::Terminated(s) => {
                    let code = s.code.unwrap_or(-1);
                    if code != 0 {
                        return Err(format!("网关进程意外退出，退出码: {}", code));
                    }
                    break;
                }
                _ => {}
            },
        }
    }
    // _child drop 后进程仍独立运行（tauri-plugin-shell 不在 drop 时 kill）
    Ok(())
}

/// 强制终止占用指定端口的进程（用于非 daemon 模式下停止网关）
#[tauri::command]
fn kill_port_process(port: u16) -> Result<(), String> {
    match std::env::consts::OS {
        "macos" | "linux" => {
            let out = std::process::Command::new("lsof")
                .args(["-ti", &format!("tcp:{}", port)])
                .output()
                .map_err(|e| e.to_string())?;
            let pids = String::from_utf8_lossy(&out.stdout);
            let pids = pids.trim();
            if pids.is_empty() {
                return Ok(()); // 进程已不存在
            }
            // 先 SIGTERM（优雅退出），给进程 2 秒清理时间，再 SIGKILL 兜底
            for pid in pids.lines() {
                let _ = std::process::Command::new("kill").args([pid.trim()]).status();
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            // 再检查一次，如果还在就强杀
            let out2 = std::process::Command::new("lsof")
                .args(["-ti", &format!("tcp:{}", port)])
                .output()
                .map_err(|e| e.to_string())?;
            for pid in String::from_utf8_lossy(&out2.stdout).trim().lines() {
                let _ = std::process::Command::new("kill").args(["-9", pid.trim()]).status();
            }
            Ok(())
        }
        "windows" => {
            let out = std::process::Command::new("netstat")
                .args(["-ano"])
                .output()
                .map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&out.stdout);
            let port_str = format!(":{}", port);
            let mut pids = std::collections::HashSet::new();
            for line in text.lines() {
                if line.contains(&port_str) && line.contains("LISTENING") {
                    if let Some(pid) = line.split_whitespace().last() {
                        pids.insert(pid.to_string());
                    }
                }
            }
            for pid in pids {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid, "/F"])
                    .status();
            }
            Ok(())
        }
        _ => Err("不支持的操作系统".to_string()),
    }
}

// ── 内部工具函数 ────────────────────────────────────────────

#[derive(serde::Serialize)]
struct SystemInfo {
    os: String,
    os_version: String,
    arch: String,
    cpu_brand: String,
    total_memory_bytes: u64,
    disk_total_bytes: u64,
    disk_free_bytes: u64,
    openclaw_dir: String,
    openclaw_dir_exists: bool,
}

/// 执行简单命令并返回 stdout（用于系统信息采集，同步阻塞）
fn run_simple(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// 解析 df -k / 输出（1K-blocks），返回 (total_bytes, free_bytes)
fn parse_df_kblocks(output: &str) -> (u64, u64) {
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let total: u64 = parts[1].parse().unwrap_or(0);
            let avail: u64 = parts[3].parse().unwrap_or(0);
            return (total * 1024, avail * 1024);
        }
    }
    (0, 0)
}


/// 采集本机系统信息（OS、CPU、内存、磁盘、openclaw 目录）
#[tauri::command]
fn get_system_info() -> SystemInfo {
    let os = std::env::consts::OS.to_string();
    let dir = openclaw_dir();
    let openclaw_dir_str = dir.to_string_lossy().to_string();
    let openclaw_dir_exists = dir.exists();

    match os.as_str() {
        "macos" => {
            let os_version = run_simple("sw_vers", &["-productVersion"]);
            let arch = run_simple("uname", &["-m"]);
            let cpu_brand = run_simple("sysctl", &["-n", "machdep.cpu.brand_string"]);
            let total_memory_bytes: u64 = run_simple("sysctl", &["-n", "hw.memsize"]).parse().unwrap_or(0);
            let (disk_total_bytes, disk_free_bytes) = parse_df_kblocks(&run_simple("df", &["-k", "/"]));
            SystemInfo { os, os_version, arch, cpu_brand, total_memory_bytes, disk_total_bytes, disk_free_bytes, openclaw_dir: openclaw_dir_str, openclaw_dir_exists }
        }
        "windows" => {
            // 使用 Get-CimInstance（Win10/11 均支持，wmic 在 Win11 22H2+ 已废弃）
            let ps = |cmd: &str| run_simple("powershell", &["-NoProfile", "-Command", cmd]);
            let os_version  = ps("(Get-CimInstance Win32_OperatingSystem).Caption");
            let arch        = std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "Unknown".to_string());
            let cpu_brand   = ps("(Get-CimInstance Win32_Processor).Name");
            let total_memory_bytes: u64 = ps("(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory").parse().unwrap_or(0);
            // Get-PSDrive C 返回已用/剩余字节数（KB 单位需 *1024，实际返回 MB，需 *1MB）
            // 用更可靠的 Win32_LogicalDisk
            let disk_total_bytes: u64 = ps("(Get-CimInstance -Query \"SELECT Size FROM Win32_LogicalDisk WHERE DeviceID='C:'\").Size").parse().unwrap_or(0);
            let disk_free_bytes: u64  = ps("(Get-CimInstance -Query \"SELECT FreeSpace FROM Win32_LogicalDisk WHERE DeviceID='C:'\").FreeSpace").parse().unwrap_or(0);
            SystemInfo { os, os_version, arch, cpu_brand, total_memory_bytes, disk_total_bytes, disk_free_bytes, openclaw_dir: openclaw_dir_str, openclaw_dir_exists }
        }
        _ => SystemInfo {
            os, os_version: String::new(), arch: std::env::consts::ARCH.to_string(),
            cpu_brand: String::new(), total_memory_bytes: 0, disk_total_bytes: 0, disk_free_bytes: 0,
            openclaw_dir: openclaw_dir_str, openclaw_dir_exists,
        }
    }
}

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
            get_system_info,
            open_dir,
            start_openclaw,
            kill_port_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
