<template>
  <div class="page">
    <h1 class="page-title">插件管理</h1>
    <p class="page-desc">
      插件安装在 <code>~/.openclaw/extensions/</code>，配置记录在
      <code>openclaw.json</code> 的 <code>plugins</code> 节点。
    </p>

    <!-- 安装新插件 -->
    <section class="section">
      <h2 class="section-title">安装新插件</h2>
      <div class="install-row">
        <input
          v-model="newPkg"
          type="text"
          placeholder="npm 包名，例如 @wecom/wecom-openclaw-plugin"
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

    <!-- 已安装插件 -->
    <section class="section">
      <div class="toolbar">
        <h2 class="section-title">已安装插件</h2>
        <button class="btn btn-secondary btn-sm" @click="loadPlugins">🔄 刷新</button>
      </div>
      <div v-if="loading" class="empty-state">读取中...</div>
      <div v-else-if="pluginList.length === 0" class="empty-state">暂无已安装插件</div>
      <div v-else class="list">
        <div v-for="plugin in pluginList" :key="plugin.id" class="list-item">
          <div class="item-info">
            <span class="item-name">{{ plugin.id }}</span>
            <span class="item-version">{{ plugin.version }}</span>
          </div>
          <div class="item-meta">
            <span class="item-spec">{{ plugin.spec }}</span>
          </div>
          <!-- 启用/禁用 toggle -->
          <label class="toggle" :title="plugin.enabled ? '点击禁用' : '点击启用'">
            <input type="checkbox" :checked="plugin.enabled" @change="togglePlugin(plugin.id, !plugin.enabled)" />
            <span class="toggle-slider"></span>
          </label>
          <button
            class="btn btn-danger btn-sm"
            :disabled="busy === plugin.id"
            @click="uninstallPlugin(plugin.id)"
          >
            {{ busy === plugin.id ? "卸载中..." : "卸载" }}
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

interface PluginInstall {
  id: string;
  spec: string;
  version: string;
  installPath: string;
  enabled: boolean;
}

interface LogLine { text: string; level: "info" | "error" | "success" }

// openclaw.json 的 plugins 节点
interface PluginsConfig {
  load?: { paths?: string[] };
  allow?: string[];
  entries?: Record<string, { enabled: boolean; [key: string]: unknown }>;
  installs?: Record<string, {
    source?: string;
    spec?: string;
    installPath?: string;
    version?: string;
    [key: string]: unknown;
  }>;
}

interface OpencLawConfig {
  plugins?: PluginsConfig;
  [key: string]: unknown;
}

const loading = ref(false);
const busy = ref("");
const newPkg = ref("");
const logs = ref<LogLine[]>([]);
const errorMsg = ref("");
const logBox = ref<HTMLElement | null>(null);
const pluginList = ref<PluginInstall[]>([]);
const rawConfig = ref<OpencLawConfig>({});

onMounted(loadPlugins);

async function loadPlugins() {
  loading.value = true;
  try {
    const raw = await invoke<string | null>("read_config", { filename: "openclaw.json" });
    if (!raw) { pluginList.value = []; return; }
    rawConfig.value = JSON.parse(raw) as OpencLawConfig;
    buildPluginList();
  } finally {
    loading.value = false;
  }
}

function buildPluginList() {
  const installs = rawConfig.value.plugins?.installs ?? {};
  const allow = rawConfig.value.plugins?.allow ?? [];
  pluginList.value = Object.entries(installs).map(([id, info]) => ({
    id,
    spec: info.spec ?? "",
    version: info.version ?? "",
    installPath: info.installPath ?? "",
    enabled: allow.includes(id),
  }));
}

async function persistConfig() {
  await invoke("write_config", {
    filename: "openclaw.json",
    content: JSON.stringify(rawConfig.value, null, 2),
  });
}

async function togglePlugin(id: string, enable: boolean) {
  if (!rawConfig.value.plugins) rawConfig.value.plugins = {};
  const cfg = rawConfig.value.plugins;

  // 更新 allow 列表
  let allow = cfg.allow ?? [];
  if (enable) {
    if (!allow.includes(id)) allow = [...allow, id];
  } else {
    allow = allow.filter((a) => a !== id);
  }
  cfg.allow = allow;

  // 更新 entries
  if (!cfg.entries) cfg.entries = {};
  if (!cfg.entries[id]) cfg.entries[id] = { enabled: enable };
  else cfg.entries[id].enabled = enable;

  rawConfig.value = { ...rawConfig.value, plugins: cfg };
  buildPluginList();

  try {
    await persistConfig();
  } catch (err) {
    errorMsg.value = `保存失败：${err}`;
  }
}

