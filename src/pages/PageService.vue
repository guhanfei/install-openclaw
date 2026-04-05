<template>
  <div class="page">
    <h1 class="page-title">启动 / 停止 OpenClaw</h1>
    <p class="page-desc">管理本机 OpenClaw 网关服务（监听端口 {{ gatewayPort }}）。</p>

    <!-- 状态卡片 -->
    <section class="section">
      <div class="status-card" :class="running ? 'running' : 'stopped'">
        <div class="status-dot"></div>
        <div class="status-info">
          <span class="status-label">{{ running ? "运行中" : "已停止" }}</span>
          <span class="status-sub">{{ running ? `127.0.0.1:${gatewayPort}` : "网关未启动" }}</span>
        </div>
        <button class="btn btn-sm btn-secondary" @click="checkStatus">
          🔄 刷新
        </button>
      </div>
    </section>

    <!-- 操作按钮 -->
    <section class="section">
      <div class="action-row">
        <button v-if="!running" class="btn btn-primary" :disabled="starting" @click="startService">
          {{ starting ? "启动中..." : "▶ 启动 OpenClaw" }}
        </button>
        <template v-if="running">
          <button class="btn btn-danger" :disabled="stopping || restarting" @click="stopService">
            {{ stopping ? "停止中..." : "■ 停止 OpenClaw" }}
          </button>
          <button class="btn btn-secondary" :disabled="restarting || stopping" @click="restartService">
            {{ restarting ? "重启中..." : "↺ 重启" }}
          </button>
        </template>
      </div>
      <p v-if="actionMsg" class="action-msg" :class="actionMsgType">{{ actionMsg }}</p>
      <p v-if="running && !daemonInstalled" class="action-msg daemon-warn">
        ⚠ 当前未安装开机自启服务，「停止/重启」命令需要 daemon。建议在下方启用开机自启。
      </p>
      <div v-if="running" class="open-section">
        <button class="btn btn-primary btn-open" @click="openInBrowser">↗ · 打开 OpenClaw 聊天界面</button>
        <div class="open-url-row">
          <span class="open-url">http://localhost:{{ gatewayPort }}/chat?session=main{{ gatewayToken ? '&token=' + gatewayToken : '' }}</span>
          <button class="copy-btn" title="复制链接" @click="copyUrl">复制</button>
        </div>
        <p v-if="!gatewayToken" class="open-warn">⚠ 未读取到网关令牌，打开后可能需要手动填写</p>
      </div>
    </section>

    <!-- 启动日志 -->
    <section v-if="logs.length > 0" class="section">
      <h2 class="section-title">输出日志</h2>
      <div class="log-box" ref="logBox">
        <div v-for="(log, i) in logs" :key="i" class="log-line" :class="log.level">
          {{ log.text }}
        </div>
      </div>
    </section>

    <!-- 开机自启动 -->
    <section class="section">
      <div class="toolbar">
        <h2 class="section-title">开机自启动</h2>
        <button class="btn btn-sm btn-secondary" :disabled="daemonBusy" @click="refreshDaemon">
          {{ daemonBusy ? "..." : "🔄 刷新" }}
        </button>
      </div>
      <div class="daemon-card" :class="daemonInstalled ? 'daemon-on' : 'daemon-off'">
        <div class="daemon-dot"></div>
        <div class="daemon-info">
          <span class="daemon-label">{{ daemonInstalled ? "已启用开机自启" : "未启用开机自启" }}</span>
          <span class="daemon-sub">{{ daemonInstalled ? "系统启动时自动运行 OpenClaw 网关" : "需手动启动，关机后不自动恢复" }}</span>
        </div>
        <button
          v-if="!daemonInstalled"
          class="btn btn-sm btn-daemon-on"
          :disabled="daemonBusy"
          @click="installDaemon"
        >
          {{ daemonBusy ? "安装中..." : "启用自启" }}
        </button>
        <button
          v-else
          class="btn btn-sm btn-daemon-off"
          :disabled="daemonBusy"
          @click="uninstallDaemon"
        >
          {{ daemonBusy ? "取消中..." : "取消自启" }}
        </button>
      </div>
      <p v-if="daemonMsg" class="daemon-msg" :class="daemonMsgType">{{ daemonMsg }}</p>
    </section>

    <!-- 说明 -->
    <section class="section">
      <div class="tip-card">
        <p>💡 <strong>说明</strong></p>
        <ul>
          <li>点击「启动」后 OpenClaw 在后台运行，关闭本工具不影响 OpenClaw 运行。</li>
          <li>「停止」会终止本机 OpenClaw 网关进程。</li>
          <li>「开机自启」通过 <code>openclaw daemon install</code> 安装系统服务（macOS LaunchAgent / Linux systemd）。</li>
          <li>网关端口从 <code>~/.openclaw/openclaw.json</code> 的 <code>gateway.port</code> 读取，默认 18790。</li>
        </ul>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

interface LogLine { text: string; level: "info" | "error" | "success" }

