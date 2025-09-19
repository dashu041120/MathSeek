import { ref } from 'vue'

// MathJax configuration
declare global {
  interface Window {
    MathJax: any
  }
}

// KaTeX types
interface KaTeXOptions {
  displayMode?: boolean
  throwOnError?: boolean
  errorColor?: string
  macros?: Record<string, string>
  strict?: boolean | string | Function
  trust?: boolean | Function
  maxSize?: number
  maxExpand?: number
  fleqn?: boolean
  leqno?: boolean
}

export type RenderEngine = 'mathjax' | 'katex'

export interface RenderOptions {
  displayMode?: boolean
  throwOnError?: boolean
  errorColor?: string
  macros?: Record<string, string>
}

export interface RenderResult {
  success: boolean
  html?: string
  error?: string
  renderTime?: number
}

class MathRenderer {
  private mathJaxLoaded = ref(false)
  private katexLoaded = ref(false)
  private mathJaxLoading = false
  private katexLoading = false
  private defaultEngine: RenderEngine = 'mathjax'

  constructor() {
    this.initializeEngines()
  }

  private async initializeEngines() {
    // Initialize both engines in parallel
    await Promise.all([
      this.loadMathJax(),
      this.loadKaTeX()
    ])
  }

  private async loadMathJax(): Promise<void> {
    if (this.mathJaxLoaded.value || this.mathJaxLoading) return
    
    this.mathJaxLoading = true

    try {
      // Configure MathJax before loading
      window.MathJax = {
        tex: {
          inlineMath: [['$', '$'], ['\\(', '\\)']],
          displayMath: [['$$', '$$'], ['\\[', '\\]']],
          processEscapes: true,
          processEnvironments: true,
          packages: ['base', 'ams', 'noerrors', 'noundefined', 'autoload']
        },
        svg: {
          fontCache: 'global',
          displayAlign: 'center',
          displayIndent: '0'
        },
        options: {
          skipHtmlTags: ['script', 'noscript', 'style', 'textarea', 'pre', 'code'],
          ignoreHtmlClass: 'tex2jax_ignore',
          processHtmlClass: 'tex2jax_process'
        },
        startup: {
          ready: () => {
            window.MathJax.startup.defaultReady()
            this.mathJaxLoaded.value = true
          }
        }
      }

      // Load MathJax script
      const script = document.createElement('script')
      script.src = 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-svg.js'
      script.async = true
      
      script.onload = () => {
        console.log('MathJax loaded successfully')
      }
      
      script.onerror = () => {
        console.error('Failed to load MathJax')
        this.mathJaxLoading = false
      }

      document.head.appendChild(script)
    } catch (error) {
      console.error('Error loading MathJax:', error)
      this.mathJaxLoading = false
    }
  }

  private async loadKaTeX(): Promise<void> {
    if (this.katexLoaded.value || this.katexLoading) return
    
    this.katexLoading = true

    try {
      // Load KaTeX CSS
      const cssLink = document.createElement('link')
      cssLink.rel = 'stylesheet'
      cssLink.href = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css'
      document.head.appendChild(cssLink)

      // Load KaTeX script
      const script = document.createElement('script')
      script.src = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js'
      script.async = true
      
      script.onload = () => {
        this.katexLoaded.value = true
        console.log('KaTeX loaded successfully')
      }
      
      script.onerror = () => {
        console.error('Failed to load KaTeX')
        this.katexLoading = false
      }

      document.head.appendChild(script)
    } catch (error) {
      console.error('Error loading KaTeX:', error)
      this.katexLoading = false
    }
  }

  async waitForEngine(engine: RenderEngine): Promise<boolean> {
    const maxWait = 10000 // 10 seconds
    const checkInterval = 100 // 100ms
    let waited = 0

    while (waited < maxWait) {
      if (engine === 'mathjax' && this.mathJaxLoaded.value) return true
      if (engine === 'katex' && this.katexLoaded.value) return true
      
      await new Promise(resolve => setTimeout(resolve, checkInterval))
      waited += checkInterval
    }

    return false
  }

  async renderWithMathJax(latex: string, options: RenderOptions = {}): Promise<RenderResult> {
    const startTime = performance.now()

    try {
      if (!this.mathJaxLoaded.value) {
        const loaded = await this.waitForEngine('mathjax')
        if (!loaded) {
          return {
            success: false,
            error: 'MathJax failed to load'
          }
        }
      }

      // Prepare LaTeX with proper delimiters
      const processedLatex = this.prepareLatexForMathJax(latex, options.displayMode)

      // Create a temporary container
      const container = document.createElement('div')
      container.innerHTML = processedLatex
      container.style.visibility = 'hidden'
      container.style.position = 'absolute'
      document.body.appendChild(container)

      try {
        // Render with MathJax
        await window.MathJax.typesetPromise([container])
        
        // Get the rendered HTML
        const renderedHtml = container.innerHTML
        
        // Clean up
        document.body.removeChild(container)

        return {
          success: true,
          html: renderedHtml,
          renderTime: performance.now() - startTime
        }
      } catch (renderError) {
        document.body.removeChild(container)
        throw renderError
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'MathJax rendering failed',
        renderTime: performance.now() - startTime
      }
    }
  }