async function installPlugin() {
  const pkg = newPkg.value.trim();
  if (!pkg) return;

  const pluginId = pkg;                              // 完整包名作为配置 key
  const dirName = pkg.split("/").pop() ?? pkg;       // 仅用于文件系统路径

  busy.value = "install";
  logs.value = [];
  errorMsg.value = "";

  const extensionsDir = `${await homeDir()}/.openclaw/extensions/${dirName}`;

  const unlisten = await listen<{ text: string; level: string }>("log", (e) => {
    logs.value.push({ text: e.payload.text, level: e.payload.level as LogLine["level"] });
    nextTick(() => { if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight; });
  });

  try {
    // npm install --prefix <path> <pkg>
    await invoke("run_command_streaming", {
      cmd: "npm",
      args: ["install", "--prefix", extensionsDir, pkg],
    });

    // 更新 openclaw.json
    if (!rawConfig.value.plugins) rawConfig.value.plugins = {};
    const cfg = rawConfig.value.plugins;
    if (!cfg.installs) cfg.installs = {};
    if (!cfg.allow) cfg.allow = [];
    if (!cfg.entries) cfg.entries = {};

    cfg.installs[pluginId] = {
      source: "npm",
      spec: pkg,
      installPath: extensionsDir,
      version: "latest",
      installedAt: new Date().toISOString(),
    };
    if (!cfg.allow.includes(pluginId)) cfg.allow.push(pluginId);
    cfg.entries[pluginId] = { enabled: true };

    rawConfig.value = { ...rawConfig.value, plugins: cfg };
    await persistConfig();
    buildPluginList();

    newPkg.value = "";
    logs.value.push({ text: `✓ ${pluginId} 安装成功`, level: "success" });
  } catch (err) {
    errorMsg.value = `安装失败：${err}`;
    logs.value.push({ text: `✗ 失败：${err}`, level: "error" });
  } finally {
    busy.value = "";
    unlisten();
  }
}

async function uninstallPlugin(id: string) {
  busy.value = id;
  logs.value = [];
  errorMsg.value = "";

  const plugin = pluginList.value.find((p) => p.id === id);
  const installPath = plugin?.installPath || `${await homeDir()}/.openclaw/extensions/${id}`;

  try {
    // 用 Rust 跨平台删除目录（macOS/Windows 均可用，不需要 rm -rf）
    await invoke("delete_dir", { path: installPath });

    // 从 openclaw.json 中移除
    const cfg = rawConfig.value.plugins ?? {};
    delete cfg.installs?.[id];
    cfg.allow = (cfg.allow ?? []).filter((a) => a !== id);
    delete cfg.entries?.[id];
    rawConfig.value = { ...rawConfig.value, plugins: cfg };
    await persistConfig();
    buildPluginList();

    logs.value.push({ text: `✓ ${id} 已卸载`, level: "success" });
  } catch (err) {
    errorMsg.value = `卸载失败：${err}`;
  } finally {
    busy.value = "";
  }
}

async function homeDir(): Promise<string> {
  return invoke<string>("get_home_dir");
}
</script>

<style scoped>
.page { max-width: 700px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; line-height: 1.6; }
.page-desc code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
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
.item-meta { flex: 1; }
.item-spec { font-size: 11px; color: var(--color-text-muted); font-family: monospace; }

/* Toggle 开关 */
.toggle { position: relative; width: 36px; height: 20px; cursor: pointer; flex-shrink: 0; }
.toggle input { opacity: 0; width: 0; height: 0; }
.toggle-slider { position: absolute; inset: 0; background: var(--color-border); border-radius: 10px; transition: background 0.2s; }
.toggle-slider::before { content: ""; position: absolute; width: 14px; height: 14px; left: 3px; top: 3px; background: #fff; border-radius: 50%; transition: transform 0.2s; }
.toggle input:checked + .toggle-slider { background: var(--color-primary); }
.toggle input:checked + .toggle-slider::before { transform: translateX(16px); }

.input { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--color-primary); }
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover { border-color: var(--color-primary); color: var(--color-primary); }
.btn-danger { background: rgba(252, 129, 129, 0.15); color: var(--color-danger); border: 1px solid rgba(252, 129, 129, 0.3); }
.btn-danger:hover:not(:disabled) { background: rgba(252, 129, 129, 0.25); }
.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