const running = ref(false);
const starting = ref(false);
const stopping = ref(false);
const restarting = ref(false);
const daemonInstalled = ref(false);
const daemonBusy = ref(false);
const logs = ref<LogLine[]>([]);
const logBox = ref<HTMLElement | null>(null);
const actionMsg = ref("");
const actionMsgType = ref<"ok" | "err">("ok");
const daemonMsg = ref("");
const daemonMsgType = ref<"ok" | "err">("ok");
const gatewayPort = ref(18790);
const gatewayToken = ref("");

let pollTimer: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  // 从 openclaw.json 读取实际端口
  try {
    const raw = await invoke<string | null>("read_config", { filename: "openclaw.json" });
    if (raw) {
      const cfg = JSON.parse(raw) as { gateway?: { port?: number; auth?: { token?: string } } };
      if (cfg.gateway?.port) gatewayPort.value = cfg.gateway.port;
      if (cfg.gateway?.auth?.token) gatewayToken.value = cfg.gateway.auth.token;
    }
  } catch { /* 读不到就用默认值 */ }

  await Promise.all([checkStatus(), checkDaemon()]);
  // 每 5 秒自动轮询一次状态
  pollTimer = setInterval(checkStatus, 5000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});

async function checkStatus() {
  running.value = await invoke<boolean>("check_openclaw_running", { port: gatewayPort.value });
}

async function checkDaemon() {
  daemonInstalled.value = await invoke<boolean>("check_daemon_installed");
}

async function startService() {
  starting.value = true;
  actionMsg.value = "";
  logs.value = [];

  try {
    if (daemonInstalled.value) {
      // 已安装 daemon：用 openclaw gateway start（有退出码，可检测结果）
      await invoke("run_command_streaming", { cmd: "openclaw", args: ["gateway", "start"] });
    } else {
      // 未安装 daemon：后台运行 openclaw gateway run
      await invoke("start_openclaw");
    }
    // 等 1.5 秒后检测一次，给进程启动留时间
    await new Promise((r) => setTimeout(r, 1500));
    await checkStatus();

    if (running.value) {
      logs.value.push({ text: "✓ OpenClaw 启动成功", level: "success" });
      actionMsg.value = "✓ 已启动";
      actionMsgType.value = "ok";
    } else {
      logs.value.push({ text: "⚠ 启动命令已执行，但暂未检测到端口响应，可能正在初始化", level: "info" });
      actionMsg.value = "已发送启动指令，请稍后刷新状态";
      actionMsgType.value = "ok";
    }
  } catch (err) {
    actionMsg.value = `启动失败：${err}`;
    actionMsgType.value = "err";
    logs.value.push({ text: `✗ ${err}`, level: "error" });
  } finally {
    starting.value = false;
    nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
  }
}

function dashboardUrl() {
  const base = `http://localhost:${gatewayPort.value}/chat?session=main`;
  return gatewayToken.value ? `${base}&token=${gatewayToken.value}` : base;
}

async function openInBrowser() {
  await open(dashboardUrl());
}

async function copyUrl() {
  await navigator.clipboard.writeText(dashboardUrl());
}

async function stopService() {
  stopping.value = true;
  actionMsg.value = "";
  try {
    await invoke("run_command_streaming", { cmd: "openclaw", args: ["gateway", "stop"] });
    await new Promise((r) => setTimeout(r, 800));
    await checkStatus();
    actionMsg.value = running.value ? "⚠ 进程可能仍在运行，请稍后重试" : "✓ 已停止";
    actionMsgType.value = running.value ? "err" : "ok";
  } catch (err) {
    await checkStatus();
    actionMsg.value = running.value ? `停止失败：${err}` : "✓ 已停止";
    actionMsgType.value = running.value ? "err" : "ok";
  } finally {
    stopping.value = false;
  }
}

async function restartService() {
  restarting.value = true;
  actionMsg.value = "";
  try {
    await invoke("run_command_streaming", { cmd: "openclaw", args: ["gateway", "restart"] });
    await new Promise((r) => setTimeout(r, 1500));
    await checkStatus();
    actionMsg.value = running.value ? "✓ 已重启" : "⚠ 重启后未检测到端口响应，请稍后刷新";
    actionMsgType.value = running.value ? "ok" : "err";
  } catch (err) {
    actionMsg.value = `重启失败：${err}`;
    actionMsgType.value = "err";
  } finally {
    restarting.value = false;
  }
}

async function refreshDaemon() {
  daemonBusy.value = true;
  try {
    await checkDaemon();
  } finally {
    daemonBusy.value = false;
  }
}

async function installDaemon() {
  daemonBusy.value = true;
  daemonMsg.value = "";
  try {
    await invoke("run_command_streaming", { cmd: "openclaw", args: ["daemon", "install"] });
    await checkDaemon();
    daemonMsg.value = "✓ 开机自启已启用，系统重启后将自动运行网关";
    daemonMsgType.value = "ok";
  } catch (err) {
    daemonMsg.value = `安装失败：${err}`;
    daemonMsgType.value = "err";
  } finally {
    daemonBusy.value = false;
  }
}

