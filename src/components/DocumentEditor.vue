<template>
  <div class="document-editor">
    <!-- Editor Header -->
    <div class="editor-header">
      <div class="header-left">
        <h3 class="text-lg font-semibold text-gray-800">文档编辑器</h3>
        <span v-if="documentTitle" class="document-title">{{ documentTitle }}</span>
      </div>
      
      <div class="header-actions">
        <button
          @click="addSection"
          class="btn-primary btn-sm"
          title="添加新章节"
        >
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
          </svg>
          添加章节
        </button>
        
        <button
          @click="togglePreview"
          class="btn-outline btn-sm"
        >
          {{ showPreview ? '编辑模式' : '预览模式' }}
        </button>
        
        <button
          @click="saveDocument"
          :disabled="!hasChanges || isSaving"
          class="btn-secondary btn-sm"
        >
          {{ isSaving ? '保存中...' : '保存文档' }}
        </button>
      </div>
    </div>

    <!-- Document Title Editor -->
    <div class="title-section">
      <input
        v-model="localDocument.title"
        @input="markAsModified"
        class="title-input"
        placeholder="输入文档标题..."
        maxlength="200"
      />
    </div>

    <!-- Main Content Area -->
    <div class="content-area" :class="{ 'preview-mode': showPreview }">
      <!-- Edit Mode -->
      <div v-if="!showPreview" class="edit-mode">
        <!-- Sections List -->
        <div class="sections-container">
          <div
            v-for="(section, sectionIndex) in localDocument.sections"
            :key="`section-${sectionIndex}`"
            class="section-item"
            :class="{ 'active': activeSectionIndex === sectionIndex }"
          >
            <!-- Section Header -->
            <div class="section-header">
              <div class="section-controls">
                <button
                  @click="setActiveSection(sectionIndex)"
                  class="section-toggle"
                  :class="{ 'expanded': activeSectionIndex === sectionIndex }"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                  </svg>
                </button>
                
                <input
                  v-model="section.heading"
                  @input="markAsModified"
                  class="section-heading-input"
                  placeholder="章节标题..."
                  maxlength="100"
                />
              </div>
              
              <div class="section-actions">
                <button
                  @click="moveSection(sectionIndex, -1)"
                  :disabled="sectionIndex === 0"
                  class="action-btn"
                  title="上移"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/>
                  </svg>
                </button>
                
                <button
                  @click="moveSection(sectionIndex, 1)"
                  :disabled="sectionIndex === localDocument.sections.length - 1"
                  class="action-btn"
                  title="下移"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                  </svg>
                </button>
                
                <button
                  @click="duplicateSection(sectionIndex)"
                  class="action-btn"
                  title="复制章节"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                  </svg>
                </button>
                
                <button
                  @click="removeSection(sectionIndex)"
                  :disabled="localDocument.sections.length <= 1"
                  class="action-btn text-red-600"
                  title="删除章节"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                  </svg>
                </button>
              </div>
            </div>

            <!-- Section Content (Expanded) -->
            <div
              v-if="activeSectionIndex === sectionIndex"
              class="section-content"
            >
              <!-- Text Content Editor -->
              <div class="text-editor-section">
                <label class="content-label">文本内容</label>
                <textarea
                  v-model="section.text"
                  @input="markAsModified"
                  class="text-editor"
                  placeholder="输入章节文本内容..."
                  rows="6"
                />
              </div>

              <!-- Formulas Section -->
              <div class="formulas-section">
                <div class="formulas-header">
                  <label class="content-label">公式列表</label>
                  <button
                    @click="addFormula(sectionIndex)"
                    class="btn-outline btn-sm"
                  >
                    <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
                    </svg>
                    添加公式
                  </button>
                </div>

                <!-- Formula Items -->
                <div class="formulas-list">
                  <div
                    v-for="(formula, formulaIndex) in section.formulas"
                    :key="`formula-${sectionIndex}-${formulaIndex}`"
                    class="formula-item"
                  >
                    <div class="formula-header">
                      <span class="formula-index">公式 {{ formulaIndex + 1 }}</span>
                      
                      <div class="formula-controls">
                        <label class="inline-checkbox">
                          <input
                            type="checkbox"
                            v-model="formula.isInline"
                            @change="markAsModified"
                          />
                          行内公式
                        </label>
                        
                        <input
                          type="number"
                          v-model.number="formula.position"
                          @input="markAsModified"
                          class="position-input"
                          min="0"
                          title="在文本中的位置"
                        />
                        
                        <button
                          @click="removeFormula(sectionIndex, formulaIndex)"
                          class="action-btn text-red-600"
                          title="删除公式"
                        >
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                          </svg>
                        </button>
                      </div>
                    </div>

                    <!-- Formula Editor -->
                    <div class="formula-editor-container">
                      <textarea
                        v-model="formula.latex"
                        @input="markAsModified"
                        class="formula-input"
                        placeholder="输入LaTeX公式..."
                        rows="3"
                      />
                      
                      <!-- Formula Preview -->
                      <div class="formula-preview">
                        <FormulaRenderer
                          :latex="formula.latex"
                          :display-mode="formula.isInline ? 'inline' : 'block'"
                          :show-controls="false"
                          :show-info="false"
                          :auto-render="true"
                        />
                      </div>
                    </div>
                  </div>
                  
                  <!-- Empty State -->
                  <div v-if="section.formulas.length === 0" class="empty-formulas">
                    <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                    </svg>
                    <span class="empty-text">暂无公式，点击"添加公式"开始</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Empty State -->
          <div v-if="localDocument.sections.length === 0" class="empty-sections">
            <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
            <span class="empty-text">暂无章节，点击"添加章节"开始</span>
          </div>
        </div>
      </div>

      <!-- Preview Mode -->
      <div v-else class="preview-mode">
        <DocumentPreview
          :document="localDocument"
          :render-formulas="true"
        />
      </div>
    </div>

    <!-- Editor Footer -->
    <div class="editor-footer">
      <div class="editor-stats">
        <span class="stat">章节数: {{ localDocument.sections.length }}</span>
        <span class="stat">公式数: {{ totalFormulas }}</span>
        <span class="stat">字符数: {{ totalCharacters }}</span>
        <span v-if="hasChanges" class="stat modified">已修改</span>
      </div>
      
      <div class="footer-actions">
        <button
          @click="resetDocument"
          :disabled="!hasChanges"
          class="btn-outline btn-sm"
        >
          重置
        </button>
        
        <button
          @click="exportDocument"
          class="btn-outline btn-sm"
        >
          导出
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import FormulaRenderer from './FormulaRenderer.vue'
import DocumentPreview from './DocumentPreview.vue'
import type { DocumentContent, DocumentSection } from '@/types'
import { createDocumentSection, createFormulaBlock } from '@/types'

