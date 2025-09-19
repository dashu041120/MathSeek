<template>
  <div class="formula-editor">
    <!-- Editor Header -->
    <div class="editor-header">
      <h3 class="text-lg font-semibold text-gray-800">公式编辑器</h3>
      <div class="header-actions">
        <button
          @click="resetToOriginal"
          :disabled="!hasChanges"
          class="btn-secondary btn-sm"
          title="重置到原始识别结果"
        >
          重置
        </button>
        <button
          @click="validateSyntax"
          class="btn-outline btn-sm"
          title="验证LaTeX语法"
        >
          验证语法
        </button>
      </div>
    </div>

    <!-- Editor Layout -->
    <div class="editor-layout">
      <!-- LaTeX Input Section -->
      <div class="editor-section">
        <div class="section-header">
          <label class="section-title">LaTeX 代码</label>
          <div class="syntax-status">
            <span
              v-if="syntaxError"
              class="status-error"
              :title="syntaxError"
            >
              语法错误
            </span>
            <span
              v-else-if="modelValue.trim()"
              class="status-valid"
            >
              语法正确
            </span>
          </div>
        </div>
        
        <!-- LaTeX Textarea with Syntax Highlighting -->
        <div class="editor-container">
          <textarea
            ref="editorTextarea"
            v-model="localValue"
            @input="handleInput"
            @keydown="handleKeydown"
            @scroll="syncScroll"
            class="latex-editor"
            :class="{ 'has-error': syntaxError }"
            placeholder="输入或编辑LaTeX公式..."
            spellcheck="false"
          />
          
          <!-- Syntax Highlighting Overlay -->
          <div
            ref="highlightOverlay"
            class="highlight-overlay"
            v-html="highlightedContent"
          />
        </div>

        <!-- Syntax Error Display -->
        <div v-if="syntaxError" class="error-message">
          <svg class="error-icon" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
          {{ syntaxError }}
        </div>

        <!-- LaTeX Toolbar -->
        <div class="latex-toolbar">
          <button
            v-for="symbol in commonSymbols"
            :key="symbol.name"
            @click="insertSymbol(symbol.latex)"
            class="symbol-btn"
            :title="symbol.name"
          >
            {{ symbol.display }}
          </button>
        </div>
      </div>

      <!-- Preview Section -->
      <div class="preview-section">
        <div class="section-header">
          <label class="section-title">实时预览</label>
          <div class="preview-controls">
            <button
              @click="togglePreviewMode"
              class="btn-outline btn-sm"
            >
              {{ previewMode === 'rendered' ? '源码' : '渲染' }}
            </button>
            <button
              @click="zoomIn"
              class="btn-outline btn-sm"
              :disabled="zoomLevel >= 200"
            >
              放大
            </button>
            <button
              @click="zoomOut"
              class="btn-outline btn-sm"
              :disabled="zoomLevel <= 50"
            >
              缩小
            </button>
          </div>
        </div>

        <!-- Preview Content -->
        <div class="preview-container" :style="{ fontSize: zoomLevel + '%' }">
          <div
            v-if="previewMode === 'rendered'"
            ref="mathPreview"
            class="math-preview"
            v-html="renderedFormula"
          />
          <pre
            v-else
            class="source-preview"
          >{{ modelValue }}</pre>
        </div>
      </div>
    </div>

    <!-- Editor Footer -->
    <div class="editor-footer">
      <div class="editor-stats">
        <span class="stat">字符数: {{ modelValue.length }}</span>
        <span class="stat">行数: {{ lineCount }}</span>
        <span v-if="hasChanges" class="stat modified">已修改</span>
      </div>
      
      <div class="footer-actions">
        <button
          @click="copyToClipboard"
          class="btn-outline btn-sm"
        >
          复制LaTeX
        </button>
        <button
          @click="$emit('save')"
          :disabled="!hasChanges || !!syntaxError"
          class="btn-primary btn-sm"
        >
          保存修改
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'

interface Props {
  modelValue: string
  originalValue?: string
  disabled?: boolean
  autoFocus?: boolean
}

interface Emits {
  (e: 'update:modelValue', value: string): void
  (e: 'save'): void
  (e: 'reset'): void
  (e: 'syntax-error', error: string | null): void
}

const props = withDefaults(defineProps<Props>(), {
  originalValue: '',
  disabled: false,
  autoFocus: false
})

const emit = defineEmits<Emits>()

// Refs
const editorTextarea = ref<HTMLTextAreaElement>()
const highlightOverlay = ref<HTMLDivElement>()
const mathPreview = ref<HTMLDivElement>()

// State
const localValue = ref(props.modelValue)
const syntaxError = ref<string | null>(null)
const previewMode = ref<'rendered' | 'source'>('rendered')
const zoomLevel = ref(100)
const isValidating = ref(false)

