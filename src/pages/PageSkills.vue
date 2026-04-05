<template>
  <div class="page">
    <h1 class="page-title">Skill 管理</h1>
    <p class="page-desc">增删改查 OpenClaw 的 skill 配置（~/.openclaw/skills.json）。</p>

    <section class="section">
      <div class="toolbar">
        <h2 class="section-title">已有 Skill</h2>
        <button class="btn btn-primary btn-sm" @click="openAdd">+ 新增</button>
      </div>
      <div v-if="loading" class="empty-state">加载中...</div>
      <div v-else-if="skills.length === 0" class="empty-state">暂无 skill，点击「新增」添加</div>
      <div v-else class="list">
        <div v-for="skill in skills" :key="skill.name" class="list-item">
          <div class="item-info">
            <span class="item-name">{{ skill.name }}</span>
            <span class="item-desc">{{ skill.description }}</span>
          </div>
          <div class="item-actions">
            <button class="btn-icon" title="编辑" @click="openEdit(skill)">✏️</button>
            <button class="btn-icon danger" title="删除" @click="deleteSkill(skill.name)">🗑️</button>
          </div>
        </div>
      </div>
      <p v-if="errorMsg" class="error-hint">{{ errorMsg }}</p>
    </section>

    <!-- 新增/编辑弹窗 -->
    <div v-if="showDialog" class="dialog-mask" @click.self="showDialog = false">
      <div class="dialog">
        <h3>{{ editingName ? "编辑 Skill" : "新增 Skill" }}</h3>
        <div class="form-group">
          <label>名称（唯一标识，不可修改）</label>
          <input v-model="form.name" type="text" placeholder="my-skill" class="input" :disabled="!!editingName" />
        </div>
        <div class="form-group">
          <label>描述</label>
          <input v-model="form.description" type="text" placeholder="这个 skill 的用途" class="input" />
        </div>
        <div class="form-group">
          <label>内容（Markdown / Prompt）</label>
          <textarea v-model="form.content" class="input textarea" rows="7" placeholder="skill 内容..."></textarea>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-secondary" @click="showDialog = false">取消</button>
          <button class="btn btn-primary" :disabled="!form.name" @click="saveSkill">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Skill {
  name: string;
  description: string;
  content: string;
}

const skills = ref<Skill[]>([]);
const loading = ref(false);
const errorMsg = ref("");
const showDialog = ref(false);
const editingName = ref<string | null>(null);
const form = ref<Skill>({ name: "", description: "", content: "" });

onMounted(loadSkills);

async function loadSkills() {
  loading.value = true;
  errorMsg.value = "";
  try {
    const raw = await invoke<string | null>("read_config", { filename: "skills.json" });
    skills.value = raw ? (JSON.parse(raw) as Skill[]) : [];
  } catch (err) {
    errorMsg.value = `读取失败：${err}`;
  } finally {
    loading.value = false;
  }
}

async function persistSkills() {
  await invoke("write_config", {
    filename: "skills.json",
    content: JSON.stringify(skills.value, null, 2),
  });
}

function openAdd() {
  editingName.value = null;
  form.value = { name: "", description: "", content: "" };
  showDialog.value = true;
}

function openEdit(skill: Skill) {
  editingName.value = skill.name;
  form.value = { ...skill };
  showDialog.value = true;
}

async function saveSkill() {
  errorMsg.value = "";
  try {
    if (editingName.value) {
      const idx = skills.value.findIndex((s) => s.name === editingName.value);
      if (idx !== -1) skills.value[idx] = { ...form.value };
    } else {
      if (skills.value.some((s) => s.name === form.value.name)) {
        errorMsg.value = `名称 "${form.value.name}" 已存在`;
        return;
      }
      skills.value.push({ ...form.value });
    }
    await persistSkills();
    showDialog.value = false;
  } catch (err) {
    errorMsg.value = `保存失败：${err}`;
  }
}

async function deleteSkill(name: string) {
  skills.value = skills.value.filter((s) => s.name !== name);
  try {
    await persistSkills();
  } catch (err) {
    errorMsg.value = `删除失败：${err}`;
  }
}
</script>

<style scoped>
.page { max-width: 680px; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.error-hint { margin-top: 8px; font-size: 12px; color: var(--color-danger); }

.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.empty-state { padding: 32px; text-align: center; color: var(--color-text-muted); font-size: 13px; background: var(--color-surface); border: 1px dashed var(--color-border); border-radius: 8px; }
.list { display: flex; flex-direction: column; gap: 6px; }
.list-item { display: flex; align-items: center; padding: 10px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; }
.item-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.item-name { font-size: 13px; font-weight: 500; }
.item-desc { font-size: 11px; color: var(--color-text-muted); }
.item-actions { display: flex; gap: 6px; }
.btn-icon { background: none; border: none; cursor: pointer; padding: 4px; border-radius: 4px; font-size: 14px; opacity: 0.6; transition: opacity 0.15s; }
.btn-icon:hover { opacity: 1; }
.btn-icon.danger:hover { color: var(--color-danger); }

.dialog-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.dialog { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 12px; padding: 24px; width: 500px; max-width: 90vw; }
.dialog h3 { font-size: 16px; font-weight: 600; margin-bottom: 20px; }
.form-group { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
.form-group label { font-size: 12px; color: var(--color-text-muted); }
.input { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border); background: var(--color-bg); color: var(--color-text); font-size: 13px; outline: none; transition: border-color 0.15s; }
.input:focus { border-color: var(--color-primary); }
.textarea { resize: vertical; font-family: monospace; line-height: 1.6; }
.dialog-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 4px; }
.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-primary:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover { border-color: var(--color-primary); }
</style>
