<template>
  <div class="page">
    <h1 class="page-title">安装 OpenClaw</h1>
    <p class="page-desc">自动完成环境依赖安装，一步到位。</p>

    <!-- 环境检测 -->
    <section class="section">
      <h2 class="section-title">环境检测</h2>
      <div class="env-grid">
        <div
          v-for="item in envItems"
          :key="item.id"
          class="env-card"
          :class="item.status"
        >
          <span class="env-icon">{{ item.icon }}</span>
          <div class="env-info">
            <span class="env-name">{{ item.name }}</span>
            <span class="env-status-text">{{ item.statusText }}</span>
          </div>
          <span class="env-dot"></span>
        </div>
      </div>
      <button class="btn btn-secondary" :disabled="checking" @click="checkEnv">
        {{ checking ? "检测中..." : "🔍 重新检测" }}
      </button>
    </section>

    <!-- 一键安装 -->
    <section class="section">
      <h2 class="section-title">一键安装</h2>
      <div class="install-steps">
        <div
          v-for="(step, i) in installSteps"
          :key="i"
          class="step-item"
          :class="step.status"
        >
          <div class="step-num">
            <span v-if="step.status === 'running'" class="spinner">⟳</span>
            <span v-else>{{ i + 1 }}</span>
          </div>
          <div class="step-info">
            <span class="step-name">{{ step.name }}</span>
            <span class="step-desc">{{ step.desc }}</span>
          </div>
          <span class="step-badge">{{ step.statusText }}</span>
        </div>
      </div>

      <!-- 日志输出 -->
      <div v-if="logs.length > 0" class="log-box" ref="logBox">
        <div v-for="(log, i) in logs" :key="i" class="log-line" :class="log.level">
          {{ log.text }}
        </div>
      </div>

      <div class="action-row">
        <button class="btn btn-primary" :disabled="installing || allInstalled" @click="startInstall">
          {{ installing ? "安装中..." : allInstalled ? "✓ 已全部安装" : "🚀 开始安装" }}
        </button>
        <button v-if="installError" class="btn btn-secondary" @click="retryInstall">
          重试
        </button>
      </div>
      <p v-if="installError" class="error-hint">{{ installError }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface EnvItem {
  id: string;
  icon: string;
  name: string;
  cmd: string;
  status: "ok" | "missing" | "checking";
  statusText: string;
}

interface InstallStep {
  name: string;
  desc: string;
  status: "pending" | "running" | "done" | "error";
  statusText: string;
  // macOS 和 Windows 的执行命令
  mac: { cmd: string; args: string[] };
  win: { cmd: string; args: string[] };
}

interface LogLine {
  text: string;
  level: "info" | "error" | "success";
}

const checking = ref(false);
const installing = ref(false);
const installError = ref("");
const logs = ref<LogLine[]>([]);
const logBox = ref<HTMLElement | null>(null);
const currentOs = ref("macos");

const envItems = ref<EnvItem[]>([
  { id: "pkgmgr", icon: "🍺", name: "包管理器 (brew/choco)", cmd: "", status: "checking", statusText: "检测中..." },
  { id: "git",    icon: "🔧", name: "Git",                    cmd: "git",  status: "checking", statusText: "检测中..." },
  { id: "node",   icon: "💚", name: "Node.js",                cmd: "node", status: "checking", statusText: "检测中..." },
  { id: "openclaw", icon: "🦞", name: "OpenClaw",             cmd: "openclaw", status: "checking", statusText: "检测中..." },
]);

const installSteps = ref<InstallStep[]>([
  {
    name: "安装包管理器", desc: "Windows: Chocolatey / macOS: Homebrew",
    status: "pending", statusText: "等待",
    // macOS: curl 下载脚本后用管道交给 bash 执行（不能用 $() 展开，Tauri 不走 shell）
    mac: { cmd: "/bin/bash", args: ["-c", "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh | /bin/bash"] },
    // Windows: PowerShell 执行 Chocolatey 安装脚本
    win: { cmd: "powershell", args: ["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"] },
  },
  {
    name: "安装 Git", desc: "通过包管理器安装",
    status: "pending", statusText: "等待",
    mac: { cmd: "brew", args: ["install", "git"] },
    win: { cmd: "choco", args: ["install", "git", "-y"] },
  },
  {
    name: "安装 Node.js LTS", desc: "通过包管理器安装",
    status: "pending", statusText: "等待",
    mac: { cmd: "brew", args: ["install", "node"] },
    win: { cmd: "choco", args: ["install", "nodejs-lts", "-y"] },
  },
  {
    name: "安装 OpenClaw", desc: "npm install -g openclaw",
    status: "pending", statusText: "等待",
    mac: { cmd: "npm", args: ["install", "-g", "openclaw"] },
    win: { cmd: "npm", args: ["install", "-g", "openclaw"] },
  },
]);

const allInstalled = computed(() =>
  envItems.value.every((e) => e.status === "ok")
);

onMounted(async () => {
  currentOs.value = await invoke<string>("get_os");
  await checkEnv();
});

async function checkEnv() {
  checking.value = true;

  // 设置包管理器检测命令（平台相关）
  const pkgCmd = currentOs.value === "windows" ? "choco" : "brew";
  envItems.value[0].cmd = pkgCmd;

  for (const item of envItems.value) {
    item.status = "checking";
    item.statusText = "检测中...";
    if (!item.cmd) continue;
    const version = await invoke<string>("check_command", { cmd: item.cmd });
    if (version) {
      item.status = "ok";
      item.statusText = version.split("\n")[0].slice(0, 40);
    } else {
      item.status = "missing";
      item.statusText = "未安装";
    }
  }

  checking.value = false;

  // 根据已安装情况跳过已完成步骤
  syncStepStatus();
}

function syncStepStatus() {
  const [pkgOk, gitOk, nodeOk, openclawOk] = envItems.value.map((e) => e.status === "ok");
  if (pkgOk)     markStepDone(0);
  if (gitOk)     markStepDone(1);
  if (nodeOk)    markStepDone(2);
  if (openclawOk) markStepDone(3);
}

function markStepDone(i: number) {
  installSteps.value[i].status = "done";
  installSteps.value[i].statusText = "已安装";
}

async function startInstall() {
  installing.value = true;
  installError.value = "";
  logs.value = [];

  // 注册流式日志监听
  const unlisten = await listen<{ text: string; level: string }>("log", (event) => {
    logs.value.push({ text: event.payload.text, level: event.payload.level as LogLine["level"] });
    nextTick(() => {
      if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
    });
  });

  try {
    for (let i = 0; i < installSteps.value.length; i++) {
      const step = installSteps.value[i];
      if (step.status === "done") continue; // 已安装，跳过

      step.status = "running";
      step.statusText = "安装中...";
      addLog(`▶ ${step.name}...`, "info");

      const target = currentOs.value === "windows" ? step.win : step.mac;
      try {
        await invoke("run_command_streaming", { cmd: target.cmd, args: target.args });
        step.status = "done";
        step.statusText = "完成";
        addLog(`✓ ${step.name} 安装完成`, "success");
      } catch (err) {
        step.status = "error";
        step.statusText = "失败";
        installError.value = `${step.name} 安装失败：${err}`;
        addLog(`✗ ${step.name} 失败：${err}`, "error");
        break;
      }
    }

    // 安装完成后重新检测
    await checkEnv();
  } finally {
    installing.value = false;
    unlisten();
  }
}

async function retryInstall() {
  // 将失败步骤重置为 pending
  installSteps.value.forEach((s) => {
    if (s.status === "error") {
      s.status = "pending";
      s.statusText = "等待";
    }
  });
  installError.value = "";
  await startInstall();
}

function addLog(text: string, level: LogLine["level"]) {
  logs.value.push({ text, level });
  nextTick(() => {
    if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
  });
}
</script>

<style scoped>
.page { max-width: 680px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }

.env-grid { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
.env-card { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: 8px; background: var(--color-surface); border: 1px solid var(--color-border); }
.env-card.ok { border-color: rgba(72, 187, 120, 0.3); }
.env-card.missing { border-color: rgba(252, 129, 129, 0.3); }
.env-icon { font-size: 18px; }
.env-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.env-name { font-size: 13px; font-weight: 500; }
.env-status-text { font-size: 11px; color: var(--color-text-muted); font-family: monospace; }
.env-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--color-text-muted); }
.env-card.ok .env-dot { background: var(--color-success); }
.env-card.missing .env-dot { background: var(--color-danger); }

