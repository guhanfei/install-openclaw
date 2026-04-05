<template>
  <div class="page">
    <h1 class="page-title">模型厂商</h1>
    <p class="page-desc">切换 OpenClaw 使用的 AI 模型，实际修改 ~/.openclaw/config.json。</p>

    <section class="section">
      <h2 class="section-title">选择厂商</h2>
      <div class="model-grid">
        <div
          v-for="provider in providers"
          :key="provider.id"
          class="model-card"
          :class="{ active: form.provider === provider.id }"
          @click="selectProvider(provider)"
        >
          <span class="model-icon">{{ provider.icon }}</span>
          <div class="model-info">
            <span class="model-name">{{ provider.name }}</span>
            <span class="model-desc">{{ provider.desc }}</span>
          </div>
          <span v-if="form.provider === provider.id" class="check-mark">✓</span>
        </div>
      </div>
    </section>

    <section v-if="form.provider" class="section">
      <h2 class="section-title">API 配置</h2>
      <div class="form-group">
        <label>API Key</label>
        <input v-model="form.apiKey" type="password" placeholder="sk-..." class="input" />
      </div>
      <div class="form-group">
        <label>模型名称</label>
        <input v-model="form.model" type="text" :placeholder="currentProvider?.defaultModel" class="input" />
      </div>
      <div class="form-group">
        <label>API Base URL（可选，留空使用官方默认）</label>
        <input v-model="form.baseUrl" type="text" placeholder="https://api.example.com" class="input" />
      </div>
      <div class="action-row">
        <button class="btn btn-primary" :disabled="saving" @click="saveConfig">
          {{ saving ? "保存中..." : "💾 保存配置" }}
        </button>
        <span v-if="saveMsg" class="save-msg" :class="saveMsgType">{{ saveMsg }}</span>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Provider {
  id: string;
  icon: string;
  name: string;
  desc: string;
  defaultModel: string;
}

interface ModelConfig {
  provider: string;
  apiKey: string;
  model: string;
  baseUrl: string;
}

const providers: Provider[] = [
  { id: "anthropic", icon: "🔴", name: "Anthropic (Claude)", desc: "Claude 3.5 / 4 系列",   defaultModel: "claude-sonnet-4-5" },
  { id: "openai",    icon: "🟢", name: "OpenAI",             desc: "GPT-4o / o1 系列",      defaultModel: "gpt-4o" },
  { id: "deepseek",  icon: "🔵", name: "DeepSeek",           desc: "DeepSeek-V3 / R1",      defaultModel: "deepseek-chat" },
  { id: "gemini",    icon: "🟡", name: "Google Gemini",      desc: "Gemini 2.0 系列",       defaultModel: "gemini-2.0-flash" },
  { id: "custom",    icon: "⚙️", name: "自定义",             desc: "任意兼容 OpenAI 的接口", defaultModel: "" },
];

const form = ref<ModelConfig>({ provider: "anthropic", apiKey: "", model: "", baseUrl: "" });
const saving = ref(false);
const saveMsg = ref("");
const saveMsgType = ref<"ok" | "err">("ok");

const currentProvider = computed(() => providers.find((p) => p.id === form.value.provider));

onMounted(async () => {
  try {
    const raw = await invoke<string | null>("read_config", { filename: "config.json" });
    if (raw) {
      const saved = JSON.parse(raw) as Partial<ModelConfig>;
      form.value = { ...form.value, ...saved };
    }
  } catch {
    // 配置不存在时忽略
  }
});

function selectProvider(provider: Provider) {
  form.value.provider = provider.id;
  if (!form.value.model) {
    form.value.model = provider.defaultModel;
  }
}

async function saveConfig() {
  saving.value = true;
  saveMsg.value = "";
  try {
    await invoke("write_config", {
      filename: "config.json",
      content: JSON.stringify(form.value, null, 2),
    });
    saveMsg.value = "✓ 已保存";
    saveMsgType.value = "ok";
  } catch (err) {
    saveMsg.value = `保存失败：${err}`;
    saveMsgType.value = "err";
  } finally {
    saving.value = false;
    setTimeout(() => (saveMsg.value = ""), 3000);
  }
}
</script>

<style scoped>
.page { max-width: 600px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px; }

.model-grid { display: flex; flex-direction: column; gap: 8px; }
.model-card { display: flex; align-items: center; gap: 12px; padding: 12px 14px; border-radius: 8px; background: var(--color-surface); border: 1px solid var(--color-border); cursor: pointer; transition: all 0.15s; }
.model-card:hover { border-color: var(--color-primary); }
.model-card.active { border-color: var(--color-primary); background: rgba(108, 99, 255, 0.08); }
.model-icon { font-size: 20px; }
.model-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.model-name { font-size: 13px; font-weight: 500; }
.model-desc { font-size: 11px; color: var(--color-text-muted); }
.check-mark { color: var(--color-primary); font-weight: 700; }

.form-group { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
.form-group label { font-size: 12px; color: var(--color-text-muted); }
.input { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--color-primary); }

.action-row { display: flex; align-items: center; gap: 12px; }
.save-msg { font-size: 12px; }
.save-msg.ok { color: var(--color-success); }
.save-msg.err { color: var(--color-danger); }
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