  async renderWithKaTeX(latex: string, options: RenderOptions = {}): Promise<RenderResult> {
    const startTime = performance.now()

    try {
      if (!this.katexLoaded.value) {
        const loaded = await this.waitForEngine('katex')
        if (!loaded) {
          return {
            success: false,
            error: 'KaTeX failed to load'
          }
        }
      }

      // Prepare KaTeX options
      const katexOptions: KaTeXOptions = {
        displayMode: options.displayMode || false,
        throwOnError: options.throwOnError !== undefined ? options.throwOnError : false,
        errorColor: options.errorColor || '#cc0000',
        macros: options.macros || {},
        strict: false,
        trust: true
      }

      // Clean LaTeX (remove delimiters if present)
      const cleanLatex = this.cleanLatexForKaTeX(latex)

      // Render with KaTeX
      const html = (window as any).katex.renderToString(cleanLatex, katexOptions)

      return {
        success: true,
        html,
        renderTime: performance.now() - startTime
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'KaTeX rendering failed',
        renderTime: performance.now() - startTime
      }
    }
  }

  async render(latex: string, engine: RenderEngine = this.defaultEngine, options: RenderOptions = {}): Promise<RenderResult> {
    if (!latex.trim()) {
      return {
        success: false,
        error: 'Empty LaTeX input'
      }
    }

    try {
      if (engine === 'katex') {
        return await this.renderWithKaTeX(latex, options)
      } else {
        return await this.renderWithMathJax(latex, options)
      }
    } catch (error) {
      // Fallback to the other engine
      const fallbackEngine = engine === 'katex' ? 'mathjax' : 'katex'
      console.warn(`${engine} failed, trying ${fallbackEngine}:`, error)
      
      try {
        if (fallbackEngine === 'katex') {
          return await this.renderWithKaTeX(latex, options)
        } else {
          return await this.renderWithMathJax(latex, options)
        }
      } catch (fallbackError) {
        return {
          success: false,
          error: `Both rendering engines failed. ${engine}: ${error instanceof Error ? error.message : 'Unknown error'}. ${fallbackEngine}: ${fallbackError instanceof Error ? fallbackError.message : 'Unknown error'}`
        }
      }
    }
  }

  private prepareLatexForMathJax(latex: string, displayMode?: boolean): string {
    let processed = latex.trim()

    // Remove existing delimiters
    processed = processed.replace(/^\$+|\$+$/g, '')
    processed = processed.replace(/^\\[\[\(]|\\[\]\)]$/g, '')

    // Add appropriate delimiters
    if (displayMode) {
      return `$$${processed}$$`
    } else {
      return `$${processed}$`
    }
  }

  private cleanLatexForKaTeX(latex: string): string {
    let processed = latex.trim()

    // Remove math delimiters
    processed = processed.replace(/^\$+|\$+$/g, '')
    processed = processed.replace(/^\\[\[\(]|\\[\]\)]$/g, '')

    return processed
  }

  setDefaultEngine(engine: RenderEngine) {
    this.defaultEngine = engine
  }

  getDefaultEngine(): RenderEngine {
    return this.defaultEngine
  }

  isEngineLoaded(engine: RenderEngine): boolean {
    return engine === 'mathjax' ? this.mathJaxLoaded.value : this.katexLoaded.value
  }

  getEngineStatus() {
    return {
      mathjax: {
        loaded: this.mathJaxLoaded.value,
        loading: this.mathJaxLoading
      },
      katex: {
        loaded: this.katexLoaded.value,
        loading: this.katexLoading
      }
    }
  }

  // Utility method to validate LaTeX syntax
  validateLatex(latex: string): { valid: boolean; error?: string } {
    const trimmed = latex.trim()
    if (!trimmed) {
      return { valid: false, error: 'Empty LaTeX input' }
    }

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
            return { valid: false, error: 'Unmatched closing brace' }
          }
          break
        case '$':
          inMathMode = !inMathMode
          break
      }

      i++
    }

    if (braceCount !== 0) {
      return { 
        valid: false, 
        error: braceCount > 0 ? 'Missing closing brace' : 'Extra closing brace' 
      }
    }

    if (inMathMode) {
      return { valid: false, error: 'Unclosed math mode' }
    }

    return { valid: true }
  }

  // Method to preload both engines
  async preloadEngines(): Promise<void> {
    await this.initializeEngines()
  }
}

// Create singleton instance
export const mathRenderer = new MathRenderer()

// Export for use in components
export default mathRenderer