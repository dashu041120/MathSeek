<template>
  <div class="formula-renderer">
    <!-- Renderer Controls -->
    <div v-if="showControls" class="renderer-controls">
      <div class="control-group">
        <label class="control-label">渲染引擎:</label>
        <select
          v-model="selectedEngine"
          @change="handleEngineChange"
          class="engine-select"
        >
          <option value="mathjax">MathJax</option>
          <option value="katex">KaTeX</option>
        </select>
      </div>
      
      <div class="control-group">
        <button
          @click="zoomIn"
          :disabled="zoomLevel >= 300"
          class="control-btn"
          title="放大"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
          </svg>
        </button>
        
        <span class="zoom-display">{{ zoomLevel }}%</span>
        
        <button
          @click="zoomOut"
          :disabled="zoomLevel <= 50"
          class="control-btn"
          title="缩小"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"/>
          </svg>
        </button>
      </div>
      
      <div class="control-group">
        <button
          @click="copyRenderedFormula"
          class="control-btn"
          title="复制公式"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
          </svg>
        </button>
        
        <button
          @click="refreshRender"
          class="control-btn"
          title="刷新渲染"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Render Container -->
    <div 
      class="render-container"
      :class="{
        'has-error': renderError,
        'is-loading': isLoading,
        'inline-mode': displayMode === 'inline'
      }"
      :style="{ fontSize: zoomLevel + '%' }"
    >
      <!-- Loading State -->
      <div v-if="isLoading" class="loading-state">
        <div class="loading-spinner"></div>
        <span class="loading-text">正在渲染公式...</span>
      </div>

      <!-- Error State -->
      <div v-else-if="renderError" class="error-state">
        <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        <div class="error-content">
          <div class="error-title">渲染失败</div>
          <div class="error-message">{{ renderError }}</div>
          <button @click="refreshRender" class="retry-btn">重试</button>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="!latex.trim()" class="empty-state">
        <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <span class="empty-text">{{ emptyMessage }}</span>
      </div>

      <!-- Rendered Formula -->
      <div
        v-else
        ref="formulaContainer"
        class="formula-container"
        :class="{ 'center-align': centerAlign }"
        v-html="renderedContent"
      />
    </div>

    <!-- Render Info -->
    <div v-if="showInfo && !renderError && !isLoading" class="render-info">
      <span class="info-item">引擎: {{ selectedEngine.toUpperCase() }}</span>
      <span class="info-item">模式: {{ displayMode === 'inline' ? '行内' : '块级' }}</span>
      <span class="info-item">缩放: {{ zoomLevel }}%</span>
      <span v-if="renderTime" class="info-item">渲染时间: {{ renderTime }}ms</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import mathRenderer, { type RenderEngine } from '@/services/mathRenderer'

interface Props {
  latex: string
  displayMode?: 'inline' | 'block'
  engine?: 'mathjax' | 'katex'
  showControls?: boolean
  showInfo?: boolean
  centerAlign?: boolean
  emptyMessage?: string
  autoRender?: boolean
}

interface Emits {
  (e: 'render-success', content: string): void
  (e: 'render-error', error: string): void
  (e: 'engine-change', engine: string): void
}

const props = withDefaults(defineProps<Props>(), {
  displayMode: 'block',
  engine: 'mathjax',
  showControls: true,
  showInfo: false,
  centerAlign: true,
  emptyMessage: '暂无公式内容',
  autoRender: true
})

const emit = defineEmits<Emits>()

// Refs
const formulaContainer = ref<HTMLDivElement>()

// State
const selectedEngine = ref(props.engine as RenderEngine)
const zoomLevel = ref(100)
const isLoading = ref(false)
const renderError = ref<string | null>(null)
const renderedContent = ref('')
const renderTime = ref<number | null>(null)

// Computed
const processedLatex = computed(() => {
  return props.latex.trim()
})

// Watch for changes
watch(() => props.latex, () => {
  if (props.autoRender) {
    renderFormula()
  }
})

watch(() => props.displayMode, () => {
  if (props.autoRender) {
    renderFormula()
  }
})

watch(selectedEngine, (newEngine) => {
  emit('engine-change', newEngine)
  if (props.autoRender) {
    renderFormula()
  }
})

