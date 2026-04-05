<template>
  <div class="page">
    <h1 class="page-title">插件管理</h1>
    <p class="page-desc">通过 npm 安装/卸载 OpenClaw 插件，并同步修改 ~/.openclaw/plugins.json。</p>

    <section class="section">
      <h2 class="section-title">安装新插件</h2>
      <div class="install-row">
        <input
          v-model="newPkg"
          type="text"
          placeholder="npm 包名，例如 openclaw-plugin-xxx"
          class="input"
          :disabled="busy !== ''"
          @keyup.enter="installPlugin"
        />
        <button class="btn btn-primary" :disabled="!newPkg.trim() || busy !== ''" @click="installPlugin">
          {{ busy === 'install' ? "安装中..." : "安装" }}
        </button>
      </div>
      <div v-if="logs.length > 0" class="log-box" ref="logBox">
        <div v-for="(log, i) in logs" :key="i" class="log-line" :class="log.level">{{ log.text }}</div>
      </div>
      <p v-if="errorMsg" class="error-hint">{{ errorMsg }}</p>
    </section>

    <section class="section">
      <h2 class="section-title">已安装插件</h2>
      <div v-if="loading" class="empty-state">加载中...</div>
      <div v-else-if="plugins.length === 0" class="empty-state">暂无已安装插件</div>
      <div v-else class="list">
        <div v-for="plugin in plugins" :key="plugin.name" class="list-item">
          <div class="item-info">
            <span class="item-name">{{ plugin.name }}</span>
            <span class="item-version">{{ plugin.version }}</span>
          </div>
          <span class="item-desc">{{ plugin.description }}</span>
          <button
            class="btn btn-danger btn-sm"
            :disabled="busy !== '' && busy === plugin.name"
            @click="uninstallPlugin(plugin.name)"
          >
            {{ busy === plugin.name ? "卸载中..." : "卸载" }}
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Plugin {
  name: string;
  version: string;
  description: string;
}

interface LogLine {
  text: string;
  level: "info" | "error" | "success";
}

const plugins = ref<Plugin[]>([]);
const newPkg = ref("");
const loading = ref(false);
const busy = ref<string>("");  // "" = 空闲 | "install" | pluginName
const logs = ref<LogLine[]>([]);
const errorMsg = ref("");
const logBox = ref<HTMLElement | null>(null);

onMounted(loadPlugins);

async function loadPlugins() {
  loading.value = true;
  try {
    const raw = await invoke<string | null>("read_config", { filename: "plugins.json" });
    plugins.value = raw ? (JSON.parse(raw) as Plugin[]) : [];
  } catch {
    plugins.value = [];
  } finally {
    loading.value = false;
  }
}

async function persistPlugins() {
  await invoke("write_config", {
    filename: "plugins.json",
    content: JSON.stringify(plugins.value, null, 2),
  });
}

async function installPlugin() {
  const pkg = newPkg.value.trim();
  if (!pkg) return;

  busy.value = "install";
  logs.value = [];
  errorMsg.value = "";

  const unlisten = await listen<{ text: string; level: string }>("log", (e) => {
    logs.value.push({ text: e.payload.text, level: e.payload.level as LogLine["level"] });
    nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
  });

  try {
    await invoke("run_command_streaming", { cmd: "npm", args: ["install", "-g", pkg] });
    // 安装成功，追加到列表（版本暂时用 "latest"，下次启动可刷新）
    if (!plugins.value.some((p) => p.name === pkg)) {
      plugins.value.push({ name: pkg, version: "latest", description: "" });
      await persistPlugins();
    }
    newPkg.value = "";
    logs.value.push({ text: `✓ ${pkg} 安装成功`, level: "success" });
  } catch (err) {
    errorMsg.value = `安装失败：${err}`;
    logs.value.push({ text: `✗ 失败：${err}`, level: "error" });
  } finally {
    busy.value = "";
    unlisten();
  }
}

async function uninstallPlugin(name: string) {
  busy.value = name;
  logs.value = [];
  errorMsg.value = "";

  const unlisten = await listen<{ text: string; level: string }>("log", (e) => {
    logs.value.push({ text: e.payload.text, level: e.payload.level as LogLine["level"] });
    nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
  });

  try {
    await invoke("run_command_streaming", { cmd: "npm", args: ["uninstall", "-g", name] });
    plugins.value = plugins.value.filter((p) => p.name !== name);
    await persistPlugins();
    logs.value.push({ text: `✓ ${name} 已卸载`, level: "success" });
  } catch (err) {
    errorMsg.value = `卸载失败：${err}`;
  } finally {
    busy.value = "";
    unlisten();
  }
}
</script>

<style scoped>
.page { max-width: 680px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }
.error-hint { margin-top: 8px; font-size: 12px; color: var(--color-danger); }

.install-row { display: flex; gap: 10px; margin-bottom: 8px; }
.install-row .input { flex: 1; }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; margin-bottom: 8px; height: 160px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; word-break: break-all; }
.log-line.success { color: var(--color-success); }
.log-line.error { color: var(--color-danger); }
.log-line.info { color: var(--color-text-muted); }

.empty-state { padding: 32px; text-align: center; color: var(--color-text-muted); font-size: 13px; background: var(--color-surface); border: 1px dashed var(--color-border); border-radius: 8px; }
.list { display: flex; flex-direction: column; gap: 6px; }
.list-item { display: flex; align-items: center; gap: 12px; padding: 10px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; }
.item-info { display: flex; flex-direction: column; gap: 2px; min-width: 160px; }
.item-name { font-size: 13px; font-weight: 500; }
.item-version { font-size: 11px; color: var(--color-primary); }
.item-desc { flex: 1; font-size: 11px; color: var(--color-text-muted); }

.input { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--color-primary); }
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger { background: rgba(252, 129, 129, 0.15); color: var(--color-danger); border: 1px solid rgba(252, 129, 129, 0.3); }
.btn-danger:hover:not(:disabled) { background: rgba(252, 129, 129, 0.25); }
.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
