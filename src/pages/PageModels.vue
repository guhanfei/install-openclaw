<template>
  <div class="page">
    <h1 class="page-title">模型厂商</h1>
    <p class="page-desc">管理 <code>~/.openclaw/openclaw.json</code> 中的 <code>models.providers</code> 配置。</p>

    <!-- 当前主模型 -->
    <section class="section">
      <h2 class="section-title">当前主模型</h2>
      <div class="primary-row">
        <select v-model="primaryModel" class="select">
          <option disabled value="">-- 请选择 --</option>
          <option v-for="opt in allModelOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
        <button class="btn btn-primary" :disabled="saving" @click="save">
          {{ saving ? "保存中..." : "💾 保存" }}
        </button>
        <span v-if="saveMsg" class="save-msg" :class="saveMsgType">{{ saveMsg }}</span>
      </div>
    </section>

    <!-- Provider 列表 -->
    <section class="section">
      <div class="toolbar">
        <h2 class="section-title">已配置的 Provider</h2>
        <button class="btn btn-primary btn-sm" @click="openAdd">+ 添加 Provider</button>
      </div>

      <div v-if="loading" class="empty-state">读取配置中...</div>
      <div v-else-if="Object.keys(providers).length === 0" class="empty-state">
        暂无 provider，点击「添加」开始配置
      </div>
      <div v-else class="provider-list">
        <div v-for="(p, id) in providers" :key="id" class="provider-card">
          <div class="provider-header">
            <div class="provider-meta">
              <span class="provider-id">{{ id }}</span>
              <span class="provider-api-badge">{{ p.api }}</span>
            </div>
            <div class="provider-actions">
              <button class="btn-icon" title="编辑" @click="openEdit(id, p)">✏️</button>
              <button class="btn-icon danger" title="删除" @click="deleteProvider(id)">🗑️</button>
            </div>
          </div>
          <div class="provider-url">{{ p.baseUrl || '（使用官方默认地址）' }}</div>
          <div class="provider-models">
            <span v-for="m in p.models" :key="m.id" class="model-tag">{{ m.id }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- 添加/编辑 Provider 弹窗 -->
    <div v-if="showDialog" class="dialog-mask" @click.self="showDialog = false">
      <div class="dialog">
        <h3>{{ editingId ? `编辑 Provider：${editingId}` : '添加 Provider' }}</h3>

        <div class="form-group">
          <label>Provider ID（唯一标识，字母/数字/横杠）</label>
          <input v-model="form.id" type="text" placeholder="my-provider" class="input" :disabled="!!editingId" />
        </div>
        <div class="form-group">
          <label>API 类型</label>
          <select v-model="form.api" class="select">
            <option value="openai-completions">openai-completions（OpenAI 兼容接口）</option>
            <option value="anthropic-messages">anthropic-messages（Anthropic 原生接口）</option>
          </select>
        </div>
        <div class="form-group">
          <label>API Key</label>
          <input v-model="form.apiKey" type="password" placeholder="sk-..." class="input" />
        </div>
        <div class="form-group">
          <label>Base URL（留空使用官方默认）</label>
          <input v-model="form.baseUrl" type="text" placeholder="https://api.example.com/v1" class="input" />
        </div>
        <div class="form-group">
          <label>模型列表（每行一个 Model ID）</label>
          <textarea v-model="form.modelsText" class="input textarea" rows="4"
            placeholder="gpt-4o&#10;gpt-4o-mini&#10;o1"></textarea>
        </div>

        <div class="dialog-actions">
          <button class="btn btn-secondary" @click="showDialog = false">取消</button>
          <button class="btn btn-primary" :disabled="!form.id || !form.apiKey" @click="saveProvider">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ProviderModel { id: string; name: string; [key: string]: unknown }
interface Provider {
  api: string;
  apiKey: string;
  baseUrl: string;
  models: ProviderModel[];
}
// openclaw.json 中我们关心的两个路径，其余字段原样保留
interface OpencLawConfig {
  models?: { mode?: string; providers?: Record<string, Provider> };
  agents?: { defaults?: { model?: { primary?: string }; [key: string]: unknown }; [key: string]: unknown };
  [key: string]: unknown;
}

const loading = ref(false);
const saving = ref(false);
const saveMsg = ref("");
const saveMsgType = ref<"ok" | "err">("ok");
const showDialog = ref(false);
const editingId = ref<string | null>(null);

// 原始 JSON 对象，写回时保留所有字段
const rawConfig = ref<OpencLawConfig>({});
const providers = computed<Record<string, Provider>>(() => rawConfig.value.models?.providers ?? {});
const primaryModel = ref("");

const form = ref({ id: "", api: "openai-completions", apiKey: "", baseUrl: "", modelsText: "" });

const allModelOptions = computed(() => {
  const opts: { value: string; label: string }[] = [];
  for (const [pid, p] of Object.entries(providers.value)) {
    for (const m of p.models) {
      const val = `${pid}/${m.id}`;
      opts.push({ value: val, label: val });
    }
  }
  return opts;
});

onMounted(async () => {
  loading.value = true;
  try {
    const raw = await invoke<string | null>("read_config", { filename: "openclaw.json" });
    if (raw) {
      rawConfig.value = JSON.parse(raw) as OpencLawConfig;
      primaryModel.value = rawConfig.value.agents?.defaults?.model?.primary ?? "";
    }
  } catch (err) {
    saveMsg.value = `读取失败：${err}`;
    saveMsgType.value = "err";
  } finally {
    loading.value = false;
  }
});

async function save() {
  saving.value = true;
  saveMsg.value = "";
  try {
    // 深度设置 primary model
    if (!rawConfig.value.agents) rawConfig.value.agents = {};
    if (!rawConfig.value.agents.defaults) rawConfig.value.agents.defaults = {};
    if (!rawConfig.value.agents.defaults.model) rawConfig.value.agents.defaults.model = {};
    rawConfig.value.agents.defaults.model.primary = primaryModel.value;

    await invoke("write_config", {
      filename: "openclaw.json",
      content: JSON.stringify(rawConfig.value, null, 2),
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

function openAdd() {
  editingId.value = null;
  form.value = { id: "", api: "openai-completions", apiKey: "", baseUrl: "", modelsText: "" };
  showDialog.value = true;
}

function openEdit(id: string, p: Provider) {
  editingId.value = id;
  form.value = {
    id,
    api: p.api,
    apiKey: p.apiKey,
    baseUrl: p.baseUrl ?? "",
    modelsText: p.models.map((m) => m.id).join("\n"),
  };
  showDialog.value = true;
}

function deleteProvider(id: string) {
  if (!rawConfig.value.models?.providers) return;
  delete rawConfig.value.models.providers[id];
  // 触发响应式更新
  rawConfig.value = { ...rawConfig.value };
  save();
}

async function saveProvider() {
  const targetId = editingId.value ?? form.value.id;
  if (!targetId) return;

  const models: ProviderModel[] = form.value.modelsText
    .split("\n")
    .map((s) => s.trim())
    .filter(Boolean)
    .map((id) => ({ id, name: id }));

  const provider: Provider = {
    api: form.value.api,
    apiKey: form.value.apiKey,
    baseUrl: form.value.baseUrl,
    models,
  };

  if (!rawConfig.value.models) rawConfig.value.models = {};
  if (!rawConfig.value.models.providers) rawConfig.value.models.providers = {};
  rawConfig.value.models.providers[targetId] = provider;
  rawConfig.value = { ...rawConfig.value };

  showDialog.value = false;
  await save();
}
</script>

<style scoped>
.page { max-width: 700px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.page-desc code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }

.primary-row { display: flex; align-items: center; gap: 10px; }
.primary-row .select { flex: 1; }
.save-msg { font-size: 12px; }
.save-msg.ok { color: var(--color-success); }
.save-msg.err { color: var(--color-danger); }

.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.empty-state { padding: 32px; text-align: center; color: var(--color-text-muted); font-size: 13px; background: var(--color-surface); border: 1px dashed var(--color-border); border-radius: 8px; }

.provider-list { display: flex; flex-direction: column; gap: 8px; }
.provider-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; padding: 12px 14px; }
.provider-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.provider-meta { display: flex; align-items: center; gap: 8px; }
.provider-id { font-size: 14px; font-weight: 600; }
.provider-api-badge { font-size: 10px; background: rgba(108, 99, 255, 0.15); color: var(--color-primary); padding: 2px 7px; border-radius: 4px; }
.provider-url { font-size: 11px; color: var(--color-text-muted); margin-bottom: 8px; font-family: monospace; }
.provider-models { display: flex; flex-wrap: wrap; gap: 4px; }
.model-tag { font-size: 11px; background: var(--color-border); padding: 2px 8px; border-radius: 4px; color: var(--color-text-muted); font-family: monospace; }
.provider-actions { display: flex; gap: 4px; }
.btn-icon { background: none; border: none; cursor: pointer; padding: 4px; border-radius: 4px; font-size: 14px; opacity: 0.6; transition: opacity 0.15s; }
.btn-icon:hover { opacity: 1; }
.btn-icon.danger:hover { color: var(--color-danger); }

.dialog-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.dialog { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 12px; padding: 24px; width: 520px; max-width: 90vw; }
.dialog h3 { font-size: 16px; font-weight: 600; margin-bottom: 20px; }
.form-group { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
.form-group label { font-size: 12px; color: var(--color-text-muted); }
.input, .select { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-bg); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus, .select:focus { border-color: var(--color-primary); }
.textarea { resize: vertical; font-family: monospace; line-height: 1.6; }
.select { appearance: none; cursor: pointer; }
.dialog-actions { display: flex; gap: 10px; justify-content: flex-end; }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover { border-color: var(--color-primary); }
</style>
