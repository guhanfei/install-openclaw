<template>
  <div class="app-layout">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <span class="logo-icon">🦞</span>
        <span class="logo-text">OpenClaw 助手</span>
      </div>
      <nav class="sidebar-nav">
        <button
          v-for="item in navItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: currentPage === item.id }"
          @click="currentPage = item.id"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </button>
      </nav>
      <div class="sidebar-footer">
        <div class="version-badge">v{{ appVersion }}</div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="main-content">
      <component :is="currentComponent" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import PageInstall from "./pages/PageInstall.vue";
import PageModels from "./pages/PageModels.vue";
import PageSkills from "./pages/PageSkills.vue";
import PagePlugins from "./pages/PagePlugins.vue";
import PageService from "./pages/PageService.vue";
import PageUpdate from "./pages/PageUpdate.vue";
import PageUninstall from "./pages/PageUninstall.vue";

const appVersion = "0.1.0";

const navItems = [
  { id: "install", icon: "📦", label: "安装 OpenClaw" },
    { id: "service", icon: "▶", label: "启动/停止 OpenClaw" },
    { id: "update", icon: "▶", label: "升级 OpenClaw" },
  { id: "models", icon: "🤖", label: "模型厂商" },
  { id: "skills", icon: "⚡", label: "Skill 管理" },
  { id: "plugins", icon: "🔌", label: "插件管理" },
  { id: "uninstall", icon: "🗑️", label: "卸载 OpenClaw" },
];

const currentPage = ref("install");

const currentComponent = computed(() => {
  const map: Record<string, unknown> = {
    install: PageInstall,
    models: PageModels,
    skills: PageSkills,
    plugins: PagePlugins,
    service: PageService,
    update: PageUpdate,
    uninstall: PageUninstall,
  };
  return map[currentPage.value];
});
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: var(--sidebar-width);
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 20px 16px 16px;
  border-bottom: 1px solid var(--color-border);
  font-weight: 600;
  font-size: 13px;
}

.logo-icon {
  font-size: 20px;
}

.sidebar-nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 13px;
  text-align: left;
  transition: all 0.15s;
}

.nav-item:hover {
  background: rgba(108, 99, 255, 0.1);
  color: var(--color-text);
}

.nav-item.active {
  background: rgba(108, 99, 255, 0.2);
  color: var(--color-primary);
  font-weight: 500;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border);
}

.version-badge {
  font-size: 11px;
  color: var(--color-text-muted);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px;
}
</style>
