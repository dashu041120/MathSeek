import { describe, it, expect } from 'vitest'
import {
  InputType,
  ExportFormat,
  RenderEngine,
  InlineFormat,
  BlockFormat,
  createDefaultAppConfig,
  createDocumentContent,
  createDocumentSection,
  createFormulaBlock,
  createFormulaResultSingle,
  createFormulaResultDocument,
  isResultContentSingleFormula,
  isResultContentDocument,
  validateAppConfig,
  validateFormulaResult,
  validateDocumentContent,
  validateDocumentSection,
  formatInlineFormula,
  formatBlockFormula,

  type ResultContent
} from './index'

describe('Data Types', () => {
  describe('Enums', () => {
    it('should have correct InputType values', () => {
      expect(InputType.SingleFormula).toBe('SingleFormula')
      expect(InputType.Document).toBe('Document')
    })

    it('should have correct ExportFormat values', () => {
      expect(ExportFormat.LaTeX).toBe('LaTeX')
      expect(ExportFormat.Markdown).toBe('Markdown')
      expect(ExportFormat.DOCX).toBe('DOCX')
    })

    it('should have correct RenderEngine values', () => {
      expect(RenderEngine.MathJax).toBe('MathJax')
      expect(RenderEngine.KaTeX).toBe('KaTeX')
    })
  })

  describe('Factory Functions', () => {
    it('should create default app config', () => {
      const config = createDefaultAppConfig()
      
      expect(config.apiEndpoint).toBe('')
      expect(config.apiKey).toBe('')
      expect(config.renderEngine).toBe(RenderEngine.MathJax)
      expect(config.markdownFormulaFormat.inline).toBe(InlineFormat.Dollar)
      expect(config.markdownFormulaFormat.block).toBe(BlockFormat.DoubleDollar)
      expect(config.defaultExportFormat[InputType.SingleFormula]).toBe(ExportFormat.LaTeX)
      expect(config.defaultExportFormat[InputType.Document]).toBe(ExportFormat.Markdown)
    })

    it('should create document content', () => {
      const document = createDocumentContent('Test Title')
      
      expect(document.title).toBe('Test Title')
      expect(document.sections).toEqual([])
    })

    it('should create document section', () => {
      const section = createDocumentSection('Heading', 'Content')
      
      expect(section.heading).toBe('Heading')
      expect(section.text).toBe('Content')
      expect(section.formulas).toEqual([])
    })

    it('should create formula block', () => {
      const formula = createFormulaBlock('E = mc^2', 10, true)
      
      expect(formula.latex).toBe('E = mc^2')
      expect(formula.position).toBe(10)
      expect(formula.isInline).toBe(true)
    })

    it('should create single formula result', () => {
      const result = createFormulaResultSingle('x = y + z', 0.95)
      
      expect(result.latex).toBe('x = y + z')
      expect(result.confidence).toBe(0.95)
      expect(result.inputType).toBe(InputType.SingleFormula)
      expect(isResultContentSingleFormula(result.content)).toBe(true)
      
      if (isResultContentSingleFormula(result.content)) {
        expect(result.content.SingleFormula).toBe('x = y + z')
      }
    })

    it('should create document formula result', () => {
      const document = createDocumentContent('Test')
      const result = createFormulaResultDocument('Combined formulas', 0.9, document)
      
      expect(result.latex).toBe('Combined formulas')
      expect(result.confidence).toBe(0.9)
      expect(result.inputType).toBe(InputType.Document)
      expect(isResultContentDocument(result.content)).toBe(true)
      
      if (isResultContentDocument(result.content)) {
        expect(result.content.Document.title).toBe('Test')
      }
    })
  })

  describe('Type Guards', () => {
    it('should correctly identify single formula content', () => {
      const singleContent: ResultContent = { SingleFormula: 'E = mc^2' }
      const documentContent: ResultContent = { Document: createDocumentContent() }
      
      expect(isResultContentSingleFormula(singleContent)).toBe(true)
      expect(isResultContentSingleFormula(documentContent)).toBe(false)
      
      expect(isResultContentDocument(singleContent)).toBe(false)
      expect(isResultContentDocument(documentContent)).toBe(true)
    })
  })

  describe('Validation Functions', () => {
    it('should validate app config', () => {
      const config = createDefaultAppConfig()
      
      // Empty config should fail
      expect(validateAppConfig(config)).toBe('API endpoint cannot be empty')
      
      // Set endpoint but not key
      config.apiEndpoint = 'https://api.example.com'
      expect(validateAppConfig(config)).toBe('API key cannot be empty')
      
      // Invalid URL
      config.apiKey = 'test-key'
      config.apiEndpoint = 'invalid-url'
      expect(validateAppConfig(config)).toBe('API endpoint must be a valid URL')
      
      // Valid config
      config.apiEndpoint = 'https://api.example.com'
      expect(validateAppConfig(config)).toBeNull()
    })

    it('should validate formula result', () => {
      let result = createFormulaResultSingle('E = mc^2', 0.95)
      
      // Valid result
      expect(validateFormulaResult(result)).toBeNull()
      
      // Invalid confidence
      result.confidence = 1.5
      expect(validateFormulaResult(result)).toBe('Confidence must be between 0 and 1')
      
      result.confidence = -0.1
      expect(validateFormulaResult(result)).toBe('Confidence must be between 0 and 1')
      
      // Empty latex
      result.confidence = 0.95
      result.latex = ''
      expect(validateFormulaResult(result)).toBe('LaTeX content cannot be empty')
    })

    it('should validate document content', () => {
      let document = createDocumentContent()
      
      // Empty document should fail
      expect(validateDocumentContent(document)).toBe('Document must have at least one section')
      
      // Add empty section
      const emptySection = createDocumentSection()
      document.sections.push(emptySection)
      expect(validateDocumentContent(document)).toBe('Section must have either text or formulas')
      
      // Add valid section
      document.sections = []
      const validSection = createDocumentSection('Title', 'Content')
      document.sections.push(validSection)
      expect(validateDocumentContent(document)).toBeNull()
    })

    it('should validate document section', () => {
      let section = createDocumentSection()
      
      // Empty section should fail
      expect(validateDocumentSection(section)).toBe('Section must have either text or formulas')
      
      // Section with text should pass
      section.text = 'Some content'
      expect(validateDocumentSection(section)).toBeNull()
      
      // Section with formulas should pass
      section.text = ''
      section.formulas.push(createFormulaBlock('x = y', 0))
      expect(validateDocumentSection(section)).toBeNull()
    })
  })

  describe('Format Functions', () => {
    it('should format inline formulas correctly', () => {
      const latex = 'E = mc^2'
      
      expect(formatInlineFormula(latex, InlineFormat.Dollar)).toBe('$E = mc^2$')
      expect(formatInlineFormula(latex, InlineFormat.Parentheses)).toBe('\\(E = mc^2\\)')
    })

    it('should format block formulas correctly', () => {
      const latex = 'E = mc^2'
      
      expect(formatBlockFormula(latex, BlockFormat.DoubleDollar)).toBe('$$E = mc^2$$')
      expect(formatBlockFormula(latex, BlockFormat.Brackets)).toBe('\\[E = mc^2\\]')
    })
  })
})