#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json;

    #[test]
    fn test_input_type_serialization() {
        let single_formula = InputType::SingleFormula;
        let document = InputType::Document;
        
        let single_json = serde_json::to_string(&single_formula).unwrap();
        let document_json = serde_json::to_string(&document).unwrap();
        
        assert_eq!(single_json, "\"SingleFormula\"");
        assert_eq!(document_json, "\"Document\"");
        
        let single_deserialized: InputType = serde_json::from_str(&single_json).unwrap();
        let document_deserialized: InputType = serde_json::from_str(&document_json).unwrap();
        
        assert_eq!(single_deserialized, InputType::SingleFormula);
        assert_eq!(document_deserialized, InputType::Document);
    }

    #[test]
    fn test_export_format_serialization() {
        let latex = ExportFormat::LaTeX;
        let markdown = ExportFormat::Markdown;
        
        let latex_json = serde_json::to_string(&latex).unwrap();
        let markdown_json = serde_json::to_string(&markdown).unwrap();
        
        assert_eq!(latex_json, "\"LaTeX\"");
        assert_eq!(markdown_json, "\"Markdown\"");
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        assert!(config.api_endpoint.is_empty());
        assert!(config.api_key.is_empty());
        assert_eq!(config.render_engine, RenderEngine::MathJax);
        assert_eq!(config.markdown_formula_format.inline, InlineFormat::Dollar);
        assert_eq!(config.markdown_formula_format.block, BlockFormat::DoubleDollar);
        
        // Test default export formats
        assert_eq!(config.default_export_format[&InputType::SingleFormula], ExportFormat::LaTeX);
        assert_eq!(config.default_export_format[&InputType::Document], ExportFormat::Markdown);
    }

    #[test]
    fn test_formula_result_creation() {
        let latex = "E = mc^2".to_string();
        let confidence = 0.95;
        
        let result = FormulaResult::new_single_formula(latex.clone(), confidence);
        
        assert_eq!(result.latex, latex);
        assert_eq!(result.confidence, confidence);
        assert_eq!(result.input_type, InputType::SingleFormula);
        
        match result.content {
            ResultContent::SingleFormula(formula) => assert_eq!(formula, latex),
            _ => panic!("Expected SingleFormula content"),
        }
    }

    #[test]
    fn test_document_content_creation() {
        let mut document = DocumentContent::new(Some("Test Document".to_string()));
        
        let mut section = DocumentSection::new(Some("Introduction".to_string()), "This is a test.".to_string());
        section.add_formula(FormulaBlock::new("x = y + z".to_string(), 15, true));
        
        document.add_section(section);
        
        assert_eq!(document.title, Some("Test Document".to_string()));
        assert_eq!(document.sections.len(), 1);
        assert_eq!(document.sections[0].heading, Some("Introduction".to_string()));
        assert_eq!(document.sections[0].formulas.len(), 1);
        assert_eq!(document.sections[0].formulas[0].latex, "x = y + z");
        assert!(document.sections[0].formulas[0].is_inline);
    }

    #[test]
    fn test_app_config_validation() {
        let mut config = AppConfig::default();
        
        // Should fail with empty endpoint and key
        assert!(config.validate().is_err());
        
        // Set endpoint but not key
        config.api_endpoint = "https://api.example.com".to_string();
        assert!(config.validate().is_err());
        
        // Set key but invalid endpoint
        config.api_key = "test-key".to_string();
        config.api_endpoint = "invalid-url".to_string();
        assert!(config.validate().is_err());
        
        // Valid configuration
        config.api_endpoint = "https://api.example.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_formula_result_validation() {
        let mut result = FormulaResult::new_single_formula("E = mc^2".to_string(), 0.95);
        
        // Valid result
        assert!(result.validate().is_ok());
        
        // Invalid confidence
        result.confidence = 1.5;
        assert!(result.validate().is_err());
        
        result.confidence = -0.1;
        assert!(result.validate().is_err());
        
        // Empty latex
        result.confidence = 0.95;
        result.latex = "".to_string();
        assert!(result.validate().is_err());
    }

    #[test]
    fn test_document_validation() {
        let mut document = DocumentContent::new(None);
        
        // Empty document should fail
        assert!(document.validate().is_err());
        
        // Add empty section
        let empty_section = DocumentSection::new(None, "".to_string());
        document.add_section(empty_section);
        assert!(document.validate().is_err());
        
        // Add valid section
        document.sections.clear();
        let valid_section = DocumentSection::new(Some("Title".to_string()), "Content".to_string());
        document.add_section(valid_section);
        assert!(document.validate().is_ok());
    }

    #[test]
    fn test_string_conversions() {
        // Test InputType conversions
        let input_type = InputType::SingleFormula;
        let input_string: String = input_type.into();
        assert_eq!(input_string, "SingleFormula");
        
        let parsed_input: InputType = input_string.try_into().unwrap();
        assert_eq!(parsed_input, InputType::SingleFormula);
        
        // Test ExportFormat conversions
        let export_format = ExportFormat::LaTeX;
        let format_string: String = export_format.into();
        assert_eq!(format_string, "LaTeX");
        
        let parsed_format: ExportFormat = format_string.try_into().unwrap();
        assert_eq!(parsed_format, ExportFormat::LaTeX);
        
        // Test invalid conversions
        let invalid_input: Result<InputType, _> = "InvalidType".to_string().try_into();
        assert!(invalid_input.is_err());
        
        let invalid_format: Result<ExportFormat, _> = "InvalidFormat".to_string().try_into();
        assert!(invalid_format.is_err());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut config = AppConfig::default();
        config.api_endpoint = "https://api.example.com".to_string();
        config.api_key = "test-key".to_string();
        
        // Serialize to JSON
        let json = serde_json::to_string(&config).unwrap();
        
        // Deserialize back
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.api_endpoint, deserialized.api_endpoint);
        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.render_engine, deserialized.render_engine);
    }
}