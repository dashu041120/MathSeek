<template>
  <div class="document-preview">
    <!-- Document Title -->
    <div v-if="document.title" class="document-title">
      {{ document.title }}
    </div>

    <!-- Document Sections -->
    <div class="document-content">
      <div
        v-for="(section, index) in document.sections"
        :key="`section-${index}`"
        class="section"
      >
        <!-- Section Heading -->
        <h2 v-if="section.heading" class="section-heading">
          {{ section.heading }}
        </h2>

        <!-- Section Content with Embedded Formulas -->
        <div class="section-content">
          <div
            v-if="renderFormulas"
            class="content-with-formulas"
            v-html="renderSectionContent(section)"
          />
          <div v-else class="content-plain">
            <p v-if="section.text" class="section-text">{{ section.text }}</p>
            
            <!-- Formula List -->
            <div v-if="section.formulas.length > 0" class="formulas-list">
              <h4 class="formulas-title">公式列表:</h4>
              <div
                v-for="(formula, formulaIndex) in section.formulas"
                :key="`formula-${index}-${formulaIndex}`"
                class="formula-item"
              >
                <div class="formula-label">
                  公式 {{ formulaIndex + 1 }} ({{ formula.isInline ? '行内' : '块级' }})
                </div>
                <div class="formula-content">
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
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="document.sections.length === 0" class="empty-document">
      <svg class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
      </svg>
      <span class="empty-text">文档为空</span>
    </div>

    <!-- Document Statistics -->
    <div v-if="showStats" class="document-stats">
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-label">章节数</span>
          <span class="stat-value">{{ document.sections.length }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">公式数</span>
          <span class="stat-value">{{ totalFormulas }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">字符数</span>
          <span class="stat-value">{{ totalCharacters }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">预计阅读时间</span>
          <span class="stat-value">{{ estimatedReadingTime }}分钟</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FormulaRenderer from './FormulaRenderer.vue'
import type { DocumentContent, DocumentSection } from '@/types'

interface Props {
  document: DocumentContent
  renderFormulas?: boolean
  showStats?: boolean
  theme?: 'light' | 'dark' | 'academic'
}

const props = withDefaults(defineProps<Props>(), {
  renderFormulas: true,
  showStats: false,
  theme: 'light'
})

// Computed properties
const totalFormulas = computed(() => {
  return props.document.sections.reduce((total, section) => {
    return total + section.formulas.length
  }, 0)
})

const totalCharacters = computed(() => {
  let count = props.document.title?.length || 0
  
  props.document.sections.forEach(section => {
    count += (section.heading?.length || 0)
    count += section.text.length
    section.formulas.forEach(formula => {
      count += formula.latex.length
    })
  })
  
  return count
})

const estimatedReadingTime = computed(() => {
  // Assume 200 words per minute reading speed
  // Average 5 characters per word
  const words = totalCharacters.value / 5
  const minutes = Math.ceil(words / 200)
  return Math.max(1, minutes)
})

// Methods
function renderSectionContent(section: DocumentSection): string {
  if (!section.text && section.formulas.length === 0) {
    return '<p class="empty-section">此章节暂无内容</p>'
  }

  let content = section.text || ''
  
  // Sort formulas by position (descending to avoid position shifts)
  const sortedFormulas = [...section.formulas].sort((a, b) => b.position - a.position)
  
  // Insert formulas at their specified positions
  for (const formula of sortedFormulas) {
    const position = Math.min(formula.position, content.length)
    
    const formulaHtml = renderFormulaHtml(formula.latex, formula.isInline)
    
    content = content.slice(0, position) + formulaHtml + content.slice(position)
  }
  
  // Convert line breaks to paragraphs
  const paragraphs = content.split('\n\n').filter(p => p.trim())
  
  return paragraphs.map(paragraph => {
    const trimmed = paragraph.trim()
    if (trimmed.includes('<span class="embedded-formula')) {
      // Paragraph contains formulas, preserve HTML
      return `<p>${trimmed}</p>`
    } else {
      // Plain text paragraph
      return `<p>${escapeHtml(trimmed)}</p>`
    }
  }).join('')
}

function renderFormulaHtml(latex: string, isInline: boolean): string {
  // This is a simplified version - in a real app, you'd use MathJax/KaTeX
  const displayClass = isInline ? 'inline-formula' : 'block-formula'
  const processedLatex = escapeHtml(latex)
  
  return `<span class="embedded-formula ${displayClass}" data-latex="${processedLatex}">${processedLatex}</span>`
}

function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}
</script>

<style scoped>
.document-preview {
  @apply max-w-4xl mx-auto bg-white;
}

.document-title {
  @apply text-3xl font-bold text-gray-900 mb-8 text-center border-b border-gray-200 pb-4;
}

.document-content {
  @apply space-y-8;
}

.section {
  @apply space-y-4;
}

.section-heading {
  @apply text-2xl font-semibold text-gray-800 border-l-4 border-blue-500 pl-4;
}

.section-content {
  @apply prose prose-lg max-w-none;
}

.content-with-formulas {
  @apply leading-relaxed;
}

.content-plain {
  @apply space-y-4;
}

.section-text {
  @apply text-gray-700 leading-relaxed whitespace-pre-wrap;
}

.formulas-list {
  @apply space-y-3 mt-6;
}

.formulas-title {
  @apply text-lg font-medium text-gray-800 mb-3;
}

.formula-item {
  @apply border-l-2 border-gray-300 pl-4 space-y-2;
}

.formula-label {
  @apply text-sm font-medium text-gray-600;
}

.formula-content {
  @apply bg-gray-50 rounded-lg p-3;
}

.empty-document {
  @apply flex flex-col items-center justify-center gap-3 text-gray-500 py-16;
}

.empty-icon {
  @apply w-12 h-12;
}

.empty-text {
  @apply text-lg;
}

.document-stats {
  @apply mt-12 pt-8 border-t border-gray-200;
}

.stats-grid {
  @apply grid grid-cols-2 md:grid-cols-4 gap-4;
}

.stat-item {
  @apply text-center p-4 bg-gray-50 rounded-lg;
}

.stat-label {
  @apply block text-sm text-gray-600 mb-1;
}

.stat-value {
  @apply text-2xl font-bold text-gray-900;
}

/* Embedded formula styles */
:deep(.embedded-formula) {
  @apply font-serif;
}

:deep(.embedded-formula.inline-formula) {
  @apply mx-1;
}

:deep(.embedded-formula.block-formula) {
  @apply block text-center my-4 text-lg;
}

:deep(.empty-section) {
  @apply text-gray-500 italic;
}

/* Prose styles for rendered content */
:deep(.content-with-formulas p) {
  @apply mb-4 text-gray-700 leading-relaxed;
}

:deep(.content-with-formulas p:last-child) {
  @apply mb-0;
}

/* Theme variations */
.document-preview.theme-dark {
  @apply bg-gray-900 text-white;
}

.document-preview.theme-dark .document-title {
  @apply text-white border-gray-700;
}

.document-preview.theme-dark .section-heading {
  @apply text-gray-100 border-blue-400;
}

.document-preview.theme-dark .section-text {
  @apply text-gray-300;
}

.document-preview.theme-academic {
  @apply font-serif;
}

.document-preview.theme-academic .document-title {
  @apply font-serif;
}

.document-preview.theme-academic .section-heading {
  @apply font-serif;
}
</style>