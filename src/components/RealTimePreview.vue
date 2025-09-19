<template>
  <div class="realtime-preview">
    <!-- Preview Header -->
    <div class="preview-header">
      <div class="header-left">
        <h3 class="preview-title">实时预览</h3>
        <div class="sync-indicator" :class="{ 'syncing': isSyncing, 'error': hasError }">
          <svg v-if="isSyncing" class="sync-icon animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          <svg v-else-if="hasError" class="sync-icon text-red-500" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          <svg v-else class="sync-icon text-green-500" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
          </svg>
          <span class="sync-text">
            {{ isSyncing ? '同步中...' : hasError ? '同步失败' : '已同步' }}
          </span>
        </div>
      </div>
      
      <div class="header-controls">
        <div class="view-mode-toggle">
          <button
            @click="setViewMode('split')"
            :class="{ 'active': viewMode === 'split' }"
            class="mode-btn"
            title="分屏模式"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 002 2m0 0v10"/>
            </svg>
          </button>
          
          <button
            @click="setViewMode('editor')"
            :class="{ 'active': viewMode === 'editor' }"
            class="mode-btn"
            title="编辑器模式"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
            </svg>
          </button>
          
          <button
            @click="setViewMode('preview')"
            :class="{ 'active': viewMode === 'preview' }"
            class="mode-btn"
            title="预览模式"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
            </svg>
          </button>
        </div>
        
        <div class="control-group">
          <button
            @click="toggleSync"
            :class="{ 'active': autoSync }"
            class="sync-toggle-btn"
            title="切换自动同步"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
            {{ autoSync ? '自动' : '手动' }}
          </button>
          
          <button
            @click="manualSync"
            :disabled="autoSync || isSyncing"
            class="manual-sync-btn"
            title="手动同步"
          >
            刷新
          </button>
        </div>
      </div>
    </div>

    <!-- Preview Content -->
    <div class="preview-content" :class="`view-mode-${viewMode}`">
      <!-- Split View -->
      <div v-if="viewMode === 'split'" class="split-view">
        <!-- Editor Panel -->
        <div class="editor-panel" :style="{ width: editorWidth + '%' }">
          <div class="panel-header">
            <span class="panel-title">LaTeX 编辑器</span>
            <div class="panel-controls">
              <button
                @click="formatLatex"
                class="control-btn"
                title="格式化代码"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"/>
                </svg>
              </button>
            </div>
          </div>
          
          <div class="editor-container">
            <textarea
              ref="editorTextarea"
              v-model="localLatex"
              @input="handleEditorInput"
              @scroll="handleEditorScroll"
              @keydown="handleEditorKeydown"
              class="latex-editor"
              placeholder="输入LaTeX公式..."
              spellcheck="false"
            />
            
            <!-- Line numbers -->
            <div class="line-numbers">
              <div
                v-for="n in lineCount"
                :key="n"
                class="line-number"
              >
                {{ n }}
              </div>
            </div>
          </div>
        </div>

        <!-- Resize Handle -->
        <div
          class="resize-handle"
          @mousedown="startResize"
        />

        <!-- Preview Panel -->
        <div class="preview-panel" :style="{ width: (100 - editorWidth) + '%' }">
          <div class="panel-header">
            <span class="panel-title">渲染预览</span>
            <div class="panel-controls">
              <select
                v-model="renderEngine"
                @change="handleEngineChange"
                class="engine-select"
              >
                <option value="mathjax">MathJax</option>
                <option value="katex">KaTeX</option>
              </select>
              
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
          </div>
          
          <div class="preview-container" :style="{ fontSize: zoomLevel + '%' }">
            <FormulaRenderer
              :latex="localLatex"
              :engine="renderEngine"
              :display-mode="displayMode"
              :show-controls="false"
              :show-info="false"
              :auto-render="autoSync"
              @render-success="handleRenderSuccess"
              @render-error="handleRenderError"
            />
          </div>
        </div>
      </div>

      <!-- Editor Only View -->
      <div v-else-if="viewMode === 'editor'" class="editor-only-view">
        <div class="editor-container">
          <textarea
            ref="editorTextarea"
            v-model="localLatex"
            @input="handleEditorInput"
            class="latex-editor fullscreen"
            placeholder="输入LaTeX公式..."
            spellcheck="false"
          />
        </div>
      </div>

      <!-- Preview Only View -->
      <div v-else-if="viewMode === 'preview'" class="preview-only-view">
        <div class="preview-container fullscreen" :style="{ fontSize: zoomLevel + '%' }">
          <FormulaRenderer
            :latex="localLatex"
            :engine="renderEngine"
            :display-mode="displayMode"
            :show-controls="true"
            :show-info="true"
            :auto-render="autoSync"
            @render-success="handleRenderSuccess"
            @render-error="handleRenderError"
          />
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-left">
        <span class="status-item">行数: {{ lineCount }}</span>
        <span class="status-item">字符: {{ characterCount }}</span>
        <span class="status-item">引擎: {{ renderEngine.toUpperCase() }}</span>
        <span v-if="lastRenderTime" class="status-item">渲染: {{ lastRenderTime }}ms</span>
      </div>
      
      <div class="status-right">
        <span v-if="hasError" class="status-error">{{ errorMessage }}</span>
        <span v-else-if="isSyncing" class="status-syncing">同步中...</span>
        <span v-else class="status-ready">就绪</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import FormulaRenderer from './FormulaRenderer.vue'