interface Props {
  document: DocumentContent
  disabled?: boolean
  autoSave?: boolean
}

interface Emits {
  (e: 'update:document', document: DocumentContent): void
  (e: 'save', document: DocumentContent): void
  (e: 'export', document: DocumentContent): void
  (e: 'change', hasChanges: boolean): void
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  autoSave: false
})

const emit = defineEmits<Emits>()

// State
const localDocument = ref<DocumentContent>(JSON.parse(JSON.stringify(props.document)))
const originalDocument = ref<DocumentContent>(JSON.parse(JSON.stringify(props.document)))
const activeSectionIndex = ref(0)
const showPreview = ref(false)
const isSaving = ref(false)
const hasChanges = ref(false)

// Computed properties
const documentTitle = computed(() => localDocument.value.title || '未命名文档')

const totalFormulas = computed(() => {
  return localDocument.value.sections.reduce((total, section) => {
    return total + section.formulas.length
  }, 0)
})

const totalCharacters = computed(() => {
  let count = localDocument.value.title?.length || 0
  
  localDocument.value.sections.forEach(section => {
    count += (section.heading?.length || 0)
    count += section.text.length
    section.formulas.forEach(formula => {
      count += formula.latex.length
    })
  })
  
  return count
})

// Watch for changes
watch(localDocument, () => {
  checkForChanges()
  emit('update:document', localDocument.value)
}, { deep: true })

