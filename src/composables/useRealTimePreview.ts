import { ref, computed, watch, nextTick } from 'vue'
import mathRenderer, { type RenderEngine, type RenderResult } from '@/services/mathRenderer'

export interface PreviewState {
  latex: string
  renderedHtml: string
  isRendering: boolean
  renderError: string | null
  lastRenderTime: number | null
  renderEngine: RenderEngine
  displayMode: 'inline' | 'block'
  autoSync: boolean
  syncCount: number
}

export interface PreviewOptions {
  debounceMs?: number
  autoSync?: boolean
  defaultEngine?: RenderEngine
  displayMode?: 'inline' | 'block'
}

export interface SyncResult {
  success: boolean
  html?: string
  error?: string
  renderTime?: number
}

export function useRealTimePreview(
  initialLatex: string = '',
  options: PreviewOptions = {}
) {
  const {
    debounceMs = 300,
    autoSync = true,
    defaultEngine = 'mathjax',
    displayMode = 'block'
  } = options

  // State
  const state = ref<PreviewState>({
    latex: initialLatex,
    renderedHtml: '',
    isRendering: false,
    renderError: null,
    lastRenderTime: null,
    renderEngine: defaultEngine,
    displayMode,
    autoSync,
    syncCount: 0
  })

  // Debouncing
  let renderTimeout: number | null = null
  let renderPromise: Promise<SyncResult> | null = null

  // Computed properties
  const hasContent = computed(() => state.value.latex.trim().length > 0)
  
  const hasError = computed(() => state.value.renderError !== null)
  
  const isReady = computed(() => 
    !state.value.isRendering && 
    !hasError.value && 
    hasContent.value
  )

  const renderStats = computed(() => ({
    characterCount: state.value.latex.length,
    lineCount: state.value.latex.split('\n').length,
    renderTime: state.value.lastRenderTime,
    syncCount: state.value.syncCount,
    engine: state.value.renderEngine
  }))

  // Watch for auto-sync
  watch(() => state.value.latex, () => {
    if (state.value.autoSync) {
      debouncedRender()
    }
  })

  watch(() => state.value.renderEngine, () => {
    if (state.value.autoSync && hasContent.value) {
      debouncedRender()
    }
  })

  watch(() => state.value.displayMode, () => {
    if (state.value.autoSync && hasContent.value) {
      debouncedRender()
    }
  })

  // Methods
  function updateLatex(latex: string) {
    state.value.latex = latex
  }

  function setRenderEngine(engine: RenderEngine) {
    state.value.renderEngine = engine
  }

  function setDisplayMode(mode: 'inline' | 'block') {
    state.value.displayMode = mode
  }

  function toggleAutoSync() {
    state.value.autoSync = !state.value.autoSync
    if (state.value.autoSync && hasContent.value) {
      render()
    }
  }

  function setAutoSync(enabled: boolean) {
    state.value.autoSync = enabled
    if (enabled && hasContent.value) {
      render()
    }
  }

  async function render(): Promise<SyncResult> {
    // Cancel any pending render
    if (renderTimeout) {
      clearTimeout(renderTimeout)
      renderTimeout = null
    }

    // Return existing promise if already rendering
    if (renderPromise) {
      return renderPromise
    }

    if (!hasContent.value) {
      state.value.renderedHtml = ''
      state.value.renderError = null
      return { success: true, html: '' }
    }

    state.value.isRendering = true
    state.value.renderError = null

    renderPromise = performRender()
    
    try {
      const result = await renderPromise
      return result
    } finally {
      renderPromise = null
      state.value.isRendering = false
    }
  }

  async function performRender(): Promise<SyncResult> {
    const startTime = performance.now()

    try {
      const result: RenderResult = await mathRenderer.render(
        state.value.latex,
        state.value.renderEngine,
        {
          displayMode: state.value.displayMode === 'block',
          throwOnError: false
        }
      )

      const renderTime = Math.round(performance.now() - startTime)

      if (result.success && result.html) {
        state.value.renderedHtml = result.html
        state.value.renderError = null
        state.value.lastRenderTime = renderTime
        state.value.syncCount++

        return {
          success: true,
          html: result.html,
          renderTime
        }
      } else {
        const error = result.error || '渲染失败'
        state.value.renderError = error
        state.value.lastRenderTime = renderTime

        return {
          success: false,
          error,
          renderTime
        }
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知渲染错误'
      const renderTime = Math.round(performance.now() - startTime)
      
      state.value.renderError = errorMessage
      state.value.lastRenderTime = renderTime

      return {
        success: false,
        error: errorMessage,
        renderTime
      }
    }
  }

  function debouncedRender(): Promise<SyncResult> {
    return new Promise((resolve) => {
      if (renderTimeout) {
        clearTimeout(renderTimeout)
      }

      renderTimeout = setTimeout(async () => {
        const result = await render()
        resolve(result)
      }, debounceMs)
    })
  }

  async function forceRender(): Promise<SyncResult> {
    // Cancel any pending debounced render
    if (renderTimeout) {
      clearTimeout(renderTimeout)
      renderTimeout = null
    }

    return render()
  }

  function clearPreview() {
    state.value.latex = ''
    state.value.renderedHtml = ''
    state.value.renderError = null
    state.value.lastRenderTime = null
  }

  function resetState() {
    clearPreview()
    state.value.syncCount = 0
    state.value.renderEngine = defaultEngine
    state.value.displayMode = displayMode
    state.value.autoSync = autoSync
  }

  // LaTeX manipulation utilities
  function insertAtPosition(position: number, text: string): number {
    const current = state.value.latex
    const newLatex = current.slice(0, position) + text + current.slice(position)
    updateLatex(newLatex)
    return position + text.length
  }

  function replaceRange(start: number, end: number, text: string): number {
    const current = state.value.latex
    const newLatex = current.slice(0, start) + text + current.slice(end)
    updateLatex(newLatex)
    return start + text.length
  }

  function wrapSelection(start: number, end: number, before: string, after: string = '') {
    const current = state.value.latex
    const selectedText = current.slice(start, end)
    const wrappedText = before + selectedText + after
    const newLatex = current.slice(0, start) + wrappedText + current.slice(end)
    updateLatex(newLatex)
    return {
      start: start + before.length,
      end: start + before.length + selectedText.length
    }
  }

  // Validation
  function validateLatex(): { valid: boolean; error?: string } {
    return mathRenderer.validateLatex(state.value.latex)
  }

  // Export functionality
  function exportAsHtml(): string {
    if (!state.value.renderedHtml) {
      throw new Error('No rendered content to export')
    }

    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Mathematical Formula</title>
  <style>
    body { font-family: serif; margin: 2rem; }
    .formula-container { text-align: center; margin: 2rem 0; }
  </style>
</head>
<body>
  <div class="formula-container">
    ${state.value.renderedHtml}
  </div>
</body>
</html>`
  }

  function exportAsLatex(): string {
    return state.value.latex
  }

  function exportAsSvg(): string {
    // This would require additional implementation to extract SVG from rendered content
    throw new Error('SVG export not yet implemented')
  }

  // Cleanup
  function cleanup() {
    if (renderTimeout) {
      clearTimeout(renderTimeout)
    }
    renderPromise = null
  }

  // Initialize
  if (state.value.autoSync && hasContent.value) {
    nextTick(() => {
      render()
    })
  }

  return {
    // State
    state: computed(() => state.value),
    
    // Computed
    hasContent,
    hasError,
    isReady,
    renderStats,
    
    // Methods
    updateLatex,
    setRenderEngine,
    setDisplayMode,
    toggleAutoSync,
    setAutoSync,
    render,
    debouncedRender,
    forceRender,
    clearPreview,
    resetState,
    
    // Utilities
    insertAtPosition,
    replaceRange,
    wrapSelection,
    validateLatex,
    
    // Export
    exportAsHtml,
    exportAsLatex,
    exportAsSvg,
    
    // Cleanup
    cleanup
  }
}

// Export types