<template>
  <div class="page">
    <h1 class="page-title">卸载 OpenClaw</h1>
    <p class="page-desc">完全移除 OpenClaw 及其相关配置。</p>

    <section class="section">
      <div class="warning-card">
        <span class="warning-icon">⚠️</span>
        <div>
          <p class="warning-title">此操作不可撤销</p>
          <p class="warning-desc">将执行 <code>npm uninstall -g openclaw</code>，并可选删除 <code>~/.openclaw</code> 配置目录。</p>
        </div>
      </div>
    </section>

    <section class="section">
      <h2 class="section-title">卸载选项</h2>
      <label class="checkbox-row">
        <input v-model="removeConfig" type="checkbox" :disabled="uninstalling" />
        <span>同时删除 <code>~/.openclaw</code> 配置目录（包含所有 skill、插件、模型配置）</span>
      </label>
    </section>

    <section class="section">
      <div v-if="!confirmed">
        <p class="confirm-prompt">请输入 <code>确认卸载</code> 以继续：</p>
        <div class="confirm-row">
          <input v-model="confirmInput" type="text" placeholder="确认卸载" class="input" />
          <button class="btn btn-danger" :disabled="confirmInput !== '确认卸载'" @click="confirmed = true">
            确认
          </button>
        </div>
      </div>
      <div v-else>
        <button class="btn btn-danger-solid" :disabled="uninstalling || done" @click="startUninstall">
          {{ uninstalling ? "卸载中..." : done ? "✓ 已卸载" : "🗑️ 开始卸载" }}
        </button>
      </div>

      <div v-if="logs.length > 0" class="log-box" ref="logBox" style="margin-top: 16px;">
        <div v-for="(log, i) in logs" :key="i" class="log-line" :class="log.level">{{ log.text }}</div>
      </div>
      <p v-if="errorMsg" class="error-hint">{{ errorMsg }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface LogLine {
  text: string;
  level: "info" | "error" | "success";
}

const removeConfig = ref(false);
const confirmInput = ref("");
const confirmed = ref(false);
const uninstalling = ref(false);
const done = ref(false);
const logs = ref<LogLine[]>([]);
const errorMsg = ref("");
const logBox = ref<HTMLElement | null>(null);

function addLog(text: string, level: LogLine["level"]) {
  logs.value.push({ text, level });
  nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
}

async function startUninstall() {
  uninstalling.value = true;
  errorMsg.value = "";
  logs.value = [];

  const unlisten = await listen<{ text: string; level: string }>("log", (e) => {
    addLog(e.payload.text, e.payload.level as LogLine["level"]);
  });

  try {
    addLog("▶ 开始卸载 openclaw...", "info");
    await invoke("run_command_streaming", { cmd: "npm", args: ["uninstall", "-g", "openclaw"] });
    addLog("✓ openclaw 已卸载", "success");

    if (removeConfig.value) {
      addLog("▶ 删除 ~/.openclaw 目录...", "info");
      await invoke("delete_config_dir");
      addLog("✓ ~/.openclaw 已删除", "success");
    }

    done.value = true;
  } catch (err) {
    errorMsg.value = `卸载失败：${err}`;
    addLog(`✗ 失败：${err}`, "error");
  } finally {
    uninstalling.value = false;
    unlisten();
  }
}
</script>

<style scoped>
.page { max-width: 600px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }
.error-hint { margin-top: 8px; font-size: 12px; color: var(--color-danger); }

.warning-card { display: flex; gap: 14px; padding: 16px; background: rgba(252, 129, 129, 0.08); border: 1px solid rgba(252, 129, 129, 0.3); border-radius: 8px; }
.warning-icon { font-size: 24px; flex-shrink: 0; }
.warning-title { font-size: 13px; font-weight: 600; color: var(--color-danger); margin-bottom: 4px; }
.warning-desc { font-size: 12px; color: var(--color-text-muted); }
.warning-desc code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; }

.checkbox-row { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; line-height: 1.5; }
.checkbox-row input[type="checkbox"] { width: 14px; height: 14px; flex-shrink: 0; margin: 0; cursor: pointer; }
.checkbox-row code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 12px; }

.confirm-prompt { font-size: 13px; margin-bottom: 10px; color: var(--color-text-muted); }
.confirm-prompt code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; }
.confirm-row { display: flex; gap: 10px; }
.input { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--color-danger); }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; height: 160px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; }
.log-line.success { color: var(--color-success); }
.log-line.error { color: var(--color-danger); }
.log-line.info { color: var(--color-text-muted); }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-danger { background: rgba(252, 129, 129, 0.15); color: var(--color-danger); border: 1px solid rgba(252, 129, 129, 0.3); }
.btn-danger:hover:not(:disabled) { background: rgba(252, 129, 129, 0.25); }
.btn-danger:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-danger-solid { background: var(--color-danger); color: #fff; }
.btn-danger-solid:hover:not(:disabled) { background: #f56565; }
.btn-danger-solid:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
