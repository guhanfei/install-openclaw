<template>
  <div class="page">
    <div class="page-header">
      <div>
        <h1 class="page-title">检测本机电脑</h1>
        <p class="page-desc">快速了解本机配置与 OpenClaw 数据目录位置。</p>
      </div>
      <button class="btn btn-secondary btn-sm" :disabled="loading" @click="loadInfo">
        {{ loading ? "检测中..." : "🔄 重新检测" }}
      </button>
    </div>

    <div v-if="loading" class="loading-state">
      <span class="spinner">⟳</span> 检测中，请稍候...
    </div>

    <template v-else-if="info">
      <!-- 系统信息 -->
      <section class="section">
        <h2 class="section-title">系统信息</h2>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">操作系统</span>
            <span class="info-value">{{ osLabel }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">系统版本</span>
            <span class="info-value mono">{{ info.os_version || '—' }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">CPU 架构</span>
            <span class="info-value">
              {{ archLabel }}
              <span v-if="archBadge" class="arch-badge" :class="archBadgeClass">{{ archBadge }}</span>
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">处理器</span>
            <span class="info-value mono small">{{ info.cpu_brand || '—' }}</span>
          </div>
        </div>
      </section>

      <!-- 内存 -->
      <section class="section">
        <h2 class="section-title">内存（RAM）</h2>
        <div class="resource-card">
          <div class="resource-top">
            <span class="resource-main">{{ formatBytes(info.total_memory_bytes) }}</span>
            <span class="resource-badge" :class="memBadgeClass">{{ memHint }}</span>
          </div>
          <div class="res-bar">
            <div class="res-bar-fill" :class="memBadgeClass" style="width: 100%"></div>
          </div>
        </div>
      </section>

      <!-- 磁盘 -->
      <section class="section">
        <h2 class="section-title">磁盘空间</h2>
        <div class="resource-card">
          <div class="resource-top">
            <span class="resource-main">
              <span class="disk-free">{{ formatBytes(info.disk_free_bytes) }}</span>
              <span class="disk-sep"> / </span>
              <span class="disk-total">{{ formatBytes(info.disk_total_bytes) }}</span>
            </span>
            <span class="resource-badge" :class="diskBadgeClass">已用 {{ diskUsedPct }}%</span>
          </div>
          <div class="res-bar">
            <div class="res-bar-fill" :class="diskBadgeClass" :style="{ width: diskUsedPct + '%' }"></div>
          </div>
          <span class="resource-sub">可用 {{ formatBytes(info.disk_free_bytes) }}</span>
        </div>
      </section>

      <!-- OpenClaw 数据目录 -->
      <section class="section">
        <h2 class="section-title">OpenClaw 数据目录</h2>
        <div class="dir-card" :class="info.openclaw_dir_exists ? 'dir-exists' : 'dir-missing'">
          <div class="dir-dot"></div>
          <div class="dir-body">
            <span class="dir-status">
              {{ info.openclaw_dir_exists ? '目录已存在（已安装）' : '目录不存在（尚未安装）' }}
            </span>
            <span class="dir-path">{{ info.openclaw_dir }}</span>
          </div>
          <button
            v-if="info.openclaw_dir_exists"
            class="btn btn-sm btn-secondary"
            @click="openDir"
          >
            📂 打开文件夹
          </button>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface SystemInfo {
  os: string;
  os_version: string;
  arch: string;
  cpu_brand: string;
  total_memory_bytes: number;
  disk_total_bytes: number;
  disk_free_bytes: number;
  openclaw_dir: string;
  openclaw_dir_exists: boolean;
}

const loading = ref(false);
const info = ref<SystemInfo | null>(null);

onMounted(loadInfo);

async function loadInfo() {
  loading.value = true;
  try {
    info.value = await invoke<SystemInfo>("get_system_info");
  } finally {
    loading.value = false;
  }
}

async function openDir() {
  if (info.value) await invoke("open_dir", { path: info.value.openclaw_dir });
}

// ── 展示计算 ──────────────────────────────────────────────

const osLabel = computed(() => {
  const map: Record<string, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };
  return map[info.value?.os ?? ""] ?? info.value?.os ?? "—";
});

const archLabel = computed(() => {
  const a = info.value?.arch ?? "";
  if (a === "arm64") return "arm64";
  if (a === "x86_64") return "x86_64";
  return a || "—";
});

const archBadge = computed(() => {
  const a = info.value?.arch ?? "";
  const os = info.value?.os ?? "";
  if (os === "macos" && a === "arm64") return "Apple Silicon";
  if (os === "macos" && a === "x86_64") return "Intel";
  if (a === "AMD64" || a === "x86_64") return "64 位";
  return "";
});

const archBadgeClass = computed(() => {
  const a = info.value?.arch ?? "";
  const os = info.value?.os ?? "";
  if (os === "macos" && a === "arm64") return "badge-purple";
  return "badge-gray";
});

function formatBytes(bytes: number): string {
  if (!bytes) return "—";
  if (bytes >= 1024 ** 3) return (bytes / 1024 ** 3).toFixed(1) + " GB";
  if (bytes >= 1024 ** 2) return (bytes / 1024 ** 2).toFixed(0) + " MB";
  return (bytes / 1024).toFixed(0) + " KB";
}

const memGb = computed(() => (info.value?.total_memory_bytes ?? 0) / 1024 ** 3);

const memHint = computed(() => {
  const gb = memGb.value;
  if (gb >= 16) return "充足";
  if (gb >= 8)  return "满足要求";
  if (gb >= 4)  return "勉强够用";
  return "可能不足";
});

const memBadgeClass = computed(() => {
  const gb = memGb.value;
  if (gb >= 8)  return "badge-green";
  if (gb >= 4)  return "badge-yellow";
  return "badge-red";
});

const diskUsedPct = computed(() => {
  const t = info.value?.disk_total_bytes ?? 0;
  const f = info.value?.disk_free_bytes ?? 0;
  if (!t) return 0;
  return Math.round(((t - f) / t) * 100);
});

const diskFreeGb = computed(() => (info.value?.disk_free_bytes ?? 0) / 1024 ** 3);

const diskBadgeClass = computed(() => {
  if (diskFreeGb.value >= 20) return "badge-green";
  if (diskFreeGb.value >= 10) return "badge-yellow";
  return "badge-red";
});
</script>

<style scoped>
.page { max-width: 600px; }
.page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 28px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); font-size: 13px; }
.section { margin-bottom: 24px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 10px; }