// Methods
async function renderFormula() {
  if (!props.latex.trim()) {
    renderedContent.value = ''
    renderError.value = null
    return
  }

  isLoading.value = true
  renderError.value = null

  try {
    const result = await mathRenderer.render(
      processedLatex.value,
      selectedEngine.value,
      {
        displayMode: props.displayMode === 'block',
        throwOnError: false
      }
    )

    if (result.success && result.html) {
      renderedContent.value = result.html
      renderTime.value = result.renderTime || null
      emit('render-success', result.html)
    } else {
      throw new Error(result.error || '渲染失败')
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : '未知渲染错误'
    renderError.value = errorMessage
    emit('render-error', errorMessage)
  } finally {
    isLoading.value = false
  }
}

function handleEngineChange() {
  if (props.autoRender) {
    renderFormula()
  }
}

function zoomIn() {
  if (zoomLevel.value < 300) {
    zoomLevel.value += 25
  }
}

function zoomOut() {
  if (zoomLevel.value > 50) {
    zoomLevel.value -= 25
  }
}

async function copyRenderedFormula() {
  try {
    await navigator.clipboard.writeText(props.latex)
    // Could show a toast notification here
  } catch (err) {
    console.error('Failed to copy formula:', err)
  }
}

function refreshRender() {
  renderFormula()
}

// Public methods (exposed via defineExpose)
function render() {
  return renderFormula()
}

function setZoom(level: number) {
  zoomLevel.value = Math.max(50, Math.min(300, level))
}

function getRenderedContent() {
  return renderedContent.value
}

// Lifecycle
onMounted(() => {
  if (props.autoRender && props.latex.trim()) {
    renderFormula()
  }
})

// Expose methods for parent components
defineExpose({
  render,
  setZoom,
  getRenderedContent,
  refreshRender
})
</script>

<style scoped>
.formula-renderer {
  @apply bg-white rounded-lg border border-gray-200;
}

.renderer-controls {
  @apply flex items-center justify-between p-3 border-b border-gray-200 bg-gray-50;
  @apply flex-wrap gap-2;
}

.control-group {
  @apply flex items-center gap-2;
}

.control-label {
  @apply text-sm font-medium text-gray-700;
}

.engine-select {
  @apply text-sm border border-gray-300 rounded px-2 py-1 focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
}

.control-btn {
  @apply p-1.5 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed;
  @apply focus:outline-none focus:ring-2 focus:ring-blue-500;
}

.zoom-display {
  @apply text-sm font-mono text-gray-600 min-w-[3rem] text-center;
}

.render-container {
  @apply min-h-24 p-4 relative;
}

.render-container.inline-mode {
  @apply p-2;
}

.render-container.has-error {
  @apply bg-red-50;
}

.render-container.is-loading {
  @apply bg-blue-50;
}

.loading-state {
  @apply flex items-center justify-center gap-3 text-blue-600;
}

.loading-spinner {
  @apply w-5 h-5 border-2 border-blue-600 border-t-transparent rounded-full animate-spin;
}

.loading-text {
  @apply text-sm;
}

.error-state {
  @apply flex items-start gap-3 text-red-600;
}

.error-icon {
  @apply w-5 h-5 flex-shrink-0 mt-0.5;
}

.error-content {
  @apply flex-1;
}

.error-title {
  @apply font-medium;
}

.error-message {
  @apply text-sm mt-1;
}

.retry-btn {
  @apply mt-2 text-sm bg-red-100 text-red-700 px-3 py-1 rounded hover:bg-red-200;
}

.empty-state {
  @apply flex flex-col items-center justify-center gap-2 text-gray-500 py-8;
}

.empty-icon {
  @apply w-8 h-8;
}

.empty-text {
  @apply text-sm;
}

.formula-container {
  @apply min-h-12 flex items-center;
}

.formula-container.center-align {
  @apply justify-center;
}

.render-info {
  @apply flex items-center gap-4 p-2 border-t border-gray-200 bg-gray-50 text-xs text-gray-600;
}

.info-item {
  @apply flex items-center gap-1;
}

/* Math formula styles */
:deep(.math-formula) {
  @apply font-serif text-lg leading-relaxed;
}

:deep(.fraction) {
  @apply inline-flex flex-col items-center mx-1;
}

:deep(.numerator) {
  @apply border-b border-current pb-0.5;
}

:deep(.denominator) {
  @apply pt-0.5;
}

:deep(.sqrt-content) {
  @apply border-t border-current;
}

:deep(sup) {
  @apply text-sm;
  vertical-align: super;
}

:deep(sub) {
  @apply text-sm;
  vertical-align: sub;
}
</style>