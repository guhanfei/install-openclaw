# 古寒飞自动化 · OpenClaw 高级辅助使用工具

一款跨平台桌面 GUI 程序，帮助用户一键安装、配置并管理 [OpenClaw](https://github.com/openclaw)（开源龙虾）。

---

## 支持平台

| 平台 | 版本要求 |
|------|---------|
| Windows（家用） | Windows 10 及以上 |
| Windows Server | Server 2016 及以上 |
| macOS | macOS 10.15 (Catalina) 及以上 |

---

## 功能介绍

### 自动化环境安装
- **Windows**：自动安装 [Chocolatey](https://chocolatey.org/) 包管理器
- **macOS**：自动安装 [Homebrew](https://brew.sh/) 包管理器

### 依赖安装
- 通过包管理器自动安装 Git
- 通过包管理器自动安装 Node.js 24 LTS 版本

### OpenClaw 管理
- 一键安装 OpenClaw（通过 npm）
- 一键卸载 OpenClaw

### 配置管理
- **模型厂商切换**：可视化修改 OpenClaw 模型配置，轻松切换不同 AI 服务商
- **Skill 管理**：支持对 OpenClaw skill 进行增删改查
- **插件管理**：支持对 OpenClaw 插件进行安装、卸载及配置管理（npm + JSON 配置联动）

---

## 安装方式

### Windows 用户
1. 下载 `.zip` 压缩包
2. 解压后直接运行 `.exe` 文件
3. GUI 界面随即打开，无需安装

### macOS 用户
1. 下载 `.dmg` 文件
2. 打开 `.dmg`，将应用拖拽至 Applications 文件夹
3. 从启动台打开即可使用

> 重复打开程序时，会自动检测 `~/.openclaw` 目录中的现有配置，无需重新设置。

---

## 开发状态

> 本项目正在积极开发中，功能和文档持续更新。

---

## License

MIT
