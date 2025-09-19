import { describe, it, expect } from 'vitest'
import {
  InputType,
  ExportFormat,
  RenderEngine,
  InlineFormat,
  BlockFormat,
  createDefaultAppConfig,
  createFormulaResultSingle,
  createDocumentContent,
  createDocumentSection,
  createFormulaBlock,

  type FormulaResult,
  type DocumentContent
} from './index'

describe('Rust-TypeScript Integration', () => {
  it('should serialize data structures compatible with Rust', () => {
    // Create a complex data structure that matches Rust expectations
    const config = createDefaultAppConfig()
    config.apiEndpoint = 'https://api.example.com'
    config.apiKey = 'test-key-123'
    config.renderEngine = RenderEngine.KaTeX
    config.markdownFormulaFormat.inline = InlineFormat.Parentheses
    config.markdownFormulaFormat.block = BlockFormat.Brackets
    
    // Serialize to JSON (this is what would be sent to Rust)
    const configJson = JSON.stringify(config)
    
    // Verify the JSON structure matches Rust expectations
    const parsed = JSON.parse(configJson)
    expect(parsed.apiEndpoint).toBe('https://api.example.com')
    expect(parsed.apiKey).toBe('test-key-123')
    expect(parsed.renderEngine).toBe('KaTeX')
    expect(parsed.markdownFormulaFormat.inline).toBe('Parentheses')
    expect(parsed.markdownFormulaFormat.block).toBe('Brackets')
    expect(parsed.defaultExportFormat.SingleFormula).toBe('LaTeX')
    expect(parsed.defaultExportFormat.Document).toBe('Markdown')
  })

  it('should handle formula results from Rust', () => {
    // Simulate a FormulaResult coming from Rust
    const rustFormulaResult = {
      latex: 'E = mc^2',
      confidence: 0.95,
      timestamp: 1640995200,
      input_type: 'SingleFormula', // Note: Rust uses snake_case
      content: {
        SingleFormula: 'E = mc^2'
      }
    }
    
    // Convert to TypeScript format (this would happen in the API layer)
    const tsFormulaResult: FormulaResult = {
      latex: rustFormulaResult.latex,
      confidence: rustFormulaResult.confidence,
      timestamp: rustFormulaResult.timestamp,
      inputType: rustFormulaResult.input_type as InputType,
      content: rustFormulaResult.content
    }
    
    expect(tsFormulaResult.latex).toBe('E = mc^2')
    expect(tsFormulaResult.confidence).toBe(0.95)
    expect(tsFormulaResult.inputType).toBe(InputType.SingleFormula)
  })

  it('should handle document content from Rust', () => {
    // Simulate a DocumentContent coming from Rust
    const rustDocument = {
      title: 'Physics Equations',
      sections: [
        {
          heading: 'Energy',
          text: 'The mass-energy equivalence is described by the equation:',
          formulas: [
            {
              latex: 'E = mc^2',
              position: 50,
              is_inline: false // Note: Rust uses snake_case
            }
          ]
        }
      ]
    }
    
    // Convert to TypeScript format
    const tsDocument: DocumentContent = {
      title: rustDocument.title,
      sections: rustDocument.sections.map(section => ({
        heading: section.heading,
        text: section.text,
        formulas: section.formulas.map(formula => ({
          latex: formula.latex,
          position: formula.position,
          isInline: formula.is_inline // Convert snake_case to camelCase
        }))
      }))
    }
    
    expect(tsDocument.title).toBe('Physics Equations')
    expect(tsDocument.sections).toHaveLength(1)
    expect(tsDocument.sections[0].formulas[0].latex).toBe('E = mc^2')
    expect(tsDocument.sections[0].formulas[0].isInline).toBe(false)
  })

  it('should handle enum serialization consistently', () => {
    // Test that enum values serialize to the same strings as Rust
    const inputTypes = [InputType.SingleFormula, InputType.Document]
    const exportFormats = [ExportFormat.LaTeX, ExportFormat.Markdown, ExportFormat.DOCX]
    
    // Serialize enums
    const serializedInputTypes = inputTypes.map(type => JSON.stringify(type))
    const serializedExportFormats = exportFormats.map(format => JSON.stringify(format))
    
    // Verify they match expected Rust enum serialization
    expect(serializedInputTypes).toEqual(['"SingleFormula"', '"Document"'])
    expect(serializedExportFormats).toEqual(['"LaTeX"', '"Markdown"', '"DOCX"'])
  })

  it('should create valid data for Rust commands', () => {
    // Create data structures that would be passed to Tauri commands
    const document = createDocumentContent('Test Document')
    const section = createDocumentSection('Introduction', 'This is a test document.')
    section.formulas.push(createFormulaBlock('x = y + z', 15, true))
    document.sections.push(section)
    
    const result = createFormulaResultSingle('Combined: x = y + z', 0.9)
    
    // Serialize for Tauri command
    const documentJson = JSON.stringify(document)
    const resultJson = JSON.stringify(result)
    
    // Verify structure
    const parsedDocument = JSON.parse(documentJson)
    const parsedResult = JSON.parse(resultJson)
    
    expect(parsedDocument.title).toBe('Test Document')
    expect(parsedDocument.sections[0].formulas[0].isInline).toBe(true)
    expect(parsedResult.inputType).toBe('SingleFormula')
    expect(parsedResult.content.SingleFormula).toBe('Combined: x = y + z')
  })

  it('should handle error cases gracefully', () => {
    // Test that invalid data doesn't break serialization
    const invalidConfig = createDefaultAppConfig()
    invalidConfig.apiEndpoint = '' // Invalid but should still serialize
    
    const configJson = JSON.stringify(invalidConfig)
    const parsed = JSON.parse(configJson)
    
    expect(parsed.apiEndpoint).toBe('')
    expect(parsed.renderEngine).toBe('MathJax') // Should have default value
  })
})