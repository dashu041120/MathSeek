// Core data types for MathSeek application

export enum InputType {
  SingleFormula = 'SingleFormula',
  Document = 'Document'
}

export enum ExportFormat {
  LaTeX = 'LaTeX',
  LaTeXInline = 'LaTeXInline',
  LaTeXBlock = 'LaTeXBlock',
  Markdown = 'Markdown',
  MarkdownInline = 'MarkdownInline',
  MarkdownBlock = 'MarkdownBlock',
  DOCX = 'DOCX',
  HTML = 'HTML',
  PlainText = 'PlainText'
}

export enum RenderEngine {
  MathJax = 'MathJax',
  KaTeX = 'KaTeX'
}

export enum InlineFormat {
  Dollar = 'Dollar',      // $...$
  Parentheses = 'Parentheses' // \(...\)
}

export enum BlockFormat {
  DoubleDollar = 'DoubleDollar', // $$...$$
  Brackets = 'Brackets'     // \[...\]
}

export interface MarkdownFormulaFormat {
  inline: InlineFormat
  block: BlockFormat
}

export interface AppConfig {
  apiEndpoint: string
  apiKey: string
  defaultExportFormat: Record<InputType, ExportFormat>
  renderEngine: RenderEngine
  markdownFormulaFormat: MarkdownFormulaFormat
}

export interface FormulaResult {
  latex: string
  confidence: number
  timestamp: number
  inputType: InputType
  content: ResultContent
}

export type ResultContent = 
  | { SingleFormula: string }
  | { Document: DocumentContent }

export interface DocumentContent {
  title?: string
  sections: DocumentSection[]
}

export interface DocumentSection {
  heading?: string
  text: string
  formulas: FormulaBlock[]
}

export interface FormulaBlock {
  latex: string
  position: number
  isInline: boolean
}

export interface AnalysisResult {
  formulaType: string
  description: string
  usage: string
  examples: string[]
}

export interface ImageLayout {
  has_multiple_formulas: boolean
  has_text_content: boolean
  formula_regions: Region[]
  text_regions: Region[]
}

export interface Region {
  x: number
  y: number
  width: number
  height: number
}

export interface SystemStatus {
  clipboard_available: boolean
  screenshot_available: boolean
  api_configured: boolean
  render_engine_ready: boolean
}

// Application state interface
export interface AppState {
  currentImage: string | null
  inputType: InputType
  recognitionResult: FormulaResult | null
  analysisResult: AnalysisResult | null
  isProcessing: boolean
  config: AppConfig
}

// Type guards and utility functions
export function isResultContentSingleFormula(content: ResultContent): content is { SingleFormula: string } {
  return 'SingleFormula' in content
}

export function isResultContentDocument(content: ResultContent): content is { Document: DocumentContent } {
  return 'Document' in content
}

export function createDefaultAppConfig(): AppConfig {
  return {
    apiEndpoint: '',
    apiKey: '',
    defaultExportFormat: {
      [InputType.SingleFormula]: ExportFormat.LaTeX,
      [InputType.Document]: ExportFormat.Markdown
    },
    renderEngine: RenderEngine.MathJax,
    markdownFormulaFormat: {
      inline: InlineFormat.Dollar,
      block: BlockFormat.DoubleDollar
    }
  }
}

export function createDocumentContent(title?: string): DocumentContent {
  return {
    title,
    sections: []
  }
}

export function createDocumentSection(heading?: string, text: string = ''): DocumentSection {
  return {
    heading,
    text,
    formulas: []
  }
}

export function createFormulaBlock(latex: string, position: number, isInline: boolean = false): FormulaBlock {
  return {
    latex,
    position,
    isInline
  }
}

export function createFormulaResultSingle(latex: string, confidence: number): FormulaResult {
  return {
    latex,
    confidence,
    timestamp: Date.now(),
    inputType: InputType.SingleFormula,
    content: { SingleFormula: latex }
  }
}

export function createFormulaResultDocument(latex: string, confidence: number, document: DocumentContent): FormulaResult {
  return {
    latex,
    confidence,
    timestamp: Date.now(),
    inputType: InputType.Document,
    content: { Document: document }
  }
}

// Component prop types
export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger' | 'outline'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
}

export interface CardProps {
  padding?: 'none' | 'sm' | 'md' | 'lg'
  hoverable?: boolean
}

export interface InputProps {
  modelValue?: string
  type?: string
  label?: string
  placeholder?: string
  helpText?: string
  errorMessage?: string
  disabled?: boolean
  required?: boolean
}

// Error handling types
export enum MathSeekErrorType {
  ApiError = 'ApiError',
  ImageError = 'ImageError',
  ConfigError = 'ConfigError',
  ExportError = 'ExportError',
  NetworkError = 'NetworkError',
  IoError = 'IoError',
  SerializationError = 'SerializationError',
  Unknown = 'Unknown'
}

export interface MathSeekError {
  type: MathSeekErrorType
  message: string
}

// Validation functions
export function validateAppConfig(config: AppConfig): string | null {
  if (!config.apiEndpoint.trim()) {
    return 'API endpoint cannot be empty'
  }
  
  if (!config.apiKey.trim()) {
    return 'API key cannot be empty'
  }
  
  if (!config.apiEndpoint.startsWith('http://') && !config.apiEndpoint.startsWith('https://')) {
    return 'API endpoint must be a valid URL'
  }
  
  return null
}

export function validateFormulaResult(result: FormulaResult): string | null {
  if (!result.latex.trim()) {
    return 'LaTeX content cannot be empty'
  }
  
  if (result.confidence < 0 || result.confidence > 1) {
    return 'Confidence must be between 0 and 1'
  }
  
  return null
}

export function validateDocumentContent(document: DocumentContent): string | null {
  if (document.sections.length === 0) {
    return 'Document must have at least one section'
  }
  
  for (const section of document.sections) {
    const sectionError = validateDocumentSection(section)
    if (sectionError) {
      return sectionError
    }
  }
  
  return null
}

export function validateDocumentSection(section: DocumentSection): string | null {
  if (!section.text.trim() && section.formulas.length === 0) {
    return 'Section must have either text or formulas'
  }
  
  return null
}

// Utility functions for format conversion
export function formatInlineFormula(latex: string, format: InlineFormat): string {
  switch (format) {
    case InlineFormat.Dollar:
      return `$${latex}$`
    case InlineFormat.Parentheses:
      return `\\(${latex}\\)`
    default:
      return `$${latex}$`
  }
}

export function formatBlockFormula(latex: string, format: BlockFormat): string {
  switch (format) {
    case BlockFormat.DoubleDollar:
      return `$$${latex}$$`
    case BlockFormat.Brackets:
      return `\\[${latex}\\]`
    default:
      return `$$${latex}$$`
  }
}