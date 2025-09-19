import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FormulaResult, AppConfig, InputType } from '@/types'

export interface FormulaEditorState {
  originalLatex: string
  currentLatex: string
  isModified: boolean
  syntaxError: string | null
  isValidating: boolean
  isSaving: boolean
}

export interface UseFormulaEditorOptions {
  autoSave?: boolean
  autoValidate?: boolean
  debounceMs?: number
}

export function useFormulaEditor(
  initialResult?: FormulaResult,
  config?: AppConfig,
  options: UseFormulaEditorOptions = {}
) {
  const {
    autoSave = false,
    autoValidate = true,
    debounceMs = 500
  } = options

  // State
  const state = ref<FormulaEditorState>({
    originalLatex: initialResult?.latex || '',
    currentLatex: initialResult?.latex || '',
    isModified: false,
    syntaxError: null,
    isValidating: false,
    isSaving: false
  })

  const recognitionResult = ref<FormulaResult | null>(initialResult || null)
  const appConfig = ref<AppConfig | null>(config || null)

  // Computed properties
  const hasChanges = computed(() => {
    return state.value.currentLatex !== state.value.originalLatex
  })

  const canSave = computed(() => {
    return hasChanges.value && !state.value.syntaxError && !state.value.isSaving
  })

  const isValid = computed(() => {
    return !state.value.syntaxError && state.value.currentLatex.trim().length > 0
  })

  // Validation debouncing
  let validationTimeout: number | null = null

  // Watch for changes
  watch(() => state.value.currentLatex, (newValue) => {
    state.value.isModified = newValue !== state.value.originalLatex
    
    if (autoValidate) {
      debouncedValidate()
    }
    
    if (autoSave && hasChanges.value && isValid.value) {
      debouncedSave()
    }
  })

  // Methods
  function updateLatex(latex: string) {
    state.value.currentLatex = latex
  }

  function resetToOriginal() {
    state.value.currentLatex = state.value.originalLatex
    state.value.syntaxError = null
    state.value.isModified = false
  }

  function setOriginalLatex(latex: string) {
    state.value.originalLatex = latex
    state.value.currentLatex = latex
    state.value.isModified = false
  }

  async function validateSyntax(): Promise<boolean> {
    if (state.value.isValidating) return false

    state.value.isValidating = true
    
    try {
      // Basic client-side validation
      const clientError = validateLatexSyntax(state.value.currentLatex)
      if (clientError) {
        state.value.syntaxError = clientError
        return false
      }

      // Server-side validation could be added here
      // const serverValidation = await invoke('validate_latex_syntax', {
      //   latex: state.value.currentLatex
      // })

      state.value.syntaxError = null
      return true
    } catch (error) {
      state.value.syntaxError = error instanceof Error ? error.message : '验证失败'
      return false
    } finally {
      state.value.isValidating = false
    }
  }

  async function saveChanges(): Promise<boolean> {
    if (!canSave.value) return false

    state.value.isSaving = true

    try {
      // Validate before saving
      const isValidSyntax = await validateSyntax()
      if (!isValidSyntax) {
        throw new Error('LaTeX语法错误，无法保存')
      }

      // Update the recognition result
      if (recognitionResult.value) {
        recognitionResult.value.latex = state.value.currentLatex
        
        // Update the content based on input type
        if (recognitionResult.value.inputType === 'SingleFormula') {
          recognitionResult.value.content = { SingleFormula: state.value.currentLatex }
        } else if ('Document' in recognitionResult.value.content) {
          // Update the first formula in the document
          const doc = recognitionResult.value.content.Document
          if (doc.sections.length > 0 && doc.sections[0].formulas.length > 0) {
            doc.sections[0].formulas[0].latex = state.value.currentLatex
          }
        }
      }

      // Update original value
      state.value.originalLatex = state.value.currentLatex
      state.value.isModified = false

      return true
    } catch (error) {
      console.error('Failed to save changes:', error)
      throw error
    } finally {
      state.value.isSaving = false
    }
  }

  async function reRecognizeWithType(imageData: string, inputType: InputType): Promise<FormulaResult | null> {
    if (!appConfig.value) {
      throw new Error('应用配置未设置')
    }

    try {
      const result = await invoke<FormulaResult>('re_recognize_with_type', {
        base64Data: imageData,
        forcedType: inputType,
        config: appConfig.value
      })

      recognitionResult.value = result
      setOriginalLatex(result.latex)
      
      return result
    } catch (error) {
      console.error('Re-recognition failed:', error)
      throw error
    }
  }

  function debouncedValidate() {
    if (validationTimeout) {
      clearTimeout(validationTimeout)
    }
    
    validationTimeout = setTimeout(() => {
      validateSyntax()
    }, debounceMs)
  }

  function debouncedSave() {
    if (validationTimeout) {
      clearTimeout(validationTimeout)
    }
    
    validationTimeout = setTimeout(() => {
      if (canSave.value) {
        saveChanges()
      }
    }, debounceMs * 2) // Longer delay for auto-save
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
        'left', 'right', 'begin', 'end', 'text', 'mathbf', 'mathit', 'mathrm',
        'sin', 'cos', 'tan', 'log', 'ln', 'exp', 'max', 'min', 'sup', 'inf'
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

  // Formula manipulation utilities
  function insertAtPosition(position: number, text: string) {
    const current = state.value.currentLatex
    const newLatex = current.slice(0, position) + text + current.slice(position)
    updateLatex(newLatex)
    return position + text.length
  }

  function replaceRange(start: number, end: number, text: string) {
    const current = state.value.currentLatex
    const newLatex = current.slice(0, start) + text + current.slice(end)
    updateLatex(newLatex)
    return start + text.length
  }

  function wrapSelection(start: number, end: number, before: string, after: string = '') {
    const current = state.value.currentLatex
    const selectedText = current.slice(start, end)
    const wrappedText = before + selectedText + after
    const newLatex = current.slice(0, start) + wrappedText + current.slice(end)
    updateLatex(newLatex)
    return {
      start: start + before.length,
      end: start + before.length + selectedText.length
    }
  }

  // Common LaTeX insertions
  function insertFraction() {
    return '\\frac{}{}'
  }

  function insertSuperscript() {
    return '^{}'
  }

  function insertSubscript() {
    return '_{}'
  }

  function insertSquareRoot() {
    return '\\sqrt{}'
  }

  function insertIntegral() {
    return '\\int_{}'
  }

  function insertSum() {
    return '\\sum_{}'
  }

  // Cleanup
  function cleanup() {
    if (validationTimeout) {
      clearTimeout(validationTimeout)
    }
  }

  return {
    // State
    state: computed(() => state.value),
    recognitionResult: computed(() => recognitionResult.value),
    
    // Computed
    hasChanges,
    canSave,
    isValid,
    
    // Methods
    updateLatex,
    resetToOriginal,
    setOriginalLatex,
    validateSyntax,
    saveChanges,
    reRecognizeWithType,
    
    // Utilities
    insertAtPosition,
    replaceRange,
    wrapSelection,
    insertFraction,
    insertSuperscript,
    insertSubscript,
    insertSquareRoot,
    insertIntegral,
    insertSum,
    
    // Cleanup
    cleanup
  }
}

// Export types for external use