// Common LaTeX symbols for toolbar
const commonSymbols = [
  { name: '分数', latex: '\\frac{a}{b}', display: '𝑎/𝑏' },
  { name: '上标', latex: '^{2}', display: 'x²' },
  { name: '下标', latex: '_{i}', display: 'xᵢ' },
  { name: '根号', latex: '\\sqrt{x}', display: '√' },
  { name: '积分', latex: '\\int', display: '∫' },
  { name: '求和', latex: '\\sum', display: '∑' },
  { name: '极限', latex: '\\lim', display: 'lim' },
  { name: '无穷', latex: '\\infty', display: '∞' },
  { name: '阿尔法', latex: '\\alpha', display: 'α' },
  { name: '贝塔', latex: '\\beta', display: 'β' },
  { name: '伽马', latex: '\\gamma', display: 'γ' },
  { name: '德尔塔', latex: '\\delta', display: 'δ' },
  { name: '等于', latex: '=', display: '=' },
  { name: '不等于', latex: '\\neq', display: '≠' },
  { name: '小于等于', latex: '\\leq', display: '≤' },
  { name: '大于等于', latex: '\\geq', display: '≥' }
]

// Computed properties
const hasChanges = computed(() => {
  return localValue.value !== (props.originalValue || props.modelValue)
})

const lineCount = computed(() => {
  return localValue.value.split('\n').length
})

const highlightedContent = computed(() => {
  return highlightLatexSyntax(localValue.value)
})

const renderedFormula = computed(() => {
  if (syntaxError.value || !localValue.value.trim()) {
    return '<div class="preview-placeholder">输入LaTeX代码查看预览</div>'
  }
  
  // This would be replaced with actual MathJax/KaTeX rendering
  return `<div class="math-formula">${escapeHtml(localValue.value)}</div>`
})

// Watch for prop changes
watch(() => props.modelValue, (newValue) => {
  if (newValue !== localValue.value) {
    localValue.value = newValue
  }
})

// Watch for local changes
watch(localValue, (newValue) => {
  emit('update:modelValue', newValue)
  validateSyntaxDebounced()
})

// Methods
function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement
  localValue.value = target.value
}