import type { RenderEngine } from '@/services/mathRenderer'

interface Props {
  latex: string
  displayMode?: 'inline' | 'block'
  engine?: RenderEngine
  autoSync?: boolean
  debounceMs?: number
}

interface Emits {
  (e: 'update:latex', latex: string): void
  (e: 'sync', latex: string): void
  (e: 'render-success', html: string): void
  (e: 'render-error', error: string): void
}

const props = withDefaults(defineProps<Props>(), {
  displayMode: 'block',
  engine: 'mathjax',
  autoSync: true,
  debounceMs: 300
})

const emit = defineEmits<Emits>()

// Refs
const editorTextarea = ref<HTMLTextAreaElement>()

// State
const localLatex = ref(props.latex)
const viewMode = ref<'split' | 'editor' | 'preview'>('split')
const renderEngine = ref<RenderEngine>(props.engine)
const autoSync = ref(props.autoSync)
const isSyncing = ref(false)
const hasError = ref(false)
const errorMessage = ref('')
const zoomLevel = ref(100)
const editorWidth = ref(50) // Percentage for split view
const lastRenderTime = ref<number | null>(null)

// Resize state
const isResizing = ref(false)
const resizeStartX = ref(0)
const resizeStartWidth = ref(50)

// Computed
const lineCount = computed(() => {
  return localLatex.value.split('\n').length
})

const characterCount = computed(() => {
  return localLatex.value.length
})

// Debouncing
let syncTimeout: number | null = null

// Watch for prop changes
watch(() => props.latex, (newLatex) => {
  if (newLatex !== localLatex.value) {
    localLatex.value = newLatex
  }
})

// Watch for local changes
watch(localLatex, (newLatex) => {
  emit('update:latex', newLatex)
  
  if (autoSync.value) {
    debouncedSync()
  }
})

// Methods
function handleEditorInput() {
  // Input is already handled by v-model
  // This is for additional processing if needed
}

function handleEditorScroll() {
  // Could implement synchronized scrolling here
}

function handleEditorKeydown(event: KeyboardEvent) {
  // Handle special key combinations
  if (event.ctrlKey || event.metaKey) {
    switch (event.key) {
      case 's':
        event.preventDefault()
        manualSync()
        break
      case 'Enter':
        if (event.shiftKey) {
          event.preventDefault()
          manualSync()
        }
        break
    }
  }
  
  // Auto-complete brackets and braces
  if (event.key === '{') {
    event.preventDefault()
    insertAtCursor('{', '}')
  } else if (event.key === '(') {
    event.preventDefault()
    insertAtCursor('(', ')')
  }
}

function insertAtCursor(before: string, after: string = '') {
  const textarea = editorTextarea.value
  if (!textarea) return
  
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const selectedText = localLatex.value.substring(start, end)
  
  const newText = before + selectedText + after
  const newValue = localLatex.value.substring(0, start) + newText + localLatex.value.substring(end)
  
  localLatex.value = newValue
  
  nextTick(() => {
    textarea.focus()
    textarea.setSelectionRange(start + before.length, start + before.length + selectedText.length)
  })
}

function formatLatex() {
  // Basic LaTeX formatting
  let formatted = localLatex.value
  
  // Add proper spacing around operators
  formatted = formatted.replace(/([=<>])/g, ' $1 ')
  formatted = formatted.replace(/\s+/g, ' ') // Remove extra spaces
  
  // Format fractions
  formatted = formatted.replace(/\\frac\s*\{([^}]+)\}\s*\{([^}]+)\}/g, '\\frac{$1}{$2}')
  
  localLatex.value = formatted.trim()
}

function setViewMode(mode: 'split' | 'editor' | 'preview') {
  viewMode.value = mode
}

function toggleSync() {
  autoSync.value = !autoSync.value
  if (autoSync.value) {
    manualSync()
  }
}

function manualSync() {
  if (isSyncing.value) return
  
  clearTimeout(syncTimeout!)
  performSync()
}

function debouncedSync() {
  if (syncTimeout) {
    clearTimeout(syncTimeout)
  }
  
  syncTimeout = setTimeout(() => {
    performSync()
  }, props.debounceMs)
}

function performSync() {
  isSyncing.value = true
  hasError.value = false
  errorMessage.value = ''
  
  emit('sync', localLatex.value)
  
  // Simulate sync delay
  setTimeout(() => {
    isSyncing.value = false
  }, 100)
}

function handleRenderSuccess(html: string) {
  hasError.value = false
  errorMessage.value = ''
  emit('render-success', html)
}

function handleRenderError(error: string) {
  hasError.value = true
  errorMessage.value = error
  emit('render-error', error)
}

