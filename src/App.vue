<script setup lang="ts">
import { ref, onMounted } from "vue";
import MainLayout from "@/components/MainLayout.vue";
import MainWorkflow from "@/components/MainWorkflow.vue";
import BaseCard from "@/components/BaseCard.vue";
import BaseButton from "@/components/BaseButton.vue";
import { useTauri } from "@/composables/useTauri";
import type { SystemStatus } from "@/types";

const showSettings = ref(false);
const showWorkflow = ref(false);
const systemStatus = ref<SystemStatus>({
  clipboard_available: false,
  screenshot_available: false,
  api_configured: false,
  render_engine_ready: false
});
const appVersion = ref('0.1.0');

const { getAppVersion, checkSystemStatus } = useTauri();

function openSettings() {
  showSettings.value = true;
}

function startWorkflow() {
  showWorkflow.value = true;
}

async function loadSystemInfo() {
  try {
    appVersion.value = await getAppVersion();
    systemStatus.value = await checkSystemStatus();
  } catch (error) {
    console.error('Failed to load system info:', error);
  }
}

onMounted(() => {
  loadSystemInfo();
});
</script>

<template>
  <MainLayout @open-settings="openSettings">
    <!-- Main Workflow -->
    <MainWorkflow v-if="showWorkflow" />
    
    <!-- Welcome Screen -->
    <div v-else class="space-y-6">
      <!-- Welcome Section -->
      <BaseCard>
        <template #header>
          <h2 class="text-2xl font-bold text-gray-900">欢迎使用 MathSeek</h2>
        </template>
        
        <div class="text-center space-y-4">
          <p class="text-gray-600">
            MathSeek 是一个强大的数学公式识别工具，支持截图识别、公式编辑和多格式导出。
          </p>
          
          <div class="flex justify-center space-x-4">
            <BaseButton @click="startWorkflow">开始识别</BaseButton>
            <BaseButton variant="outline">查看帮助</BaseButton>
          </div>
        </div>
      </BaseCard>

      <!-- Quick Actions -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <BaseCard hoverable>
          <div class="text-center space-y-3">
            <div class="w-12 h-12 bg-primary-100 rounded-lg flex items-center justify-center mx-auto">
              <svg class="w-6 h-6 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </div>
            <h3 class="font-semibold text-gray-900">截图识别</h3>
            <p class="text-sm text-gray-600">快速截取屏幕上的数学公式进行识别</p>
          </div>
        </BaseCard>

        <BaseCard hoverable>
          <div class="text-center space-y-3">
            <div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mx-auto">
              <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <h3 class="font-semibold text-gray-900">粘贴识别</h3>
            <p class="text-sm text-gray-600">从剪贴板粘贴图像进行公式识别</p>
          </div>
        </BaseCard>

        <BaseCard hoverable>
          <div class="text-center space-y-3">
            <div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mx-auto">
              <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
              </svg>
            </div>
            <h3 class="font-semibold text-gray-900">AI 分析</h3>
            <p class="text-sm text-gray-600">智能分析公式类型和数学含义</p>
          </div>
        </BaseCard>
      </div>

      <!-- Status Section -->
      <BaseCard>
        <template #header>
          <h3 class="text-lg font-semibold text-gray-900">系统状态</h3>
        </template>
        
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-gray-600">应用版本</span>
            <span class="px-2 py-1 bg-blue-100 text-blue-800 rounded-full text-sm">v{{ appVersion }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-600">API 连接状态</span>
            <span :class="[
              'px-2 py-1 rounded-full text-sm',
              systemStatus.api_configured 
                ? 'bg-green-100 text-green-800' 
                : 'bg-yellow-100 text-yellow-800'
            ]">
              {{ systemStatus.api_configured ? '已配置' : '未配置' }}
            </span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-600">剪贴板支持</span>
            <span :class="[
              'px-2 py-1 rounded-full text-sm',
              systemStatus.clipboard_available 
                ? 'bg-green-100 text-green-800' 
                : 'bg-red-100 text-red-800'
            ]">
              {{ systemStatus.clipboard_available ? '可用' : '不可用' }}
            </span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-600">公式渲染引擎</span>
            <span :class="[
              'px-2 py-1 rounded-full text-sm',
              systemStatus.render_engine_ready 
                ? 'bg-green-100 text-green-800' 
                : 'bg-red-100 text-red-800'
            ]">
              {{ systemStatus.render_engine_ready ? '就绪' : '未就绪' }}
            </span>
          </div>
        </div>
      </BaseCard>
    </div>
  </MainLayout>
</template>