function handleKeydown(event: KeyboardEvent) {
  // Handle special key combinations
  if (event.ctrlKey || event.metaKey) {
    switch (event.key) {
      case 's':
        event.preventDefault()
        if (!syntaxError.value && hasChanges.value) {
          emit('save')
        }
        break
      case 'z':
        if (!event.shiftKey) {
          // Undo functionality could be implemented here
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
  const selectedText = localValue.value.substring(start, end)
  
  const newText = before + selectedText + after
  const newValue = localValue.value.substring(0, start) + newText + localValue.value.substring(end)
  
  localValue.value = newValue
  
  nextTick(() => {
    textarea.focus()
    textarea.setSelectionRange(start + before.length, start + before.length + selectedText.length)
  })
}

function insertSymbol(latex: string) {
  insertAtCursor(latex)
}

function syncScroll() {
  if (highlightOverlay.value && editorTextarea.value) {
    highlightOverlay.value.scrollTop = editorTextarea.value.scrollTop
    highlightOverlay.value.scrollLeft = editorTextarea.value.scrollLeft
  }
}

function resetToOriginal() {
  localValue.value = props.originalValue || props.modelValue
  emit('reset')
}

function togglePreviewMode() {
  previewMode.value = previewMode.value === 'rendered' ? 'source' : 'rendered'
}

function zoomIn() {
  if (zoomLevel.value < 200) {
    zoomLevel.value += 25
  }
}

function zoomOut() {
  if (zoomLevel.value > 50) {
    zoomLevel.value -= 25
  }
}

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(localValue.value)
    // Could show a toast notification here
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
  }
}

// Syntax validation with debouncing
let validationTimeout: number | null = null

function validateSyntaxDebounced() {
  if (validationTimeout) {
    clearTimeout(validationTimeout)
  }
  
  validationTimeout = setTimeout(() => {
    validateSyntax()
  }, 500)
}

function validateSyntax() {
  if (isValidating.value) return
  
  isValidating.value = true
  const error = validateLatexSyntax(localValue.value)
  syntaxError.value = error
  emit('syntax-error', error)
  isValidating.value = false
}

// LaTeX syntax validation
function validateLatexSyntax(latex: string): string | null {
  const trimmed = latex.trim()
  if (!trimmed) return null
  
  // Check for balanced braces
  let braceCount = 0
  let inMathMode = false
  let i = 0
  
  while (i < trimmed.length) {
    const char = trimmed[i]
    
    if (char === '\\' && i + 1 < trimmed.length) {
      // Skip escaped characters
      i += 2
      continue
    }
    
    switch (char) {
      case '{':
        braceCount++
        break
      case '}':
        braceCount--
        if (braceCount < 0) {
          return '不匹配的右花括号'
        }
        break
      case '$':
        inMathMode = !inMathMode
        break
    }
    
    i++
  }
  
  if (braceCount !== 0) {
    return braceCount > 0 ? '缺少右花括号' : '多余的右花括号'
  }
  
  if (inMathMode) {
    return '未闭合的数学模式'
  }
  
  // Check for common LaTeX command errors
  const invalidCommands = trimmed.match(/\\[a-zA-Z]+/g)
  if (invalidCommands) {
    const knownCommands = [
      'frac', 'sqrt', 'sum', 'int', 'lim', 'alpha', 'beta', 'gamma', 'delta',
      'epsilon', 'theta', 'lambda', 'mu', 'pi', 'sigma', 'phi', 'omega',
      'infty', 'partial', 'nabla', 'cdot', 'times', 'div', 'pm', 'mp',
      'leq', 'geq', 'neq', 'approx', 'equiv', 'sim', 'propto',
      'left', 'right', 'begin', 'end', 'text', 'mathbf', 'mathit', 'mathrm'
    ]
    
    for (const cmd of invalidCommands) {
      const cmdName = cmd.substring(1)
      if (!knownCommands.includes(cmdName)) {
        return `未知的LaTeX命令: ${cmd}`
      }
    }
  }
  
  return null
}

// LaTeX syntax highlighting
function highlightLatexSyntax(latex: string): string {
  let highlighted = escapeHtml(latex)
  
  // Highlight LaTeX commands
  highlighted = highlighted.replace(
    /\\([a-zA-Z]+)/g,
    '<span class="latex-command">\\$1</span>'
  )
  
  // Highlight braces
  highlighted = highlighted.replace(
    /[{}]/g,
    '<span class="latex-brace">$&</span>'
  )
  
  // Highlight math delimiters
  highlighted = highlighted.replace(
    /[$]/g,
    '<span class="latex-delimiter">$&</span>'
  )
  
  return highlighted
}

function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

// Lifecycle
onMounted(() => {
  if (props.autoFocus && editorTextarea.value) {
    editorTextarea.value.focus()
  }
  
  // Initial syntax validation
  validateSyntax()
})

onUnmounted(() => {
  if (validationTimeout) {
    clearTimeout(validationTimeout)
  }
})
</script>

<style scoped>
.formula-editor {
  @apply bg-white rounded-lg border border-gray-200 shadow-sm;
}

.editor-header {
  @apply flex items-center justify-between p-4 border-b border-gray-200;
}

.header-actions {
  @apply flex gap-2;
}

.editor-layout {
  @apply grid grid-cols-1 lg:grid-cols-2 gap-4 p-4;
}

.editor-section,
.preview-section {
  @apply space-y-3;
}

.section-header {
  @apply flex items-center justify-between;
}

.section-title {
  @apply text-sm font-medium text-gray-700;
}

.syntax-status {
  @apply text-xs;
}

.status-error {
  @apply text-red-600 font-medium;
}

.status-valid {
  @apply text-green-600 font-medium;
}

.editor-container {
  @apply relative;
}

.latex-editor {
  @apply w-full h-48 p-3 border border-gray-300 rounded-md font-mono text-sm resize-none;
  @apply focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
  @apply relative z-10 bg-transparent;
}

.latex-editor.has-error {
  @apply border-red-300 focus:ring-red-500 focus:border-red-500;
}

.highlight-overlay {
  @apply absolute inset-0 p-3 font-mono text-sm pointer-events-none overflow-hidden;
  @apply whitespace-pre-wrap break-words;
  z-index: 1;
}

.error-message {
  @apply flex items-center gap-2 text-sm text-red-600 bg-red-50 p-2 rounded;
}

.error-icon {
  @apply w-4 h-4 flex-shrink-0;
}

.latex-toolbar {
  @apply flex flex-wrap gap-1 p-2 bg-gray-50 rounded border;
}

.symbol-btn {
  @apply px-2 py-1 text-sm bg-white border border-gray-200 rounded hover:bg-gray-50;
  @apply focus:outline-none focus:ring-1 focus:ring-blue-500;
}

.preview-controls {
  @apply flex gap-2;
}

.preview-container {
  @apply min-h-48 p-4 border border-gray-200 rounded-md bg-gray-50;
}

.math-preview {
  @apply text-center;
}

.source-preview {
  @apply font-mono text-sm whitespace-pre-wrap break-words;
}

.preview-placeholder {
  @apply text-gray-500 text-center italic;
}

.math-formula {
  @apply font-serif text-lg;
}

.editor-footer {
  @apply flex items-center justify-between p-4 border-t border-gray-200 bg-gray-50;
}

.editor-stats {
  @apply flex gap-4 text-sm text-gray-600;
}

.stat.modified {
  @apply text-orange-600 font-medium;
}

.footer-actions {
  @apply flex gap-2;
}

/* Syntax highlighting styles */
:deep(.latex-command) {
  @apply text-blue-600 font-semibold;
}

:deep(.latex-brace) {
  @apply text-purple-600 font-bold;
}

:deep(.latex-delimiter) {
  @apply text-green-600 font-bold;
}

/* Button styles */
.btn-primary {
  @apply bg-blue-600 text-white px-3 py-1.5 rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed;
}

.btn-secondary {
  @apply bg-gray-600 text-white px-3 py-1.5 rounded hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed;
}

.btn-outline {
  @apply border border-gray-300 text-gray-700 px-3 py-1.5 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed;
}

.btn-sm {
  @apply text-sm px-2 py-1;
}
</style>