watch(hasChanges, (newValue) => {
  emit('change', newValue)
})

// Methods
function checkForChanges() {
  const current = JSON.stringify(localDocument.value)
  const original = JSON.stringify(originalDocument.value)
  hasChanges.value = current !== original
}

function markAsModified() {
  nextTick(() => {
    checkForChanges()
  })
}

function setActiveSection(index: number) {
  activeSectionIndex.value = activeSectionIndex.value === index ? -1 : index
}

function addSection() {
  const newSection = createDocumentSection(`章节 ${localDocument.value.sections.length + 1}`)
  localDocument.value.sections.push(newSection)
  activeSectionIndex.value = localDocument.value.sections.length - 1
  markAsModified()
}

function removeSection(index: number) {
  if (localDocument.value.sections.length <= 1) return
  
  localDocument.value.sections.splice(index, 1)
  
  // Adjust active section index
  if (activeSectionIndex.value >= localDocument.value.sections.length) {
    activeSectionIndex.value = localDocument.value.sections.length - 1
  }
  
  markAsModified()
}

function moveSection(index: number, direction: number) {
  const newIndex = index + direction
  
  if (newIndex < 0 || newIndex >= localDocument.value.sections.length) return
  
  const sections = localDocument.value.sections
  const temp = sections[index]
  sections[index] = sections[newIndex]
  sections[newIndex] = temp
  
  // Update active section index
  if (activeSectionIndex.value === index) {
    activeSectionIndex.value = newIndex
  } else if (activeSectionIndex.value === newIndex) {
    activeSectionIndex.value = index
  }
  
  markAsModified()
}

function duplicateSection(index: number) {
  const originalSection = localDocument.value.sections[index]
  const duplicatedSection: DocumentSection = {
    heading: (originalSection.heading || '') + ' (副本)',
    text: originalSection.text,
    formulas: originalSection.formulas.map(formula => ({
      latex: formula.latex,
      position: formula.position,
      isInline: formula.isInline
    }))
  }
  
  localDocument.value.sections.splice(index + 1, 0, duplicatedSection)
  activeSectionIndex.value = index + 1
  markAsModified()
}

function addFormula(sectionIndex: number) {
  const section = localDocument.value.sections[sectionIndex]
  const newFormula = createFormulaBlock('', section.text.length, false)
  section.formulas.push(newFormula)
  markAsModified()
}

function removeFormula(sectionIndex: number, formulaIndex: number) {
  const section = localDocument.value.sections[sectionIndex]
  section.formulas.splice(formulaIndex, 1)
  markAsModified()
}

function togglePreview() {
  showPreview.value = !showPreview.value
}

async function saveDocument() {
  if (isSaving.value || !hasChanges.value) return
  
  isSaving.value = true
  
  try {
    // Validate document structure
    validateDocument()
    
    // Update original document
    originalDocument.value = JSON.parse(JSON.stringify(localDocument.value))
    hasChanges.value = false
    
    emit('save', localDocument.value)
  } catch (error) {
    console.error('Failed to save document:', error)
    throw error
  } finally {
    isSaving.value = false
  }
}

function resetDocument() {
  localDocument.value = JSON.parse(JSON.stringify(originalDocument.value))
  hasChanges.value = false
  activeSectionIndex.value = 0
}

function exportDocument() {
  emit('export', localDocument.value)
}