async function uninstallDaemon() {
  daemonBusy.value = true;
  daemonMsg.value = "";
  try {
    await invoke("run_command_streaming", { cmd: "openclaw", args: ["daemon", "uninstall"] });
    await checkDaemon();
    daemonMsg.value = "✓ 已取消开机自启";
    daemonMsgType.value = "ok";
  } catch (err) {
    daemonMsg.value = `取消失败：${err}`;
    daemonMsgType.value = "err";
  } finally {
    daemonBusy.value = false;
  }
}
</script>

<style scoped>
.page { max-width: 600px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 28px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }

.status-card {
  display: flex; align-items: center; gap: 14px;
  padding: 16px 20px; border-radius: 10px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
}
.status-card.running { border-color: rgba(72, 187, 120, 0.4); }
.status-card.stopped { border-color: rgba(252, 129, 129, 0.3); }
.status-dot { width: 12px; height: 12px; border-radius: 50%; flex-shrink: 0; background: var(--color-text-muted); }
.status-card.running .status-dot { background: var(--color-success); box-shadow: 0 0 0 3px rgba(72, 187, 120, 0.25); animation: pulse 2s infinite; }
.status-card.stopped .status-dot { background: var(--color-danger); }
@keyframes pulse { 0%, 100% { box-shadow: 0 0 0 3px rgba(72, 187, 120, 0.25); } 50% { box-shadow: 0 0 0 6px rgba(72, 187, 120, 0.1); } }
.status-info { flex: 1; display: flex; flex-direction: column; gap: 3px; }
.status-label { font-size: 15px; font-weight: 600; }
.status-sub { font-size: 12px; color: var(--color-text-muted); font-family: monospace; }

.action-row { display: flex; gap: 12px; align-items: center; }
.action-msg { margin-top: 10px; font-size: 12px; }
.action-msg.ok { color: var(--color-success); }
.action-msg.err { color: var(--color-danger); }
.daemon-warn { color: #f6ad55; }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; height: 160px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; }
.log-line.success { color: var(--color-success); }
.log-line.error { color: var(--color-danger); }
.log-line.info { color: var(--color-text-muted); }

.tip-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; padding: 14px 16px; font-size: 13px; line-height: 1.7; }
.tip-card p { font-weight: 600; margin-bottom: 6px; }
.tip-card ul { padding-left: 18px; color: var(--color-text-muted); }
.tip-card code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover { border-color: var(--color-primary); color: var(--color-primary); }
.btn-danger { background: rgba(252, 129, 129, 0.15); color: var(--color-danger); border: 1px solid rgba(252, 129, 129, 0.3); }
.btn-danger:hover:not(:disabled) { background: rgba(252, 129, 129, 0.25); }
.btn-danger:disabled { opacity: 0.4; cursor: not-allowed; }
.open-section { margin-top: 14px; display: flex; flex-direction: column; gap: 8px; }
.btn-open { align-self: flex-start; }
.open-url-row { display: flex; align-items: center; gap: 8px; background: #0a0c14; border: 1px solid var(--color-border); border-radius: 6px; padding: 7px 10px; }
.open-url { flex: 1; font-size: 11px; font-family: monospace; color: var(--color-text-muted); word-break: break-all; }
.copy-btn { flex-shrink: 0; padding: 3px 10px; font-size: 11px; border-radius: 4px; border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text-muted); cursor: pointer; }
.copy-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.open-warn { font-size: 11px; color: #f6ad55; margin: 0; }

/* 开机自启动 */
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.daemon-card {
  display: flex; align-items: center; gap: 14px;
  padding: 14px 18px; border-radius: 10px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
}
.daemon-card.daemon-on { border-color: rgba(72, 187, 120, 0.35); }
.daemon-card.daemon-off { border-color: var(--color-border); }
.daemon-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.daemon-card.daemon-on .daemon-dot { background: var(--color-success); }
.daemon-card.daemon-off .daemon-dot { background: var(--color-text-muted); }
.daemon-info { flex: 1; display: flex; flex-direction: column; gap: 3px; }
.daemon-label { font-size: 13px; font-weight: 600; }
.daemon-sub { font-size: 11px; color: var(--color-text-muted); }
.daemon-msg { margin-top: 8px; font-size: 12px; }
.daemon-msg.ok { color: var(--color-success); }
.daemon-msg.err { color: var(--color-danger); }
.btn-daemon-on { background: rgba(72, 187, 120, 0.12); color: var(--color-success); border: 1px solid rgba(72, 187, 120, 0.3); }
.btn-daemon-on:hover:not(:disabled) { background: rgba(72, 187, 120, 0.22); }
.btn-daemon-on:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-daemon-off { background: rgba(252, 129, 129, 0.1); color: var(--color-danger); border: 1px solid rgba(252, 129, 129, 0.25); }
.btn-daemon-off:hover:not(:disabled) { background: rgba(252, 129, 129, 0.2); }
.btn-daemon-off:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
