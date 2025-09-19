<template>
  <div class="export-options">
    <!-- Export Header -->
    <div class="export-header">
      <h3 class="export-title">导出选项</h3>
      <div class="export-actions">
        <button
          @click="resetToDefaults"
          class="btn-outline btn-sm"
          title="重置为默认设置"
        >
          重置
        </button>
        <button
          @click="$emit('close')"
          class="btn-outline btn-sm"
          title="关闭"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Format Selection -->
    <div class="format-section">
      <div class="section-header">
        <label class="section-title">导出格式</label>
        <span class="format-count">{{ availableFormats.length }} 种格式可用</span>
      </div>
      
      <div class="format-grid">
        <div
          v-for="format in availableFormats"
          :key="format.value"
          class="format-option"
          :class="{ 'selected': selectedFormat === format.value }"
          @click="selectFormat(format.value)"
        >
          <div class="format-icon">
            <component :is="getFormatIcon(format.value)" class="w-6 h-6" />
          </div>
          <div class="format-info">
            <div class="format-name">{{ format.label }}</div>
            <div class="format-description">{{ format.description }}</div>
          </div>
          <div class="format-indicator">
            <svg v-if="selectedFormat === format.value" class="w-5 h-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Format Options -->
    <div v-if="selectedFormatInfo" class="options-section">
      <div class="section-header">
        <label class="section-title">格式选项</label>
      </div>
      
      <div class="options-grid">
        <!-- Include Metadata -->
        <div class="option-item">
          <label class="option-label">
            <input
              type="checkbox"
              v-model="exportOptions.includeMetadata"
              class="option-checkbox"
            />
            包含元数据
          </label>
          <p class="option-description">在导出文件中包含生成时间、置信度等信息</p>
        </div>

        <!-- Custom Template (for supported formats) -->
        <div v-if="supportsCustomTemplate" class="option-item">
          <label class="option-label">自定义模板</label>
          <textarea
            v-model="exportOptions.customTemplate"
            class="template-input"
            placeholder="输入自定义模板..."
            rows="4"
          />
          <p class="option-description">使用自定义模板格式化输出</p>
        </div>

        <!-- Format-specific options -->
        <div v-if="selectedFormat === 'Markdown'" class="option-item">
          <label class="option-label">Markdown 公式格式</label>
          <div class="format-controls">
            <div class="control-group">
              <label class="control-label">行内公式:</label>
              <select v-model="markdownOptions.inlineFormat" class="format-select">
                <option value="Dollar">$ ... $</option>
                <option value="Parentheses">\( ... \)</option>
              </select>
            </div>
            <div class="control-group">
              <label class="control-label">块级公式:</label>
              <select v-model="markdownOptions.blockFormat" class="format-select">
                <option value="DoubleDollar">$$ ... $$</option>
                <option value="Brackets">\[ ... \]</option>
              </select>
            </div>
          </div>
        </div>

        <div v-if="selectedFormat === 'HTML'" class="option-item">
          <label class="option-label">HTML 选项</label>
          <div class="html-options">
            <label class="option-checkbox-label">
              <input
                type="checkbox"
                v-model="htmlOptions.includeMathJax"
                class="option-checkbox"
              />
              包含 MathJax 脚本
            </label>
            <label class="option-checkbox-label">
              <input
                type="checkbox"
                v-model="htmlOptions.inlineCSS"
                class="option-checkbox"
              />
              内联 CSS 样式
            </label>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Section -->
    <div class="preview-section">
      <div class="section-header">
        <label class="section-title">预览</label>
        <div class="preview-controls">
          <button
            @click="generatePreview"
            :disabled="isGeneratingPreview"
            class="btn-outline btn-sm"
          >
            {{ isGeneratingPreview ? '生成中...' : '刷新预览' }}
          </button>
          <button
            @click="copyPreview"
            :disabled="!previewContent"
            class="btn-outline btn-sm"
            title="复制预览内容"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
            </svg>
          </button>
        </div>
      </div>
      
      <div class="preview-container">
        <div v-if="isGeneratingPreview" class="preview-loading">
          <div class="loading-spinner"></div>
          <span>生成预览中...</span>
        </div>
        
        <div v-else-if="previewError" class="preview-error">
          <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          <div class="error-content">
            <div class="error-title">预览生成失败</div>
            <div class="error-message">{{ previewError }}</div>
          </div>
        </div>
        
        <div v-else-if="previewContent" class="preview-content">
          <pre class="preview-text">{{ previewContent }}</pre>
        </div>
        
        <div v-else class="preview-empty">
          <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
          <span>点击"刷新预览"查看导出内容</span>
        </div>
      </div>
    </div>

    <!-- Export Actions -->
    <div class="export-footer">
      <div class="export-info">
        <span v-if="lastExportTime" class="info-item">
          上次导出: {{ formatTime(lastExportTime) }}
        </span>
        <span v-if="previewContent" class="info-item">
          预览大小: {{ formatFileSize(previewContent.length) }}
        </span>
      </div>
      
      <div class="export-buttons">
        <button
          @click="copyToClipboard"
          :disabled="!previewContent || isExporting"
          class="btn-outline"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
          </svg>
          复制到剪贴板
        </button>
        
        <button
          @click="saveToFile"
          :disabled="!previewContent || isExporting"
          class="btn-primary"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
          {{ isExporting ? '导出中...' : '保存到文件' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { FormulaResult, AppConfig } from '@/types'

interface Props {
  formulaResult: FormulaResult
  appConfig: AppConfig
}

interface Emits {
  (e: 'export-success', result: any): void
  (e: 'export-error', error: string): void
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Format definitions
const formatDefinitions = {
  LaTeX: {
    label: 'LaTeX',
    description: '标准 LaTeX 格式，适用于学术文档',
    icon: 'DocumentTextIcon',
    extensions: ['.tex'],
    supportsTemplate: true
  },
  LaTeXInline: {
    label: 'LaTeX (行内)',
    description: '行内 LaTeX 格式',
    icon: 'DocumentTextIcon',
    extensions: ['.tex'],
    supportsTemplate: false
  },
  LaTeXBlock: {
    label: 'LaTeX (块级)',
    description: '块级 LaTeX 格式',
    icon: 'DocumentTextIcon',
    extensions: ['.tex'],
    supportsTemplate: false
  },
  Markdown: {
    label: 'Markdown',
    description: '支持数学公式的 Markdown 格式',
    icon: 'HashtagIcon',
    extensions: ['.md'],
    supportsTemplate: true
  },
  MarkdownInline: {
    label: 'Markdown (行内)',
    description: '行内公式的 Markdown 格式',
    icon: 'HashtagIcon',
    extensions: ['.md'],
    supportsTemplate: false
  },
  MarkdownBlock: {
    label: 'Markdown (块级)',
    description: '块级公式的 Markdown 格式',
    icon: 'HashtagIcon',
    extensions: ['.md'],
    supportsTemplate: false
  },
  HTML: {
    label: 'HTML',
    description: '包含 MathJax 的 HTML 文档',
    icon: 'CodeBracketIcon',
    extensions: ['.html'],
    supportsTemplate: true
  },
  DOCX: {
    label: 'Word 文档',
    description: 'Microsoft Word 文档格式',
    icon: 'DocumentIcon',
    extensions: ['.docx'],
    supportsTemplate: false
  },
  PlainText: {
    label: '纯文本',
    description: '纯文本格式，无格式化',
    icon: 'DocumentTextIcon',
    extensions: ['.txt'],
    supportsTemplate: false
  }
}

// State
const availableFormats = ref<Array<{value: string, label: string, description: string}>>([])
const selectedFormat = ref<string>('')
const exportOptions = ref({
  includeMetadata: true,
  customTemplate: ''
})
const markdownOptions = ref({
  inlineFormat: 'Dollar',
  blockFormat: 'DoubleDollar'
})
const htmlOptions = ref({
  includeMathJax: true,
  inlineCSS: false
})
const previewContent = ref('')
const previewError = ref('')
const isGeneratingPreview = ref(false)
const isExporting = ref(false)
const lastExportTime = ref<Date | null>(null)

// Computed
const selectedFormatInfo = computed(() => {
  return formatDefinitions[selectedFormat.value as keyof typeof formatDefinitions]
})

const supportsCustomTemplate = computed(() => {
  return selectedFormatInfo.value?.supportsTemplate || false
})

// Methods
async function loadAvailableFormats() {
  try {
    const formats = await invoke<string[]>('get_available_export_formats', {
      inputType: props.formulaResult.inputType,
      appConfig: props.appConfig
    })
    
    availableFormats.value = formats.map(format => ({
      value: format,
      label: formatDefinitions[format as keyof typeof formatDefinitions]?.label || format,
      description: formatDefinitions[format as keyof typeof formatDefinitions]?.description || ''
    }))
    
    // Set default format
    if (availableFormats.value.length > 0 && !selectedFormat.value) {
      const defaultFormat = await invoke<string>('get_default_export_format', {
        inputType: props.formulaResult.inputType,
        appConfig: props.appConfig
      })
      selectedFormat.value = defaultFormat
    }
  } catch (error) {
    console.error('Failed to load available formats:', error)
  }
}

function selectFormat(format: string) {
  selectedFormat.value = format
  // Clear preview when format changes
  previewContent.value = ''
  previewError.value = ''
}

async function generatePreview() {
  if (!selectedFormat.value) return
  
  isGeneratingPreview.value = true
  previewError.value = ''
  
  try {
    const exportConfig = {
      format: selectedFormat.value,
      includeMetadata: exportOptions.value.includeMetadata,
      customTemplate: exportOptions.value.customTemplate || null,
      formatOptions: getFormatOptions()
    }
    
    const result = await invoke<any>('export_formula_result', {
      result: props.formulaResult,
      exportConfig,
      appConfig: props.appConfig
    })
    
    previewContent.value = result.content
  } catch (error) {
    previewError.value = error instanceof Error ? error.message : '预览生成失败'
  } finally {
    isGeneratingPreview.value = false
  }
}

function getFormatOptions(): Record<string, string> {
  const options: Record<string, string> = {}
  
  if (selectedFormat.value === 'Markdown') {
    options.inlineFormat = markdownOptions.value.inlineFormat
    options.blockFormat = markdownOptions.value.blockFormat
  } else if (selectedFormat.value === 'HTML') {
    options.includeMathJax = htmlOptions.value.includeMathJax.toString()
    options.inlineCSS = htmlOptions.value.inlineCSS.toString()
  }
  
  return options
}

async function copyToClipboard() {
  if (!previewContent.value) return
  
  try {
    await navigator.clipboard.writeText(previewContent.value)
    // Could show a toast notification here
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
  }
}

async function copyPreview() {
  await copyToClipboard()
}

async function saveToFile() {
  if (!previewContent.value || !selectedFormatInfo.value) return
  
  isExporting.value = true
  
  try {
    const extensions = selectedFormatInfo.value.extensions
    const defaultPath = `formula${extensions[0]}`
    
    const filePath = await save({
      defaultPath,
      filters: [{
        name: selectedFormatInfo.value.label,
        extensions: extensions.map(ext => ext.substring(1)) // Remove the dot
      }]
    })
    
    if (filePath) {
      const exportConfig = {
        format: selectedFormat.value,
        includeMetadata: exportOptions.value.includeMetadata,
        customTemplate: exportOptions.value.customTemplate || null,
        formatOptions: getFormatOptions()
      }
      
      await invoke('export_to_file', {
        result: props.formulaResult,
        exportConfig,
        appConfig: props.appConfig,
        filePath
      })
      
      lastExportTime.value = new Date()
      emit('export-success', { filePath, format: selectedFormat.value })
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : '导出失败'
    emit('export-error', errorMessage)
  } finally {
    isExporting.value = false
  }
}

function resetToDefaults() {
  exportOptions.value = {
    includeMetadata: true,
    customTemplate: ''
  }
  markdownOptions.value = {
    inlineFormat: 'Dollar',
    blockFormat: 'DoubleDollar'
  }
  htmlOptions.value = {
    includeMathJax: true,
    inlineCSS: false
  }
  previewContent.value = ''
  previewError.value = ''
}

function getFormatIcon(_format: string) {
  // Return a simple div for now - in a real app you'd import and return the actual icon component
  return 'div'
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString()
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// Watch for changes
watch(selectedFormat, () => {
  if (selectedFormat.value) {
    generatePreview()
  }
})

watch(() => exportOptions.value, () => {
  if (previewContent.value) {
    generatePreview()
  }
}, { deep: true })

watch(() => markdownOptions.value, () => {
  if (selectedFormat.value === 'Markdown' && previewContent.value) {
    generatePreview()
  }
}, { deep: true })

watch(() => htmlOptions.value, () => {
  if (selectedFormat.value === 'HTML' && previewContent.value) {
    generatePreview()
  }
}, { deep: true })

// Lifecycle
onMounted(() => {
  loadAvailableFormats()
})
</script>

<style scoped>
.export-options {
  @apply bg-white rounded-lg border border-gray-200 shadow-sm max-w-4xl mx-auto;
}

.export-header {
  @apply flex items-center justify-between p-4 border-b border-gray-200;
}

.export-title {
  @apply text-lg font-semibold text-gray-800;
}

.export-actions {
  @apply flex gap-2;
}

.format-section,
.options-section,
.preview-section {
  @apply p-4 border-b border-gray-200 last:border-b-0;
}

.section-header {
  @apply flex items-center justify-between mb-3;
}

.section-title {
  @apply text-sm font-medium text-gray-700;
}

.format-count {
  @apply text-xs text-gray-500;
}

.format-grid {
  @apply grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3;
}

.format-option {
  @apply flex items-center gap-3 p-3 border border-gray-200 rounded-lg cursor-pointer;
  @apply hover:border-blue-300 hover:bg-blue-50 transition-colors;
}

.format-option.selected {
  @apply border-blue-500 bg-blue-50;
}

.format-icon {
  @apply flex-shrink-0 w-8 h-8 bg-gray-100 rounded flex items-center justify-center;
}

.format-info {
  @apply flex-1;
}

.format-name {
  @apply font-medium text-gray-900;
}

.format-description {
  @apply text-sm text-gray-600;
}

.format-indicator {
  @apply flex-shrink-0;
}

.options-grid {
  @apply space-y-4;
}

.option-item {
  @apply space-y-2;
}

.option-label {
  @apply flex items-center gap-2 font-medium text-gray-700;
}

.option-checkbox {
  @apply rounded border-gray-300 text-blue-600 focus:ring-blue-500;
}

.option-description {
  @apply text-sm text-gray-600;
}

.template-input {
  @apply w-full border border-gray-300 rounded-md p-2 text-sm font-mono;
  @apply focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
}

.format-controls {
  @apply space-y-2;
}

.control-group {
  @apply flex items-center gap-2;
}

.control-label {
  @apply text-sm text-gray-600 min-w-[4rem];
}

.format-select {
  @apply border border-gray-300 rounded px-2 py-1 text-sm;
}

.html-options {
  @apply space-y-2;
}

.option-checkbox-label {
  @apply flex items-center gap-2 text-sm;
}

.preview-controls {
  @apply flex gap-2;
}

.preview-container {
  @apply border border-gray-200 rounded-lg min-h-32 max-h-64 overflow-auto;
}

.preview-loading {
  @apply flex items-center justify-center gap-2 p-8 text-blue-600;
}

.loading-spinner {
  @apply w-5 h-5 border-2 border-blue-600 border-t-transparent rounded-full animate-spin;
}

.preview-error {
  @apply flex items-start gap-3 p-4 text-red-600;
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

.preview-content {
  @apply p-4;
}

.preview-text {
  @apply text-sm font-mono whitespace-pre-wrap break-words;
}

.preview-empty {
  @apply flex flex-col items-center justify-center gap-2 p-8 text-gray-500;
}

.empty-icon {
  @apply w-8 h-8;
}

.export-footer {
  @apply flex items-center justify-between p-4 bg-gray-50;
}

.export-info {
  @apply flex gap-4 text-sm text-gray-600;
}

.export-buttons {
  @apply flex gap-2;
}

/* Button styles */
.btn-primary {
  @apply bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed;
  @apply flex items-center;
}

.btn-outline {
  @apply border border-gray-300 text-gray-700 px-4 py-2 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed;
  @apply flex items-center;
}

.btn-sm {
  @apply text-sm px-2 py-1;
}
</style>