function validateDocument() {
  if (localDocument.value.sections.length === 0) {
    throw new Error('文档必须包含至少一个章节')
  }
  
  for (const section of localDocument.value.sections) {
    if (!section.text.trim() && section.formulas.length === 0) {
      throw new Error('章节必须包含文本内容或公式')
    }
    
    // Validate formula positions
    for (const formula of section.formulas) {
      if (formula.position < 0 || formula.position > section.text.length) {
        throw new Error('公式位置超出文本范围')
      }
    }
  }
}

// Initialize
if (localDocument.value.sections.length === 0) {
  addSection()
}
</script>

<style scoped>
.document-editor {
  @apply bg-white rounded-lg border border-gray-200 shadow-sm;
}

.editor-header {
  @apply flex items-center justify-between p-4 border-b border-gray-200;
}

.header-left {
  @apply flex items-center gap-3;
}

.document-title {
  @apply text-sm text-gray-600 italic;
}

.header-actions {
  @apply flex gap-2;
}

.title-section {
  @apply p-4 border-b border-gray-200;
}

.title-input {
  @apply w-full text-xl font-semibold border-none outline-none placeholder-gray-400;
  @apply focus:ring-2 focus:ring-blue-500 rounded px-2 py-1;
}

.content-area {
  @apply min-h-96;
}

.edit-mode {
  @apply p-4;
}

.sections-container {
  @apply space-y-4;
}

.section-item {
  @apply border border-gray-200 rounded-lg overflow-hidden;
}

.section-item.active {
  @apply border-blue-300 shadow-sm;
}

.section-header {
  @apply flex items-center justify-between p-3 bg-gray-50 border-b border-gray-200;
}

.section-controls {
  @apply flex items-center gap-2 flex-1;
}

.section-toggle {
  @apply p-1 hover:bg-gray-200 rounded transition-transform;
}

.section-toggle.expanded {
  @apply transform rotate-90;
}

.section-heading-input {
  @apply flex-1 font-medium border-none bg-transparent outline-none;
  @apply focus:bg-white focus:border focus:border-blue-300 focus:rounded px-2 py-1;
}

.section-actions {
  @apply flex gap-1;
}

.action-btn {
  @apply p-1.5 hover:bg-gray-200 rounded disabled:opacity-50 disabled:cursor-not-allowed;
}

.section-content {
  @apply p-4 space-y-4;
}

.text-editor-section {
  @apply space-y-2;
}

.content-label {
  @apply block text-sm font-medium text-gray-700;
}

.text-editor {
  @apply w-full border border-gray-300 rounded-md p-3 resize-none;
  @apply focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
}

.formulas-section {
  @apply space-y-3;
}

.formulas-header {
  @apply flex items-center justify-between;
}

.formulas-list {
  @apply space-y-3;
}

.formula-item {
  @apply border border-gray-200 rounded-lg p-3 bg-gray-50;
}

.formula-header {
  @apply flex items-center justify-between mb-2;
}

.formula-index {
  @apply text-sm font-medium text-gray-700;
}

.formula-controls {
  @apply flex items-center gap-3;
}

.inline-checkbox {
  @apply flex items-center gap-1 text-sm;
}

.position-input {
  @apply w-16 text-sm border border-gray-300 rounded px-2 py-1;
}

.formula-editor-container {
  @apply grid grid-cols-1 lg:grid-cols-2 gap-3;
}

.formula-input {
  @apply w-full border border-gray-300 rounded-md p-2 font-mono text-sm resize-none;
  @apply focus:ring-2 focus:ring-blue-500 focus:border-blue-500;
}

.formula-preview {
  @apply border border-gray-200 rounded-md p-2 bg-white min-h-16 flex items-center justify-center;
}

.empty-formulas,
.empty-sections {
  @apply flex flex-col items-center justify-center gap-2 text-gray-500 py-8;
}

.empty-icon {
  @apply w-8 h-8;
}

.empty-text {
  @apply text-sm;
}

.preview-mode {
  @apply p-4;
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