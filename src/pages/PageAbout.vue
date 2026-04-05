<template>
  <div class="page">
    <div class="hero">
      <span class="hero-icon">🦞</span>
      <div class="hero-info">
        <h1 class="app-name">OpenClaw 助手</h1>
        <span class="app-version">v{{ appVersion }}</span>
      </div>
    </div>
    <p class="app-desc">安装 OpenClaw 的辅助工具，帮助小白用户快速完成环境配置、插件安装与服务管理。</p>

    <section class="section">
      <h2 class="section-title">快速链接</h2>
      <div class="link-list">
        <button class="link-card" @click="openUrl('https://openclaw.ai')">
          <span class="link-icon">🌐</span>
          <div class="link-info">
            <span class="link-title">OpenClaw 官网</span>
            <span class="link-url">openclaw.ai</span>
          </div>
          <span class="link-arrow">↗</span>
        </button>
        <button class="link-card" @click="openUrl('https://docs.openclaw.ai')">
          <span class="link-icon">📖</span>
          <div class="link-info">
            <span class="link-title">官方文档</span>
            <span class="link-url">docs.openclaw.ai</span>
          </div>
          <span class="link-arrow">↗</span>
        </button>
        <button class="link-card" @click="openUrl('https://docs.openclaw.ai/cli')">
          <span class="link-icon">⌨️</span>
          <div class="link-info">
            <span class="link-title">CLI 命令参考</span>
            <span class="link-url">docs.openclaw.ai/cli</span>
          </div>
          <span class="link-arrow">↗</span>
        </button>
      </div>
    </section>

    <section class="section">
      <h2 class="section-title">安装流程速览</h2>
      <div class="flow-card">
        <div v-for="(step, i) in steps" :key="i" class="flow-step">
          <span class="flow-num">{{ i + 1 }}</span>
          <div class="flow-content">
            <span class="flow-title">{{ step.title }}</span>
            <span class="flow-desc">{{ step.desc }}</span>
          </div>
        </div>
      </div>
    </section>

    <section class="section">
      <h2 class="section-title">依赖版本</h2>
      <div class="dep-list">
        <div v-for="dep in deps" :key="dep.name" class="dep-item">
          <span class="dep-name">{{ dep.name }}</span>
          <span class="dep-version">{{ dep.version }}</span>
          <span v-if="depStatus[dep.name]" class="dep-status ok">{{ depStatus[dep.name] }}</span>
          <span v-else-if="checked" class="dep-status missing">未检测到</span>
          <span v-else class="dep-status muted">—</span>
        </div>
      </div>
      <button class="btn btn-secondary btn-sm" :disabled="checking" style="margin-top:12px" @click="checkDeps">
        {{ checking ? "检测中..." : "🔍 检测本机环境" }}
      </button>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

const appVersion = __APP_VERSION__;
const checking = ref(false);
const checked = ref(false);
const depStatus = ref<Record<string, string>>({});

const steps = [
  { title: "环境检测", desc: "确认包管理器、Git、Node.js、OpenClaw 均已安装" },
  { title: "安装 OpenClaw", desc: "通过安装页一键完成全部依赖" },
  { title: "配置模型厂商", desc: "在「模型厂商」页填写 API Key 和模型 ID" },
  { title: "启动网关", desc: "在「启动/停止」页启动 OpenClaw 网关服务" },
  { title: "启用开机自启（推荐）", desc: "安装 daemon，让网关随系统自动启动" },
  { title: "安装插件", desc: "在「插件管理」页安装所需功能插件" },
];

const deps = [
  { name: "openclaw", version: "最新" },
  { name: "node",     version: "LTS" },
  { name: "git",      version: "任意" },
];

async function openUrl(url: string) {
  await open(url);
}

async function checkDeps() {
  checking.value = true;
  depStatus.value = {};
  try {
    const results = await Promise.all(
      deps.map((d) => invoke<string>("check_command", { cmd: d.name }))
    );
    deps.forEach((d, i) => {
      const v = results[i];
      if (v) depStatus.value[d.name] = v.split("\n")[0].slice(0, 40);
    });
    checked.value = true;
  } finally {
    checking.value = false;
  }
}
</script>

<style scoped>
.page { max-width: 580px; }
.hero { display: flex; align-items: center; gap: 16px; margin-bottom: 12px; }
.hero-icon { font-size: 48px; }
.hero-info { display: flex; flex-direction: column; gap: 4px; }
.app-name { font-size: 22px; font-weight: 700; margin: 0; }
.app-version { font-size: 12px; color: var(--color-primary); background: rgba(108,99,255,0.12); padding: 2px 8px; border-radius: 4px; align-self: flex-start; }
.app-desc { color: var(--color-text-muted); font-size: 13px; line-height: 1.6; margin-bottom: 28px; }

.section { margin-bottom: 28px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }

.link-list { display: flex; flex-direction: column; gap: 6px; }
.link-card { display: flex; align-items: center; gap: 12px; padding: 12px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; cursor: pointer; text-align: left; transition: all 0.15s; width: 100%; }
.link-card:hover { border-color: var(--color-primary); }
.link-icon { font-size: 18px; flex-shrink: 0; }
.link-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.link-title { font-size: 13px; font-weight: 500; color: var(--color-text); }
.link-url { font-size: 11px; color: var(--color-text-muted); font-family: monospace; }
.link-arrow { color: var(--color-primary); font-size: 16px; }

.flow-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; overflow: hidden; }
.flow-step { display: flex; align-items: flex-start; gap: 12px; padding: 10px 14px; border-bottom: 1px solid var(--color-border); }
.flow-step:last-child { border-bottom: none; }
.flow-num { width: 20px; height: 20px; border-radius: 50%; background: rgba(108,99,255,0.15); color: var(--color-primary); font-size: 11px; font-weight: 700; display: flex; align-items: center; justify-content: center; flex-shrink: 0; margin-top: 1px; }
.flow-content { display: flex; flex-direction: column; gap: 2px; }
.flow-title { font-size: 13px; font-weight: 500; }
.flow-desc { font-size: 11px; color: var(--color-text-muted); }

.dep-list { display: flex; flex-direction: column; gap: 6px; }
.dep-item { display: flex; align-items: center; gap: 10px; padding: 8px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; }
.dep-name { font-size: 13px; font-weight: 500; width: 100px; }
.dep-version { font-size: 11px; color: var(--color-text-muted); flex: 1; }
.dep-status { font-size: 11px; font-family: monospace; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dep-status.ok { color: var(--color-success); }
.dep-status.missing { color: var(--color-danger); }
.dep-status.muted { color: var(--color-text-muted); }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
