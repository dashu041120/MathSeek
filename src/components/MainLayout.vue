<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Header -->
    <header class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center space-x-4">
            <h1 class="text-xl font-bold text-gray-900">MathSeek</h1>
            <span class="text-sm text-gray-500">数学公式识别工具</span>
          </div>
          
          <nav class="flex items-center space-x-4">
            <BaseButton variant="outline" size="sm" @click="$emit('openSettings')">
              设置
            </BaseButton>
          </nav>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <slot />
    </main>

    <!-- Footer -->
    <footer class="bg-white border-t border-gray-200 mt-auto">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
        <p class="text-center text-sm text-gray-500">
          MathSeek - 基于 Tauri 2 + Vue.js 3 构建
        </p>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import BaseButton from '@/components/BaseButton.vue'

defineEmits<{
  openSettings: []
}>()
</script>