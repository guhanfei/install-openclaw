<template>
  <div class="page">
    <h1 class="page-title">升级 OpenClaw</h1>
    <p class="page-desc">检查并更新到最新版本。</p>

    <!-- 版本状态卡片 -->
    <section class="section">
      <div class="version-card">
        <!-- 当前版本 -->
        <div class="version-item">
          <span class="version-item-label">当前版本</span>
          <span class="version-item-value" :class="{ muted: !currentVersion }">
            {{ currentVersion || (checking ? '检测中...' : '未安装') }}
          </span>
        </div>

        <div class="version-divider">→</div>

        <!-- 最新版本 -->
        <div class="version-item">
          <span class="version-item-label">npm 最新版本</span>
          <span class="version-item-value" :class="{ muted: !latestVersion }">
            {{ latestVersion || (checking ? '查询中...' : '—') }}
          </span>
        </div>

        <!-- 状态徽章 -->
        <div class="version-badge-wrap">
          <span v-if="checking" class="status-badge checking">检测中</span>
          <span v-else-if="versionStatus === 'latest'" class="status-badge latest">✓ 已是最新</span>
          <span v-else-if="versionStatus === 'outdated'" class="status-badge outdated">有新版本</span>
          <span v-else-if="versionStatus === 'unknown'" class="status-badge unknown">无法比较</span>
        </div>

        <button class="btn btn-secondary btn-sm refresh-btn" :disabled="checking" @click="checkVersions">
          {{ checking ? '...' : '🔍' }}
        </button>
      </div>

      <!-- 提示文字 -->
      <p v-if="!checking && versionStatus === 'latest'" class="hint-text ok">
        🎉 你已经在使用最新版本，无需升级。
      </p>
      <p v-else-if="!checking && versionStatus === 'outdated'" class="hint-text warn">
        🆕 发现新版本 <strong>{{ latestVersion }}</strong>，当前为 {{ currentVersion }}，建议立即升级。
      </p>
    </section>

    <!-- 操作区 -->
    <section class="section">
      <div class="action-row">
        <button
          class="btn btn-primary"
          :disabled="upgrading || !canUpgrade"
          @click="startUpgrade"
        >
          <span v-if="upgrading">升级中...</span>
          <span v-else-if="done">✓ 升级完成</span>
          <span v-else-if="versionStatus === 'latest'">🔄 重新安装当前版本</span>
          <span v-else>🔄 升级到 {{ latestVersion || '最新版' }}</span>
        </button>
        <span v-if="resultMsg" class="result-msg" :class="resultType">{{ resultMsg }}</span>
      </div>

      <!-- 日志 -->
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
          <li>执行 <code>openclaw update</code> 完成升级，会自动检测安装方式并重启 gateway。</li>
          <li>升级过程中 OpenClaw 会短暂中断，完成后自动恢复运行。</li>
        </ul>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface LogLine { text: string; level: "info" | "error" | "success" }

const currentVersion = ref("");
const latestVersion = ref("");
const checking = ref(false);
const upgrading = ref(false);
const done = ref(false);
const resultMsg = ref("");
const resultType = ref<"ok" | "err">("ok");
const logs = ref<LogLine[]>([]);
const logBox = ref<HTMLElement | null>(null);

// 从版本字符串里提取纯数字版本号，例如 "openclaw/1.2.3 node/20.0.0" → "1.2.3"
function extractVersion(raw: string): string {
  const m = raw.match(/(\d+\.\d+\.\d+[\w.-]*)/);
  return m ? m[1] : raw.split("\n")[0].trim();
}

const versionStatus = computed<"latest" | "outdated" | "unknown" | "">(() => {
  if (!currentVersion.value || !latestVersion.value) return "";
  const cur = extractVersion(currentVersion.value);
  const latest = extractVersion(latestVersion.value);
  if (cur === latest) return "latest";
  // 简单语义化版本比较
  const toNum = (v: string) => v.split(".").map((n) => parseInt(n) || 0);
  const [ca, cb, cc] = toNum(cur);
  const [la, lb, lc] = toNum(latest);
  if (la > ca || (la === ca && lb > cb) || (la === ca && lb === cb && lc > cc)) return "outdated";
  return "unknown"; // 本地比 npm 还新（pre-release 等）
});