.install-steps { display: flex; flex-direction: column; gap: 6px; margin-bottom: 16px; }
.step-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: 8px; background: var(--color-surface); border: 1px solid var(--color-border); }
.step-item.running { border-color: rgba(108, 99, 255, 0.4); }
.step-item.done { border-color: rgba(72, 187, 120, 0.3); }
.step-item.error { border-color: rgba(252, 129, 129, 0.3); }
.step-num { width: 24px; height: 24px; border-radius: 50%; background: var(--color-border); display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700; flex-shrink: 0; }
.step-item.running .step-num { background: var(--color-primary); }
.step-item.done .step-num { background: var(--color-success); }
.step-item.error .step-num { background: var(--color-danger); }
.spinner { display: inline-block; animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.step-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.step-name { font-size: 13px; font-weight: 500; }
.step-desc { font-size: 11px; color: var(--color-text-muted); }
.step-badge { font-size: 11px; padding: 2px 8px; border-radius: 4px; background: var(--color-border); color: var(--color-text-muted); }
.step-item.done .step-badge { background: rgba(72, 187, 120, 0.15); color: var(--color-success); }
.step-item.error .step-badge { background: rgba(252, 129, 129, 0.15); color: var(--color-danger); }
.step-item.running .step-badge { background: rgba(108, 99, 255, 0.15); color: var(--color-primary); }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; margin-bottom: 16px; height: 200px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; word-break: break-all; }
.log-line.success { color: var(--color-success); }
.log-line.error { color: var(--color-danger); }
.log-line.info { color: var(--color-text-muted); }

.action-row { display: flex; gap: 10px; align-items: center; }
.error-hint { margin-top: 8px; font-size: 12px; color: var(--color-danger); }
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
