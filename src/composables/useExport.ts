import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { FormulaResult, AppConfig } from '@/types'
import { ExportFormat } from '@/types'

export interface ExportConfig {
  format: ExportFormat
  includeMetadata: boolean
  customTemplate?: string
  formatOptions: Record<string, string>
}

export interface ExportResult {
  content: string
  format: ExportFormat
  metadata: ExportMetadata
}

export interface ExportMetadata {
  timestamp: number
  originalInputType: string
  exportFormat: ExportFormat
  characterCount: number
  formulaCount: number
  processingTimeMs: number
}

export interface ExportState {
  availableFormats: string[]
  selectedFormat: ExportFormat | null
  isExporting: boolean
  lastExportResult: ExportResult | null
  exportHistory: ExportResult[]
}

export function useExport(formulaResult: FormulaResult, appConfig: AppConfig) {
  // State
  const state = ref<ExportState>({
    availableFormats: [],
    selectedFormat: null,
    isExporting: false,
    lastExportResult: null,
    exportHistory: []
  })

  const exportError = ref<string | null>(null)

  // Computed
  const hasResult = computed(() => !!formulaResult)
  
  const canExport = computed(() => 
    hasResult.value && 
    state.value.selectedFormat && 
    !state.value.isExporting
  )

  const exportStats = computed(() => ({
    totalExports: state.value.exportHistory.length,
    lastExportTime: state.value.lastExportResult?.metadata.timestamp,
    averageProcessingTime: state.value.exportHistory.length > 0 
      ? state.value.exportHistory.reduce((sum, result) => sum + result.metadata.processingTimeMs, 0) / state.value.exportHistory.length
      : 0
  }))

  // Methods
  async function loadAvailableFormats(): Promise<void> {
    try {
      const formats = await invoke<string[]>('get_available_export_formats', {
        inputType: formulaResult.inputType,
        appConfig
      })
      
      state.value.availableFormats = formats
      
      // Set default format if none selected
      if (!state.value.selectedFormat && formats.length > 0) {
        const defaultFormat = await invoke<string>('get_default_export_format', {
          inputType: formulaResult.inputType,
          appConfig
        })
        state.value.selectedFormat = defaultFormat as ExportFormat
      }
    } catch (error) {
      exportError.value = error instanceof Error ? error.message : '加载导出格式失败'
      throw error
    }
  }

  function setSelectedFormat(format: ExportFormat) {
    state.value.selectedFormat = format
    exportError.value = null
  }

  async function exportToString(config: ExportConfig): Promise<ExportResult> {
    if (!hasResult.value) {
      throw new Error('没有可导出的内容')
    }

    state.value.isExporting = true
    exportError.value = null

    try {
      const result = await invoke<ExportResult>('export_formula_result', {
        result: formulaResult,
        exportConfig: config,
        appConfig
      })

      state.value.lastExportResult = result
      state.value.exportHistory.push(result)

      return result
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '导出失败'
      exportError.value = errorMessage
      throw new Error(errorMessage)
    } finally {
      state.value.isExporting = false
    }
  }

  async function exportToFile(config: ExportConfig, filePath?: string): Promise<string> {
    if (!hasResult.value) {
      throw new Error('没有可导出的内容')
    }

    state.value.isExporting = true
    exportError.value = null

    try {
      let targetPath = filePath

      if (!targetPath) {
        // Show file save dialog
        const extensions = getFileExtensions(config.format)
        const defaultPath = `formula${extensions[0]}`

        const savedPath = await save({
          defaultPath,
          filters: [{
            name: getFormatDisplayName(config.format),
            extensions: extensions.map(ext => ext.substring(1)) // Remove the dot
          }]
        })
        
        targetPath = savedPath || undefined

        if (!targetPath) {
          throw new Error('用户取消了文件保存')
        }
      }

      await invoke('export_to_file', {
        result: formulaResult,
        exportConfig: config,
        appConfig,
        filePath: targetPath
      })

      // Also update the export result for history
      await exportToString(config)
      
      return targetPath
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '导出到文件失败'
      exportError.value = errorMessage
      throw new Error(errorMessage)
    } finally {
      state.value.isExporting = false
    }
  }

  async function exportToClipboard(config: ExportConfig): Promise<void> {
    try {
      const result = await exportToString(config)
      await navigator.clipboard.writeText(result.content)
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '复制到剪贴板失败'
      exportError.value = errorMessage
      throw new Error(errorMessage)
    }
  }

  async function quickExport(format?: ExportFormat): Promise<ExportResult> {
    const targetFormat = format || state.value.selectedFormat
    if (!targetFormat) {
      throw new Error('未选择导出格式')
    }

    const config: ExportConfig = {
      format: targetFormat,
      includeMetadata: true,
      formatOptions: {}
    }

    return exportToString(config)
  }

  async function quickExportToFile(format?: ExportFormat): Promise<string> {
    const targetFormat = format || state.value.selectedFormat
    if (!targetFormat) {
      throw new Error('未选择导出格式')
    }

    const config: ExportConfig = {
      format: targetFormat,
      includeMetadata: true,
      formatOptions: {}
    }

    return exportToFile(config)
  }

  async function quickExportToClipboard(format?: ExportFormat): Promise<void> {
    const targetFormat = format || state.value.selectedFormat
    if (!targetFormat) {
      throw new Error('未选择导出格式')
    }

    const config: ExportConfig = {
      format: targetFormat,
      includeMetadata: false, // Usually don't want metadata in clipboard
      formatOptions: {}
    }

    return exportToClipboard(config)
  }

  function getFileExtensions(format: ExportFormat): string[] {
    const extensionMap: Record<ExportFormat, string[]> = {
      [ExportFormat.LaTeX]: ['.tex'],
      [ExportFormat.LaTeXInline]: ['.tex'],
      [ExportFormat.LaTeXBlock]: ['.tex'],
      [ExportFormat.Markdown]: ['.md'],
      [ExportFormat.MarkdownInline]: ['.md'],
      [ExportFormat.MarkdownBlock]: ['.md'],
      [ExportFormat.HTML]: ['.html'],
      [ExportFormat.DOCX]: ['.docx'],
      [ExportFormat.PlainText]: ['.txt']
    }

    return extensionMap[format] || ['.txt']
  }

  function getFormatDisplayName(format: ExportFormat): string {
    const nameMap: Record<ExportFormat, string> = {
      [ExportFormat.LaTeX]: 'LaTeX',
      [ExportFormat.LaTeXInline]: 'LaTeX (Inline)',
      [ExportFormat.LaTeXBlock]: 'LaTeX (Block)',
      [ExportFormat.Markdown]: 'Markdown',
      [ExportFormat.MarkdownInline]: 'Markdown (Inline)',
      [ExportFormat.MarkdownBlock]: 'Markdown (Block)',
      [ExportFormat.HTML]: 'HTML',
      [ExportFormat.DOCX]: 'Word Document',
      [ExportFormat.PlainText]: 'Plain Text'
    }

    return nameMap[format] || format
  }

  function clearHistory() {
    state.value.exportHistory = []
    state.value.lastExportResult = null
  }

  function removeFromHistory(index: number) {
    if (index >= 0 && index < state.value.exportHistory.length) {
      state.value.exportHistory.splice(index, 1)
    }
  }

  function getExportPreview(config: ExportConfig, maxLength: number = 500): Promise<string> {
    return exportToString(config).then(result => {
      const content = result.content
      if (content.length <= maxLength) {
        return content
      }
      return content.substring(0, maxLength) + '...'
    })
  }

  // Validation
  function validateConfig(config: ExportConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = []

    if (!config.format) {
      errors.push('必须选择导出格式')
    }

    if (!state.value.availableFormats.includes(config.format)) {
      errors.push('选择的导出格式不可用')
    }

    if (config.customTemplate && config.customTemplate.trim().length === 0) {
      errors.push('自定义模板不能为空')
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }

  // Format-specific helpers
  function createMarkdownConfig(inlineFormat: 'Dollar' | 'Parentheses' = 'Dollar', blockFormat: 'DoubleDollar' | 'Brackets' = 'DoubleDollar'): ExportConfig {
    return {
      format: ExportFormat.Markdown,
      includeMetadata: true,
      formatOptions: {
        inlineFormat,
        blockFormat
      }
    }
  }

  function createHtmlConfig(includeMathJax: boolean = true, inlineCSS: boolean = false): ExportConfig {
    return {
      format: ExportFormat.HTML,
      includeMetadata: true,
      formatOptions: {
        includeMathJax: includeMathJax.toString(),
        inlineCSS: inlineCSS.toString()
      }
    }
  }

  function createLatexConfig(includeDocumentClass: boolean = true): ExportConfig {
    return {
      format: ExportFormat.LaTeX,
      includeMetadata: true,
      formatOptions: {
        includeDocumentClass: includeDocumentClass.toString()
      }
    }
  }

  return {
    // State
    state: computed(() => state.value),
    exportError: computed(() => exportError.value),
    
    // Computed
    hasResult,
    canExport,
    exportStats,
    
    // Methods
    loadAvailableFormats,
    setSelectedFormat,
    exportToString,
    exportToFile,
    exportToClipboard,
    quickExport,
    quickExportToFile,
    quickExportToClipboard,
    getExportPreview,
    
    // Utilities
    getFileExtensions,
    getFormatDisplayName,
    validateConfig,
    
    // History management
    clearHistory,
    removeFromHistory,
    
    // Format-specific helpers
    createMarkdownConfig,
    createHtmlConfig,
    createLatexConfig
  }
}

// Export types