const canUpgrade = computed(() => !checking.value && !!latestVersion.value);

onMounted(checkVersions);

async function checkVersions() {
  checking.value = true;
  currentVersion.value = "";
  latestVersion.value = "";
  done.value = false;
  resultMsg.value = "";

  try {
    const [cur, latest] = await Promise.all([
      invoke<string>("check_command", { cmd: "openclaw" }),
      invoke<string>("run_command_output", { cmd: "npm", args: ["view", "openclaw", "version"] }),
    ]);
    currentVersion.value = cur || "未安装";
    latestVersion.value = latest.trim();
  } catch {
    currentVersion.value = currentVersion.value || "检测失败";
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
    await invoke("run_command_streaming", { cmd: "openclaw", args: ["update", "--yes"] });
    logs.value.push({ text: "✓ 升级完成", level: "success" });
    resultMsg.value = "✓ 升级成功";
    resultType.value = "ok";
    done.value = true;
    // 重新检测版本，验证升级结果
    await checkVersions();
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
.page { max-width: 620px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 28px; }

/* 版本卡片 */
.version-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 18px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  margin-bottom: 10px;
}
.version-item { display: flex; flex-direction: column; gap: 4px; min-width: 120px; }
.version-item-label { font-size: 11px; color: var(--color-text-muted); }
.version-item-value { font-size: 14px; font-family: monospace; font-weight: 600; }
.version-item-value.muted { color: var(--color-text-muted); font-weight: 400; }
.version-divider { font-size: 18px; color: var(--color-text-muted); flex-shrink: 0; }
.version-badge-wrap { flex: 1; display: flex; justify-content: center; }
.refresh-btn { flex-shrink: 0; }

/* 状态徽章 */
.status-badge { display: inline-flex; align-items: center; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 500; }
.status-badge.latest   { background: rgba(72, 187, 120, 0.15); color: var(--color-success); border: 1px solid rgba(72, 187, 120, 0.3); }
.status-badge.outdated { background: rgba(246, 173, 85, 0.15);  color: #f6ad55;              border: 1px solid rgba(246, 173, 85, 0.3); }
.status-badge.checking { background: rgba(108, 99, 255, 0.1);   color: var(--color-primary);  border: 1px solid rgba(108, 99, 255, 0.2); }
.status-badge.unknown  { background: var(--color-border);        color: var(--color-text-muted); }

/* 提示文字 */
.hint-text { font-size: 13px; padding: 10px 14px; border-radius: 7px; margin: 0; }
.hint-text.ok   { background: rgba(72, 187, 120, 0.08); color: var(--color-success); border: 1px solid rgba(72, 187, 120, 0.2); }
.hint-text.warn { background: rgba(246, 173, 85, 0.08);  color: #f6ad55;              border: 1px solid rgba(246, 173, 85, 0.2); }

/* 操作区 */
.action-row { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
.result-msg { font-size: 12px; }
.result-msg.ok { color: var(--color-success); }
.result-msg.err { color: var(--color-danger); }

.log-box { background: #0a0c14; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; height: 200px; overflow-y: auto; font-family: monospace; font-size: 12px; }
.log-line { line-height: 1.7; white-space: pre-wrap; word-break: break-all; }
.log-line.success { color: var(--color-success); }
.log-line.error   { color: var(--color-danger); }
.log-line.info    { color: var(--color-text-muted); }

/* 说明卡片 */
.tip-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; padding: 14px 16px; font-size: 13px; line-height: 1.7; }
.tip-card p { font-weight: 600; margin-bottom: 6px; }
.tip-card ul { padding-left: 18px; color: var(--color-text-muted); }
.tip-card code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; }

/* 按钮 */
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 10px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
