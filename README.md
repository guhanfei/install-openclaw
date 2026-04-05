# 古寒飞自动化 · OpenClaw 高级辅助使用工具

一款跨平台桌面 GUI 程序，帮助用户一键安装、配置并管理 [OpenClaw](https://openclaw.ai)。

---

## 支持平台

| 平台 | 版本要求 |
|------|---------|
| Windows（家用） | Windows 10 及以上 |
| Windows Server | Server 2016 及以上 |
| macOS | macOS 10.15 (Catalina) 及以上 |

---

## 功能介绍

| 功能 | 说明 |
|------|------|
| 检测本机电脑 | 查看 CPU / 内存 / 磁盘 / OpenClaw 数据目录 |
| 安装 OpenClaw | 自动安装包管理器、Git、Node.js、OpenClaw |
| 启动 / 停止 | 启动、停止、重启网关服务，一键打开聊天界面 |
| 升级 OpenClaw | 一键升级到最新版本 |
| 模型厂商 | 可视化切换 AI 服务商及 API Key |
| 卸载 OpenClaw | 完整卸载程序与配置目录 |

---

## 下载使用

### Windows 用户
1. 下载 `.zip` 压缩包，解压后直接运行 `.exe`
2. 无需安装，GUI 界面随即打开

### macOS 用户
1. 下载 `.dmg` 文件，将应用拖拽至 Applications 文件夹
2. 从启动台打开即可使用

> 重复打开程序时，会自动检测 `~/.openclaw` 目录中的现有配置，无需重新设置。

---

## 本地开发

### 环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)（含 cargo）
- [Tauri CLI 前置依赖](https://v2.tauri.app/start/prerequisites/)

### 启动开发模式

```bash
npm install
npm run tauri dev
```

首次运行会自动编译 Rust 后端，耗时约 1-2 分钟，后续热重载极快。

### 构建发布包

```bash
make check        # 检查环境工具是否齐全
make install-deps # 首次使用：安装交叉编译依赖
make build-mac    # 打包 macOS（Apple Silicon）
make build-win    # 打包 Windows（x86_64）
make all          # 同时打包两个平台
```

产物统一输出到 `./builds/` 目录：
- macOS：`install-openclaw-<version>-macos-arm64.dmg`
- Windows：`install-openclaw-<version>-windows-x86_64.exe`

---

## 技术栈

- **前端**：Vue 3 + TypeScript + Vite
- **后端**：Rust + Tauri 2.0
- **UI 图标**：lucide-vue-next

---

## License

Copyright 2026 古寒飞自动化

Licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE) for details.
