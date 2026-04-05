<template>
  <div class="app-layout">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <span class="logo-icon">🦞</span>
        <span class="logo-text">OpenClaw 助手</span>
      </div>
      <nav class="sidebar-nav">
        <button v-for="item in navItems" :key="item.id" class="nav-item" :class="{ active: currentPage === item.id }"
          @click="handleNavClick(item.id, '')">
          <component :is="item.icon" class="nav-icon" :size="16" :stroke-width="1.6" />
          <span>{{ item.label }}</span>
        </button>
      </nav>
      <div class="sidebar-bottom">
        <button v-for="item in bottomNavItems" :key="item.id" class="nav-item"
          :class="{ active: !item.external && currentPage === item.id }"
          @click="handleNavClick(item.id, item.external)">
          <component :is="item.icon" class="nav-icon" :size="16" :stroke-width="1.6" />
          <span>{{ item.label }}</span>
          <span v-if="item.showVersion" class="nav-version-badge">v{{ appVersion }}</span>
          <span v-if="item.external" class="ext-icon">↗</span>
        </button>
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
import {
  Monitor, PackagePlus, Power, ArrowUpCircle,
  BrainCircuit, Trash2,
  BookOpen, Info,
} from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-shell";
import PageCheckSystem from "./pages/PageCheckSystem.vue";
import PageInstall from "./pages/PageInstall.vue";
import PageModels from "./pages/PageModels.vue";
import PageSkills from "./pages/PageSkills.vue";
import PagePlugins from "./pages/PagePlugins.vue";
import PageService from "./pages/PageService.vue";
import PageUpdate from "./pages/PageUpdate.vue";
import PageUninstall from "./pages/PageUninstall.vue";
import PageAbout from "./pages/PageAbout.vue";

const appVersion = __APP_VERSION__;

const navItems = [
  { id: "check",     icon: Monitor,        label: "检测本机电脑" },
  { id: "install",   icon: PackagePlus,    label: "安装 OpenClaw" },
  { id: "service",   icon: Power,          label: "启动/停止 OpenClaw" },
  { id: "update",    icon: ArrowUpCircle,  label: "升级 OpenClaw" },
  { id: "models",    icon: BrainCircuit,   label: "模型厂商" },
  // { id: "skills",    icon: Wand2,          label: "Skill 管理" },   // TODO: 待完善
  // { id: "plugins",   icon: Puzzle,         label: "插件管理" },     // TODO: 待完善
  { id: "uninstall", icon: Trash2,         label: "卸载 OpenClaw" },
];

const bottomNavItems = [
  { id: "docs",  icon: BookOpen, label: "官网文档", external: "https://docs.openclaw.ai", showVersion: false },
  { id: "about", icon: Info,     label: "关于",     external: "",                          showVersion: true  },
];

const currentPage = ref("check");

function handleNavClick(id: string, external: string) {
  if (external) {
    open(external);
  } else {
    currentPage.value = id;
  }
}

const currentComponent = computed(() => {
  const map: Record<string, unknown> = {
    check: PageCheckSystem,
    install: PageInstall,
    models: PageModels,
    skills: PageSkills,
    plugins: PagePlugins,
    service: PageService,
    update: PageUpdate,
    uninstall: PageUninstall,
    about: PageAbout,
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
  width: 20px;
  flex-shrink: 0;
}

.sidebar-bottom {
  padding: 8px;
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ext-icon {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-left: auto;
}

.nav-version-badge {
  margin-left: auto;
  font-size: 10px;
  font-weight: 500;
  color: var(--color-primary);
  background: rgba(108, 99, 255, 0.12);
  border: 1px solid rgba(108, 99, 255, 0.2);
  border-radius: 4px;
  padding: 1px 6px;
  letter-spacing: 0.02em;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px;
}
</style>
