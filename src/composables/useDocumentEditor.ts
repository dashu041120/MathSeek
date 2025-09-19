import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DocumentContent, DocumentSection, FormulaBlock, AppConfig, FormulaResult } from '@/types'
import { createDocumentContent, createDocumentSection, createFormulaBlock } from '@/types'

export interface DocumentEditorState {
  document: DocumentContent
  originalDocument: DocumentContent
  isModified: boolean
  isSaving: boolean
  isExporting: boolean
  activeSection: number
  validationErrors: string[]
}

export interface UseDocumentEditorOptions {
  autoSave?: boolean
  autoValidate?: boolean
  debounceMs?: number
}

export function useDocumentEditor(
  initialDocument?: DocumentContent,
  config?: AppConfig,
  options: UseDocumentEditorOptions = {}
) {
  const {
    autoSave = false,
    autoValidate = true,
    debounceMs = 1000
  } = options

  // State
  const state = ref<DocumentEditorState>({
    document: initialDocument ? JSON.parse(JSON.stringify(initialDocument)) : createDocumentContent(),
    originalDocument: initialDocument ? JSON.parse(JSON.stringify(initialDocument)) : createDocumentContent(),
    isModified: false,
    isSaving: false,
    isExporting: false,
    activeSection: 0,
    validationErrors: []
  })

  const appConfig = ref<AppConfig | null>(config || null)

  // Computed properties
  const hasChanges = computed(() => {
    return JSON.stringify(state.value.document) !== JSON.stringify(state.value.originalDocument)
  })

  const canSave = computed(() => {
    return hasChanges.value && 
           state.value.validationErrors.length === 0 && 
           !state.value.isSaving
  })

  const isValid = computed(() => {
    return state.value.validationErrors.length === 0 && 
           state.value.document.sections.length > 0
  })

  const totalSections = computed(() => state.value.document.sections.length)

  const totalFormulas = computed(() => {
    return state.value.document.sections.reduce((total, section) => {
      return total + section.formulas.length
    }, 0)
  })

  const totalCharacters = computed(() => {
    let count = state.value.document.title?.length || 0
    
    state.value.document.sections.forEach(section => {
      count += (section.heading?.length || 0)
      count += section.text.length
      section.formulas.forEach(formula => {
        count += formula.latex.length
      })
    })
    
    return count
  })

  // Debouncing
  let saveTimeout: number | null = null
  let validationTimeout: number | null = null

  // Watch for changes
  watch(() => state.value.document, () => {
    state.value.isModified = hasChanges.value
    
    if (autoValidate) {
      debouncedValidate()
    }
    
    if (autoSave && canSave.value) {
      debouncedSave()
    }
  }, { deep: true })

  // Document manipulation methods
  function updateDocument(document: DocumentContent) {
    state.value.document = JSON.parse(JSON.stringify(document))
  }

  function setOriginalDocument(document: DocumentContent) {
    state.value.originalDocument = JSON.parse(JSON.stringify(document))
    state.value.document = JSON.parse(JSON.stringify(document))
    state.value.isModified = false
  }

  function resetToOriginal() {
    state.value.document = JSON.parse(JSON.stringify(state.value.originalDocument))
    state.value.isModified = false
    state.value.validationErrors = []
  }

  // Section management
  function addSection(title?: string, position?: number): number {
    const newSection = createDocumentSection(title || `章节 ${state.value.document.sections.length + 1}`)
    
    if (position !== undefined && position >= 0 && position <= state.value.document.sections.length) {
      state.value.document.sections.splice(position, 0, newSection)
      return position
    } else {
      state.value.document.sections.push(newSection)
      return state.value.document.sections.length - 1
    }
  }

  function removeSection(index: number): boolean {
    if (index < 0 || index >= state.value.document.sections.length) return false
    if (state.value.document.sections.length <= 1) return false
    
    state.value.document.sections.splice(index, 1)
    
    // Adjust active section
    if (state.value.activeSection >= state.value.document.sections.length) {
      state.value.activeSection = state.value.document.sections.length - 1
    }
    
    return true
  }

  function moveSection(fromIndex: number, toIndex: number): boolean {
    if (fromIndex < 0 || fromIndex >= state.value.document.sections.length) return false
    if (toIndex < 0 || toIndex >= state.value.document.sections.length) return false
    if (fromIndex === toIndex) return false
    
    const section = state.value.document.sections.splice(fromIndex, 1)[0]
    state.value.document.sections.splice(toIndex, 0, section)
    
    // Update active section
    if (state.value.activeSection === fromIndex) {
      state.value.activeSection = toIndex
    } else if (state.value.activeSection === toIndex) {
      state.value.activeSection = fromIndex
    }
    
    return true
  }

  function duplicateSection(index: number): number {
    if (index < 0 || index >= state.value.document.sections.length) return -1
    
    const originalSection = state.value.document.sections[index]
    const duplicatedSection: DocumentSection = {
      heading: (originalSection.heading || '') + ' (副本)',
      text: originalSection.text,
      formulas: originalSection.formulas.map(formula => ({
        latex: formula.latex,
        position: formula.position,
        isInline: formula.isInline
      }))
    }
    
    state.value.document.sections.splice(index + 1, 0, duplicatedSection)
    return index + 1
  }

  function updateSection(index: number, updates: Partial<DocumentSection>): boolean {
    if (index < 0 || index >= state.value.document.sections.length) return false
    
    const section = state.value.document.sections[index]
    Object.assign(section, updates)
    
    return true
  }

  // Formula management
  function addFormula(sectionIndex: number, latex: string = '', position?: number, isInline: boolean = false): number {
    if (sectionIndex < 0 || sectionIndex >= state.value.document.sections.length) return -1
    
    const section = state.value.document.sections[sectionIndex]
    const formulaPosition = position !== undefined ? position : section.text.length
    const newFormula = createFormulaBlock(latex, formulaPosition, isInline)
    
    section.formulas.push(newFormula)
    return section.formulas.length - 1
  }

  function removeFormula(sectionIndex: number, formulaIndex: number): boolean {
    if (sectionIndex < 0 || sectionIndex >= state.value.document.sections.length) return false
    
    const section = state.value.document.sections[sectionIndex]
    if (formulaIndex < 0 || formulaIndex >= section.formulas.length) return false
    
    section.formulas.splice(formulaIndex, 1)
    return true
  }

  function updateFormula(sectionIndex: number, formulaIndex: number, updates: Partial<FormulaBlock>): boolean {
    if (sectionIndex < 0 || sectionIndex >= state.value.document.sections.length) return false
    
    const section = state.value.document.sections[sectionIndex]
    if (formulaIndex < 0 || formulaIndex >= section.formulas.length) return false
    
    const formula = section.formulas[formulaIndex]
    Object.assign(formula, updates)
    
    return true
  }

  // Validation
  function validateDocument(): string[] {
    const errors: string[] = []
    
    if (state.value.document.sections.length === 0) {
      errors.push('文档必须包含至少一个章节')
    }
    
    state.value.document.sections.forEach((section, sectionIndex) => {
      if (!section.text.trim() && section.formulas.length === 0) {
        errors.push(`章节 ${sectionIndex + 1} 必须包含文本内容或公式`)
      }
      
      section.formulas.forEach((formula, formulaIndex) => {
        if (formula.position < 0 || formula.position > section.text.length) {
          errors.push(`章节 ${sectionIndex + 1} 的公式 ${formulaIndex + 1} 位置超出文本范围`)
        }
        
        if (!formula.latex.trim()) {
          errors.push(`章节 ${sectionIndex + 1} 的公式 ${formulaIndex + 1} 内容不能为空`)
        }
        
        // Basic LaTeX validation
        const latexError = validateLatexSyntax(formula.latex)
        if (latexError) {
          errors.push(`章节 ${sectionIndex + 1} 的公式 ${formulaIndex + 1}: ${latexError}`)
        }
      })
    })
    
    state.value.validationErrors = errors
    return errors
  }

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

    return null
  }

  // Save and export
  async function saveDocument(): Promise<boolean> {
    if (!canSave.value) return false

    state.value.isSaving = true

    try {
      const errors = validateDocument()
      if (errors.length > 0) {
        throw new Error(`文档验证失败: ${errors.join(', ')}`)
      }

      // Update original document
      state.value.originalDocument = JSON.parse(JSON.stringify(state.value.document))
      state.value.isModified = false

      return true
    } catch (error) {
      console.error('Failed to save document:', error)
      throw error
    } finally {
      state.value.isSaving = false
    }
  }

  async function exportDocument(format: string = 'markdown'): Promise<string> {
    if (!appConfig.value) {
      throw new Error('应用配置未设置')
    }

    state.value.isExporting = true

    try {
      const errors = validateDocument()
      if (errors.length > 0) {
        throw new Error(`文档验证失败，无法导出: ${errors.join(', ')}`)
      }

      // This would call the Tauri backend to export the document
      // For now, we'll return a placeholder
      const exportedContent = await invoke<string>('export_document', {
        document: state.value.document,
        format,
        config: appConfig.value
      })

      return exportedContent
    } catch (error) {
      console.error('Failed to export document:', error)
      throw error
    } finally {
      state.value.isExporting = false
    }
  }

  // Import from recognition result
  function importFromRecognitionResult(result: FormulaResult): boolean {
    try {
      if ('Document' in result.content) {
        setOriginalDocument(result.content.Document)
        return true
      } else if ('SingleFormula' in result.content) {
        // Convert single formula to document
        const document = createDocumentContent('识别结果')
        const section = createDocumentSection('公式', '')
        section.formulas.push(createFormulaBlock(result.content.SingleFormula, 0, false))
        document.sections.push(section)
        
        setOriginalDocument(document)
        return true
      }
      
      return false
    } catch (error) {
      console.error('Failed to import recognition result:', error)
      return false
    }
  }

  // Debounced operations
  function debouncedSave() {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }
    
    saveTimeout = setTimeout(() => {
      if (canSave.value) {
        saveDocument()
      }
    }, debounceMs * 2)
  }

  function debouncedValidate() {
    if (validationTimeout) {
      clearTimeout(validationTimeout)
    }
    
    validationTimeout = setTimeout(() => {
      validateDocument()
    }, debounceMs)
  }

  // Navigation
  function setActiveSection(index: number) {
    if (index >= 0 && index < state.value.document.sections.length) {
      state.value.activeSection = index
    }
  }

  function nextSection(): boolean {
    if (state.value.activeSection < state.value.document.sections.length - 1) {
      state.value.activeSection++
      return true
    }
    return false
  }

  function previousSection(): boolean {
    if (state.value.activeSection > 0) {
      state.value.activeSection--
      return true
    }
    return false
  }

  // Cleanup
  function cleanup() {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }
    if (validationTimeout) {
      clearTimeout(validationTimeout)
    }
  }

  // Initialize with default section if empty
  if (state.value.document.sections.length === 0) {
    addSection('默认章节')
  }

  return {
    // State
    state: computed(() => state.value),
    
    // Computed
    hasChanges,
    canSave,
    isValid,
    totalSections,
    totalFormulas,
    totalCharacters,
    
    // Document methods
    updateDocument,
    setOriginalDocument,
    resetToOriginal,
    
    // Section methods
    addSection,
    removeSection,
    moveSection,
    duplicateSection,
    updateSection,
    
    // Formula methods
    addFormula,
    removeFormula,
    updateFormula,
    
    // Validation
    validateDocument,
    
    // Save and export
    saveDocument,
    exportDocument,
    
    // Import
    importFromRecognitionResult,
    
    // Navigation
    setActiveSection,
    nextSection,
    previousSection,
    
    // Cleanup
    cleanup
  }
}

// Export types