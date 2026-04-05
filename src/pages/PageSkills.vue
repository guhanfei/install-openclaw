<template>
  <div class="page">
    <h1 class="page-title">Skill 管理</h1>
    <p class="page-desc">
      Skills 是 OpenClaw 插件目录下的 Markdown 文件，路径为
      <code>~/.openclaw/extensions/&lt;plugin&gt;/skills/&lt;skill&gt;/SKILL.md</code>。
    </p>

    <section class="section">
      <div class="toolbar">
        <h2 class="section-title">全部 Skills（按插件分组）</h2>
        <button class="btn btn-secondary btn-sm" @click="loadSkills">🔄 刷新</button>
      </div>

      <div v-if="loading" class="empty-state">读取中...</div>
      <div v-else-if="skills.length === 0" class="empty-state">
        未找到任何 skill。请先在「插件管理」中安装至少一个插件。
      </div>
      <div v-else>
        <!-- 按 plugin 分组展示 -->
        <div v-for="(group, pluginName) in grouped" :key="pluginName" class="plugin-group">
          <div class="plugin-name">🔌 {{ pluginName }}</div>
          <div class="skill-list">
            <div
              v-for="skill in group"
              :key="skill.path"
              class="skill-card"
              :class="{ active: selectedSkill?.path === skill.path }"
              @click="selectSkill(skill)"
            >
              <div class="skill-info">
                <span class="skill-name">{{ skill.name }}</span>
                <span class="skill-desc">{{ skill.description }}</span>
              </div>
              <span class="view-arrow">›</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 右侧详情抽屉 -->
    <div v-if="selectedSkill" class="detail-panel">
      <div class="detail-header">
        <div>
          <div class="detail-name">{{ selectedSkill.name }}</div>
          <div class="detail-plugin">{{ selectedSkill.plugin }}</div>
        </div>
        <button class="close-btn" @click="selectedSkill = null">✕</button>
      </div>
      <div class="detail-path">{{ selectedSkill.path }}</div>
      <div v-if="loadingContent" class="detail-content-loading">加载中...</div>
      <pre v-else class="detail-content">{{ skillContent }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface SkillInfo {
  name: string;
  description: string;
  plugin: string;
  path: string;
}

const skills = ref<SkillInfo[]>([]);
const loading = ref(false);
const selectedSkill = ref<SkillInfo | null>(null);
const skillContent = ref("");
const loadingContent = ref(false);

const grouped = computed(() => {
  const map: Record<string, SkillInfo[]> = {};
  for (const s of skills.value) {
    if (!map[s.plugin]) map[s.plugin] = [];
    map[s.plugin].push(s);
  }
  return map;
});

onMounted(loadSkills);

async function loadSkills() {
  loading.value = true;
  selectedSkill.value = null;
  try {
    skills.value = await invoke<SkillInfo[]>("list_skills");
  } finally {
    loading.value = false;
  }
}

async function selectSkill(skill: SkillInfo) {
  selectedSkill.value = skill;
  loadingContent.value = true;
  skillContent.value = "";
  try {
    skillContent.value = await invoke<string>("read_skill", { path: skill.path });
  } catch (err) {
    skillContent.value = `读取失败：${err}`;
  } finally {
    loadingContent.value = false;
  }
}
</script>

<style scoped>
.page { max-width: 680px; position: relative; }
.page-title { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.page-desc { color: var(--color-text-muted); margin-bottom: 28px; font-size: 13px; line-height: 1.6; }
.page-desc code { background: var(--color-border); padding: 1px 5px; border-radius: 3px; font-size: 11px; word-break: break-all; }
.section { margin-bottom: 32px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.empty-state { padding: 32px; text-align: center; color: var(--color-text-muted); font-size: 13px; background: var(--color-surface); border: 1px dashed var(--color-border); border-radius: 8px; }

.plugin-group { margin-bottom: 20px; }
.plugin-name { font-size: 12px; font-weight: 600; color: var(--color-primary); margin-bottom: 6px; padding-left: 2px; }
.skill-list { display: flex; flex-direction: column; gap: 4px; }
.skill-card { display: flex; align-items: center; padding: 10px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; cursor: pointer; transition: all 0.15s; }
.skill-card:hover { border-color: var(--color-primary); }
.skill-card.active { border-color: var(--color-primary); background: rgba(108, 99, 255, 0.06); }
.skill-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.skill-name { font-size: 13px; font-weight: 500; }
.skill-desc { font-size: 11px; color: var(--color-text-muted); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.view-arrow { color: var(--color-text-muted); font-size: 18px; }

/* 右侧详情面板 */
.detail-panel {
  position: fixed;
  top: 0; right: 0; bottom: 0;
  width: 480px;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  display: flex; flex-direction: column;
  z-index: 50;
  box-shadow: -8px 0 24px rgba(0,0,0,0.3);
}
.detail-header { display: flex; justify-content: space-between; align-items: flex-start; padding: 20px; border-bottom: 1px solid var(--color-border); }
.detail-name { font-size: 15px; font-weight: 600; margin-bottom: 3px; }
.detail-plugin { font-size: 11px; color: var(--color-primary); }
.close-btn { background: none; border: none; font-size: 18px; cursor: pointer; color: var(--color-text-muted); padding: 0 4px; }
.close-btn:hover { color: var(--color-text); }
.detail-path { font-size: 10px; color: var(--color-text-muted); font-family: monospace; padding: 6px 20px; border-bottom: 1px solid var(--color-border); word-break: break-all; }
.detail-content-loading { padding: 20px; color: var(--color-text-muted); font-size: 13px; }
.detail-content { flex: 1; overflow-y: auto; padding: 16px 20px; font-size: 12px; font-family: monospace; line-height: 1.7; white-space: pre-wrap; word-break: break-all; color: var(--color-text); margin: 0; }

.btn { padding: 8px 18px; border-radius: 6px; border: none; font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.btn-secondary { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.btn-secondary:hover { border-color: var(--color-primary); color: var(--color-primary); }
</style>