function handleEngineChange() {
  if (autoSync.value) {
    manualSync()
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

// Resize functionality
function startResize(event: MouseEvent) {
  isResizing.value = true
  resizeStartX.value = event.clientX
  resizeStartWidth.value = editorWidth.value
  
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  
  event.preventDefault()
}

function handleResize(event: MouseEvent) {
  if (!isResizing.value) return
  
  const container = document.querySelector('.split-view') as HTMLElement
  if (!container) return
  
  const containerWidth = container.offsetWidth
  const deltaX = event.clientX - resizeStartX.value
  const deltaPercent = (deltaX / containerWidth) * 100
  
  const newWidth = Math.max(20, Math.min(80, resizeStartWidth.value + deltaPercent))
  editorWidth.value = newWidth
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
}

// Lifecycle
onMounted(() => {
  if (autoSync.value) {
    performSync()
  }
})

onUnmounted(() => {
  if (syncTimeout) {
    clearTimeout(syncTimeout)
  }
  
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
.realtime-preview {
  @apply bg-white rounded-lg border border-gray-200 shadow-sm h-full flex flex-col;
}

.preview-header {
  @apply flex items-center justify-between p-3 border-b border-gray-200 bg-gray-50;
}

.header-left {
  @apply flex items-center gap-3;
}

.preview-title {
  @apply text-lg font-semibold text-gray-800;
}

.sync-indicator {
  @apply flex items-center gap-2 text-sm;
}

.sync-icon {
  @apply w-4 h-4;
}

.sync-text {
  @apply font-medium;
}

.sync-indicator.syncing .sync-text {
  @apply text-blue-600;
}

.sync-indicator.error .sync-text {
  @apply text-red-600;
}

.header-controls {
  @apply flex items-center gap-4;
}

.view-mode-toggle {
  @apply flex border border-gray-300 rounded overflow-hidden;
}

.mode-btn {
  @apply px-3 py-1.5 bg-white hover:bg-gray-50 border-r border-gray-300 last:border-r-0;
  @apply focus:outline-none focus:ring-2 focus:ring-blue-500 focus:z-10;
}

.mode-btn.active {
  @apply bg-blue-100 text-blue-700;
}

.control-group {
  @apply flex items-center gap-2;
}

.sync-toggle-btn {
  @apply flex items-center gap-1 px-3 py-1.5 border border-gray-300 rounded;
  @apply hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500;
}

.sync-toggle-btn.active {
  @apply bg-blue-100 text-blue-700 border-blue-300;
}

.manual-sync-btn {
  @apply px-3 py-1.5 bg-gray-600 text-white rounded hover:bg-gray-700;
  @apply disabled:opacity-50 disabled:cursor-not-allowed;
}

.preview-content {
  @apply flex-1 overflow-hidden;
}

.split-view {
  @apply flex h-full;
}

.editor-panel,
.preview-panel {
  @apply flex flex-col border-r border-gray-200 last:border-r-0;
}

.panel-header {
  @apply flex items-center justify-between p-2 border-b border-gray-200 bg-gray-50;
}

.panel-title {
  @apply text-sm font-medium text-gray-700;
}

.panel-controls {
  @apply flex items-center gap-2;
}

.control-btn {
  @apply p-1 hover:bg-gray-200 rounded disabled:opacity-50 disabled:cursor-not-allowed;
}

.engine-select {
  @apply text-xs border border-gray-300 rounded px-2 py-1;
}

.zoom-display {
  @apply text-xs font-mono text-gray-600 min-w-[2.5rem] text-center;
}

.editor-container {
  @apply flex-1 relative;
}

.latex-editor {
  @apply w-full h-full p-3 border-none outline-none font-mono text-sm resize-none;
  @apply focus:ring-2 focus:ring-blue-500;
}

.latex-editor.fullscreen {
  @apply text-base;
}

.line-numbers {
  @apply absolute left-0 top-0 p-3 text-xs text-gray-500 font-mono pointer-events-none;
  @apply border-r border-gray-200 bg-gray-50;
  width: 3rem;
}

.line-number {
  @apply text-right pr-2;
  line-height: 1.5;
}

.resize-handle {
  @apply w-1 bg-gray-300 hover:bg-blue-400 cursor-col-resize;
  @apply transition-colors duration-200;
}

.preview-container {
  @apply flex-1 p-4 overflow-auto;
}

.preview-container.fullscreen {
  @apply p-6;
}

.editor-only-view,
.preview-only-view {
  @apply h-full;
}

.status-bar {
  @apply flex items-center justify-between p-2 border-t border-gray-200 bg-gray-50 text-xs;
}

.status-left {
  @apply flex items-center gap-4;
}

.status-item {
  @apply text-gray-600;
}

.status-right {
  @apply flex items-center;
}

.status-error {
  @apply text-red-600 font-medium;
}

.status-syncing {
  @apply text-blue-600 font-medium;
}

.status-ready {
  @apply text-green-600 font-medium;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .split-view {
    @apply flex-col;
  }
  
  .editor-panel,
  .preview-panel {
    @apply border-r-0 border-b border-gray-200 last:border-b-0;
  }
  
  .resize-handle {
    @apply h-1 w-full cursor-row-resize;
  }
}
</style>