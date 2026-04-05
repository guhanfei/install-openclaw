<template>
  <div class="page">
    <h1 class="page-title">升级 OpenClaw</h1>
    <p class="page-desc">通过 npm 将 OpenClaw 更新到最新版本。</p>

    <!-- 当前版本 -->
    <section class="section">
      <h2 class="section-title">版本信息</h2>
      <div class="version-card">
        <div class="version-row">
          <span class="version-label">当前版本</span>
          <span class="version-value">{{ currentVersion || "检测中..." }}</span>
        </div>
        <button class="btn btn-secondary btn-sm" :disabled="checking" @click="checkVersion">
          {{ checking ? "检测中..." : "🔍 重新检测" }}
        </button>
      </div>
    </section>

    <!-- 升级操作 -->
    <section class="section">
      <div class="action-row">
        <button class="btn btn-primary" :disabled="upgrading || done" @click="startUpgrade">
          {{ upgrading ? "升级中..." : done ? "✓ 已完成" : "🔄 立即升级" }}
        </button>
        <span v-if="resultMsg" class="result-msg" :class="resultType">{{ resultMsg }}</span>
      </div>

      <div v-if="logs.length > 0" class="log-box" ref="logBox">
        <div v-for="(log, i) in logs" :key="i" class="log-line" :class="log.level">
          {{ log.text }}
        </div>
      </div>
    </section>

    <!-- 说明 -->
    <section class="section">
      <div class="tip-card">
        <p>💡 <strong>说明</strong></p>
        <ul>
          <li>升级命令：<code>npm install -g openclaw</code></li>
          <li>升级后如果 OpenClaw 正在运行，需重启才能使新版本生效。</li>
        </ul>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface LogLine { text: string; level: "info" | "error" | "success" }

const currentVersion = ref("");
const checking = ref(false);
const upgrading = ref(false);
const done = ref(false);
const resultMsg = ref("");
const resultType = ref<"ok" | "err">("ok");
const logs = ref<LogLine[]>([]);
const logBox = ref<HTMLElement | null>(null);

onMounted(checkVersion);

async function checkVersion() {
  checking.value = true;
  currentVersion.value = "";
  try {
    const v = await invoke<string>("check_command", { cmd: "openclaw" });
    currentVersion.value = v ? v.split("\n")[0].slice(0, 60) : "未安装";
  } finally {
    checking.value = false;
  }
}

async function startUpgrade() {
  upgrading.value = true;
  done.value = false;
  resultMsg.value = "";
  logs.value = [];

  const unlisten = await listen<{ text: string; level: string }>("log", (e) => {
    logs.value.push({ text: e.payload.text, level: e.payload.level as LogLine["level"] });
    nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
  });

  try {
    await invoke("run_command_streaming", {
      cmd: "npm",
      args: ["install", "-g", "openclaw"],
    });
    logs.value.push({ text: "✓ 升级完成", level: "success" });
    resultMsg.value = "✓ 升级成功";
    resultType.value = "ok";
    done.value = true;
    await checkVersion();
  } catch (err) {
    resultMsg.value = `升级失败：${err}`;
    resultType.value = "err";
    logs.value.push({ text: `✗ ${err}`, level: "error" });
  } finally {
    upgrading.value = false;
    unlisten();
  }
}
</script>

<style scoped>
.page { max-width: 600px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 28px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }

.version-card { display: flex; align-items: center; gap: 16px; padding: 14px 16px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; }
.version-row { flex: 1; display: flex; flex-direction: column; gap: 3px; }
.version-label { font-size: 11px; color: var(--color-text-muted); }
.version-value { font-size: 13px; font-family: monospace; font-weight: 500; }

.action-row { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
.result-msg { font-size: 12px; }
.result-msg.ok { color: var(--color-success); }
.result-msg.err { color: var(--color-danger); }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; height: 200px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; word-break: break-all; }
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
.btn-secondary:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