.loading-state { display: flex; align-items: center; gap: 8px; color: var(--color-text-muted); font-size: 13px; padding: 40px 0; }
.spinner { display: inline-block; animation: spin 1s linear infinite; font-size: 18px; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* 系统信息表格 */
.info-grid { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 10px; overflow: hidden; }
.info-row { display: flex; align-items: center; padding: 10px 16px; border-bottom: 1px solid var(--color-border); gap: 12px; }
.info-row:last-child { border-bottom: none; }
.info-label { font-size: 12px; color: var(--color-text-muted); width: 80px; flex-shrink: 0; }
.info-value { font-size: 13px; font-weight: 500; flex: 1; display: flex; align-items: center; gap: 8px; }
.info-value.mono { font-family: monospace; font-weight: 400; font-size: 12px; }
.info-value.small { font-size: 11px; }

.arch-badge { font-size: 10px; padding: 2px 7px; border-radius: 4px; font-weight: 600; }
.badge-purple { background: rgba(108, 99, 255, 0.15); color: var(--color-primary); border: 1px solid rgba(108, 99, 255, 0.25); }
.badge-gray   { background: var(--color-border); color: var(--color-text-muted); }

/* 资源卡片 */
.resource-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 10px; padding: 14px 16px; }
.resource-top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.resource-main { font-size: 20px; font-weight: 700; display: flex; align-items: baseline; gap: 4px; }
.disk-free { font-size: 20px; font-weight: 700; }
.disk-sep { font-size: 14px; color: var(--color-text-muted); }
.disk-total { font-size: 13px; color: var(--color-text-muted); font-weight: 400; }
.resource-sub { font-size: 11px; color: var(--color-text-muted); margin-top: 6px; display: block; }

.resource-badge { font-size: 11px; font-weight: 600; padding: 3px 10px; border-radius: 20px; }
.badge-green  { background: rgba(72, 187, 120, 0.15); color: var(--color-success); border: 1px solid rgba(72, 187, 120, 0.3); }
.badge-yellow { background: rgba(246, 173, 85, 0.15); color: #f6ad55;              border: 1px solid rgba(246, 173, 85, 0.3); }
.badge-red    { background: rgba(252, 129, 129, 0.15); color: var(--color-danger);  border: 1px solid rgba(252, 129, 129, 0.3); }

.res-bar { height: 6px; background: var(--color-border); border-radius: 3px; overflow: hidden; }
.res-bar-fill { height: 100%; border-radius: 3px; transition: width 0.6s ease; }
.res-bar-fill.badge-green  { background: var(--color-success); }
.res-bar-fill.badge-yellow { background: #f6ad55; }
.res-bar-fill.badge-red    { background: var(--color-danger); }

/* OpenClaw 目录 */
.dir-card { display: flex; align-items: center; gap: 14px; padding: 14px 16px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 10px; }
.dir-card.dir-exists  { border-color: rgba(72, 187, 120, 0.35); }
.dir-card.dir-missing { border-color: var(--color-border); }
.dir-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.dir-card.dir-exists  .dir-dot { background: var(--color-success); }
.dir-card.dir-missing .dir-dot { background: var(--color-text-muted); }
.dir-body { flex: 1; display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.dir-status { font-size: 13px; font-weight: 600; }
.dir-path { font-size: 11px; font-family: monospace; color: var(--color-text-muted); word-break: break-all; }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
