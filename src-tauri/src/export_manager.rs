use crate::{
    MathSeekError, MathSeekResult, AppConfig, FormulaResult, DocumentContent, 
    DocumentSection, FormulaBlock, ExportFormat, InputType, InlineFormat, BlockFormat
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Export configuration for different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub custom_template: Option<String>,
    pub format_options: HashMap<String, String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::LaTeX,
            include_metadata: true,
            custom_template: None,
            format_options: HashMap::new(),
        }
    }
}

/// Export result containing the formatted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub content: String,
    pub format: ExportFormat,
    pub metadata: ExportMetadata,
}

/// Metadata about the export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub timestamp: u64,
    pub original_input_type: InputType,
    pub export_format: ExportFormat,
    pub character_count: usize,
    pub formula_count: usize,
    pub processing_time_ms: u64,
}

/// Export manager for handling different output formats
pub struct ExportManager {
    config: AppConfig,
}

impl ExportManager {
    /// Create a new export manager with the given configuration
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Export a formula result to the specified format
    pub fn export_formula_result(
        &self,
        result: &FormulaResult,
        export_config: &ExportConfig,
    ) -> MathSeekResult<ExportResult> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let content = match export_config.format {
            ExportFormat::LaTeX => self.export_to_latex(result, export_config)?,
            ExportFormat::LaTeXInline => self.export_to_latex_inline(result, export_config)?,
            ExportFormat::LaTeXBlock => self.export_to_latex_block(result, export_config)?,
            ExportFormat::Markdown => self.export_to_markdown(result, export_config)?,
            ExportFormat::MarkdownInline => self.export_to_markdown_inline(result, export_config)?,
            ExportFormat::MarkdownBlock => self.export_to_markdown_block(result, export_config)?,
            ExportFormat::HTML => self.export_to_html(result, export_config)?,
            ExportFormat::DOCX => self.export_to_docx(result, export_config)?,
            ExportFormat::PlainText => self.export_to_plain_text(result, export_config)?,
        };

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let formula_count = match &result.content {
            crate::ResultContent::SingleFormula(_) => 1,
            crate::ResultContent::Document(doc) => {
                doc.sections.iter().map(|s| s.formulas.len()).sum()
            }
        };

        let metadata = ExportMetadata {
            timestamp: end_time,
            original_input_type: result.input_type.clone(),
            export_format: export_config.format.clone(),
            character_count: content.len(),
            formula_count,
            processing_time_ms: end_time - start_time,
        };

        Ok(ExportResult {
            content,
            format: export_config.format.clone(),
            metadata,
        })
    }

    /// Export to LaTeX format
    fn export_to_latex(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                Ok(latex.clone())
            }
            crate::ResultContent::Document(doc) => {
                self.document_to_latex(doc)
            }
        }
    }

    /// Export to inline LaTeX format
    fn export_to_latex_inline(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                Ok(format!("${}$", latex.trim_matches('$')))
            }
            crate::ResultContent::Document(doc) => {
                // Convert all formulas to inline format
                let mut content = String::new();
                for section in &doc.sections {
                    if let Some(heading) = &section.heading {
                        content.push_str(&format!("\\section{{{}}}\n\n", heading));
                    }
                    
                    let mut text = section.text.clone();
                    for formula in &section.formulas {
                        let inline_formula = format!("${}$", formula.latex.trim_matches('$'));
                        text.insert_str(formula.position, &inline_formula);
                    }
                    
                    content.push_str(&text);
                    content.push_str("\n\n");
                }
                Ok(content)
            }
        }
    }

    /// Export to block LaTeX format
    fn export_to_latex_block(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                Ok(format!("$${}$$", latex.trim_matches('$')))
            }
            crate::ResultContent::Document(doc) => {
                self.document_to_latex_block(doc)
            }
        }
    }

    /// Export to Markdown format
    fn export_to_markdown(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                let format = &self.config.markdown_formula_format;
                Ok(self.format_formula_for_markdown(latex, false, format))
            }
            crate::ResultContent::Document(doc) => {
                self.document_to_markdown(doc)
            }
        }
    }

    /// Export to inline Markdown format
    fn export_to_markdown_inline(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                let format = &self.config.markdown_formula_format;
                Ok(self.format_formula_for_markdown(latex, true, format))
            }
            crate::ResultContent::Document(doc) => {
                self.document_to_markdown_inline(doc)
            }
        }
    }

    /// Export to block Markdown format
    fn export_to_markdown_block(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                let format = &self.config.markdown_formula_format;
                Ok(self.format_formula_for_markdown(latex, false, format))
            }
            crate::ResultContent::Document(doc) => {
                self.document_to_markdown_block(doc)
            }
        }
    }

    /// Export to HTML format
    fn export_to_html(&self, result: &FormulaResult, config: &ExportConfig) -> MathSeekResult<String> {
        let mut html = String::new();
        
        // HTML header
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str("<title>Mathematical Formula</title>\n");
        
        // Include MathJax
        html.push_str("<script src=\"https://polyfill.io/v3/polyfill.min.js?features=es6\"></script>\n");
        html.push_str("<script id=\"MathJax-script\" async src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"></script>\n");
        html.push_str("<script>\n");
        html.push_str("window.MathJax = {\n");
        html.push_str("  tex: {\n");
        html.push_str("    inlineMath: [['$', '$'], ['\\\\(', '\\\\)']],\n");
        html.push_str("    displayMath: [['$$', '$$'], ['\\\\[', '\\\\]']]\n");
        html.push_str("  }\n");
        html.push_str("};\n");
        html.push_str("</script>\n");
        
        // CSS styles
        html.push_str("<style>\n");
        html.push_str("body { font-family: serif; margin: 2rem; line-height: 1.6; }\n");
        html.push_str(".formula { text-align: center; margin: 1rem 0; }\n");
        html.push_str(".document-title { text-align: center; font-size: 1.5em; margin-bottom: 2rem; }\n");
        html.push_str(".section-heading { font-size: 1.2em; margin: 1.5rem 0 1rem 0; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Content
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                html.push_str("<div class=\"formula\">\n");
                html.push_str(&format!("$${}$$", latex.trim_matches('$')));
                html.push_str("\n</div>\n");
            }
            crate::ResultContent::Document(doc) => {
                if let Some(title) = &doc.title {
                    html.push_str(&format!("<h1 class=\"document-title\">{}</h1>\n", title));
                }
                
                for section in &doc.sections {
                    if let Some(heading) = &section.heading {
                        html.push_str(&format!("<h2 class=\"section-heading\">{}</h2>\n", heading));
                    }
                    
                    if !section.text.is_empty() {
                        let mut text = section.text.clone();
                        
                        // Insert formulas at their positions
                        let mut sorted_formulas = section.formulas.clone();
                        sorted_formulas.sort_by(|a, b| b.position.cmp(&a.position));
                        
                        for formula in sorted_formulas {
                            let formula_html = if formula.is_inline {
                                format!("${}$", formula.latex.trim_matches('$'))
                            } else {
                                format!("$${}$$", formula.latex.trim_matches('$'))
                            };
                            text.insert_str(formula.position, &formula_html);
                        }
                        
                        // Convert line breaks to paragraphs
                        let paragraphs: Vec<&str> = text.split("\n\n").collect();
                        for paragraph in paragraphs {
                            if !paragraph.trim().is_empty() {
                                html.push_str(&format!("<p>{}</p>\n", paragraph.trim()));
                            }
                        }
                    }
                    
                    html.push_str("\n");
                }
            }
        }

        // Metadata
        if config.include_metadata {
            html.push_str("<hr>\n");
            html.push_str("<div class=\"metadata\">\n");
            html.push_str("<small>\n");
            html.push_str(&format!("Generated by MathSeek<br>\n"));
            html.push_str(&format!("Input Type: {:?}<br>\n", result.input_type));
            html.push_str(&format!("Confidence: {:.2}<br>\n", result.confidence));
            html.push_str("</small>\n");
            html.push_str("</div>\n");
        }

        html.push_str("</body>\n</html>");
        Ok(html)
    }

    /// Export to DOCX format (placeholder implementation)
    fn export_to_docx(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        // This is a placeholder - real DOCX export would require a library like docx-rs
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                Ok(format!("DOCX Export:\n\nFormula: {}", latex))
            }
            crate::ResultContent::Document(doc) => {
                let mut content = String::from("DOCX Export:\n\n");
                
                if let Some(title) = &doc.title {
                    content.push_str(&format!("Title: {}\n\n", title));
                }
                
                for (i, section) in doc.sections.iter().enumerate() {
                    content.push_str(&format!("Section {}:\n", i + 1));
                    
                    if let Some(heading) = &section.heading {
                        content.push_str(&format!("Heading: {}\n", heading));
                    }
                    
                    if !section.text.is_empty() {
                        content.push_str(&format!("Text: {}\n", section.text));
                    }
                    
                    for (j, formula) in section.formulas.iter().enumerate() {
                        content.push_str(&format!("Formula {}: {} ({})\n", 
                            j + 1, 
                            formula.latex,
                            if formula.is_inline { "inline" } else { "block" }
                        ));
                    }
                    
                    content.push_str("\n");
                }
                
                Ok(content)
            }
        }
    }

    /// Export to plain text format
    fn export_to_plain_text(&self, result: &FormulaResult, _config: &ExportConfig) -> MathSeekResult<String> {
        match &result.content {
            crate::ResultContent::SingleFormula(latex) => {
                Ok(latex.clone())
            }
            crate::ResultContent::Document(doc) => {
                let mut content = String::new();
                
                if let Some(title) = &doc.title {
                    content.push_str(&format!("{}\n", title));
                    content.push_str(&"=".repeat(title.len()));
                    content.push_str("\n\n");
                }
                
                for section in &doc.sections {
                    if let Some(heading) = &section.heading {
                        content.push_str(&format!("{}\n", heading));
                        content.push_str(&"-".repeat(heading.len()));
                        content.push_str("\n\n");
                    }
                    
                    if !section.text.is_empty() {
                        content.push_str(&section.text);
                        content.push_str("\n\n");
                    }
                    
                    if !section.formulas.is_empty() {
                        content.push_str("Formulas:\n");
                        for (i, formula) in section.formulas.iter().enumerate() {
                            content.push_str(&format!("{}. {} ({})\n", 
                                i + 1, 
                                formula.latex,
                                if formula.is_inline { "inline" } else { "block" }
                            ));
                        }
                        content.push_str("\n");
                    }
                }
                
                Ok(content)
            }
        }
    }

    /// Convert document to LaTeX format
    fn document_to_latex(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        let mut latex = String::new();
        
        // Document class and packages
        latex.push_str("\\documentclass{article}\n");
        latex.push_str("\\usepackage{amsmath}\n");
        latex.push_str("\\usepackage{amsfonts}\n");
        latex.push_str("\\usepackage{amssymb}\n");
        latex.push_str("\\usepackage[utf8]{inputenc}\n");
        latex.push_str("\n");
        
        latex.push_str("\\begin{document}\n\n");
        
        if let Some(title) = &doc.title {
            latex.push_str(&format!("\\title{{{}}}\n", title));
            latex.push_str("\\maketitle\n\n");
        }
        
        for section in &doc.sections {
            if let Some(heading) = &section.heading {
                latex.push_str(&format!("\\section{{{}}}\n\n", heading));
            }
            
            if !section.text.is_empty() {
                let mut text = section.text.clone();
                
                // Insert formulas at their positions
                let mut sorted_formulas = section.formulas.clone();
                sorted_formulas.sort_by(|a, b| b.position.cmp(&a.position));
                
                for formula in sorted_formulas {
                    let formula_latex = if formula.is_inline {
                        format!("${}$", formula.latex.trim_matches('$'))
                    } else {
                        format!("\\begin{{equation}}\n{}\n\\end{{equation}}", formula.latex.trim_matches('$'))
                    };
                    text.insert_str(formula.position, &formula_latex);
                }
                
                latex.push_str(&text);
                latex.push_str("\n\n");
            }
        }
        
        latex.push_str("\\end{document}");
        Ok(latex)
    }

    /// Convert document to block LaTeX format
    fn document_to_latex_block(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        let mut latex = String::new();
        
        if let Some(title) = &doc.title {
            latex.push_str(&format!("{}\n", title));
            latex.push_str(&"=".repeat(title.len()));
            latex.push_str("\n\n");
        }
        
        for section in &doc.sections {
            if let Some(heading) = &section.heading {
                latex.push_str(&format!("{}\n", heading));
                latex.push_str(&"-".repeat(heading.len()));
                latex.push_str("\n\n");
            }
            
            if !section.text.is_empty() {
                latex.push_str(&section.text);
                latex.push_str("\n\n");
            }
            
            for formula in &section.formulas {
                latex.push_str(&format!("$${}$$\n\n", formula.latex.trim_matches('$')));
            }
        }
        
        Ok(latex)
    }

    /// Convert document to Markdown format
    fn document_to_markdown(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        let mut markdown = String::new();
        let format = &self.config.markdown_formula_format;
        
        if let Some(title) = &doc.title {
            markdown.push_str(&format!("# {}\n\n", title));
        }
        
        for section in &doc.sections {
            if let Some(heading) = &section.heading {
                markdown.push_str(&format!("## {}\n\n", heading));
            }
            
            if !section.text.is_empty() {
                let mut text = section.text.clone();
                
                // Insert formulas at their positions
                let mut sorted_formulas = section.formulas.clone();
                sorted_formulas.sort_by(|a, b| b.position.cmp(&a.position));
                
                for formula in sorted_formulas {
                    let formula_md = self.format_formula_for_markdown(&formula.latex, formula.is_inline, format);
                    text.insert_str(formula.position, &formula_md);
                }
                
                markdown.push_str(&text);
                markdown.push_str("\n\n");
            }
        }
        
        Ok(markdown)
    }

    /// Convert document to inline Markdown format
    fn document_to_markdown_inline(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        let mut markdown = String::new();
        let format = &self.config.markdown_formula_format;
        
        if let Some(title) = &doc.title {
            markdown.push_str(&format!("# {}\n\n", title));
        }
        
        for section in &doc.sections {
            if let Some(heading) = &section.heading {
                markdown.push_str(&format!("## {}\n\n", heading));
            }
            
            if !section.text.is_empty() {
                let mut text = section.text.clone();
                
                // Convert all formulas to inline
                let mut sorted_formulas = section.formulas.clone();
                sorted_formulas.sort_by(|a, b| b.position.cmp(&a.position));
                
                for formula in sorted_formulas {
                    let formula_md = self.format_formula_for_markdown(&formula.latex, true, format);
                    text.insert_str(formula.position, &formula_md);
                }
                
                markdown.push_str(&text);
                markdown.push_str("\n\n");
            }
        }
        
        Ok(markdown)
    }

    /// Convert document to block Markdown format
    fn document_to_markdown_block(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        let mut markdown = String::new();
        let format = &self.config.markdown_formula_format;
        
        if let Some(title) = &doc.title {
            markdown.push_str(&format!("# {}\n\n", title));
        }
        
        for section in &doc.sections {
            if let Some(heading) = &section.heading {
                markdown.push_str(&format!("## {}\n\n", heading));
            }
            
            if !section.text.is_empty() {
                markdown.push_str(&section.text);
                markdown.push_str("\n\n");
            }
            
            for formula in &section.formulas {
                let formula_md = self.format_formula_for_markdown(&formula.latex, false, format);
                markdown.push_str(&formula_md);
                markdown.push_str("\n\n");
            }
        }
        
        Ok(markdown)
    }

    /// Format a formula for Markdown based on configuration
    fn format_formula_for_markdown(&self, latex: &str, is_inline: bool, format: &crate::MarkdownFormulaFormat) -> String {
        let clean_latex = latex.trim_matches('$');
        
        if is_inline {
            match format.inline {
                InlineFormat::Dollar => format!("${}$", clean_latex),
                InlineFormat::Parentheses => format!("\\({}\\)", clean_latex),
            }
        } else {
            match format.block {
                BlockFormat::DoubleDollar => format!("$${}$$", clean_latex),
                BlockFormat::Brackets => format!("\\[{}\\]", clean_latex),
            }
        }
    }

    /// Get available export formats for a given input type
    pub fn get_available_formats(&self, input_type: &InputType) -> Vec<ExportFormat> {
        match input_type {
            InputType::SingleFormula => vec![
                ExportFormat::LaTeX,
                ExportFormat::LaTeXInline,
                ExportFormat::LaTeXBlock,
                ExportFormat::Markdown,
                ExportFormat::MarkdownInline,
                ExportFormat::MarkdownBlock,
                ExportFormat::HTML,
                ExportFormat::PlainText,
            ],
            InputType::Document => vec![
                ExportFormat::LaTeX,
                ExportFormat::Markdown,
                ExportFormat::HTML,
                ExportFormat::DOCX,
                ExportFormat::PlainText,
            ],
        }
    }

    /// Get the default export format for a given input type
    pub fn get_default_format(&self, input_type: &InputType) -> ExportFormat {
        self.config.default_export_format
            .get(input_type)
            .cloned()
            .unwrap_or_else(|| match input_type {
                InputType::SingleFormula => ExportFormat::LaTeX,
                InputType::Document => ExportFormat::Markdown,
            })
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FormulaResult, ResultContent, DocumentContent, DocumentSection, FormulaBlock};

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    fn create_test_single_formula() -> FormulaResult {
        FormulaResult::new_single_formula("x^2 + y^2 = r^2".to_string(), 0.95)
    }

    #[test]
    fn test_export_single_formula_to_latex() {
        let config = create_test_config();
        let manager = ExportManager::new(config);
        let formula = create_test_single_formula();
        let export_config = ExportConfig::default();

        let result = manager.export_formula_result(&formula, &export_config).unwrap();
        assert_eq!(result.content, "x^2 + y^2 = r^2");
        assert_eq!(result.format, ExportFormat::LaTeX);
    }

    #[test]
    fn test_export_single_formula_to_markdown() {
        let config = create_test_config();
        let manager = ExportManager::new(config);
        let formula = create_test_single_formula();
        let export_config = ExportConfig {
            format: ExportFormat::Markdown,
            ..Default::default()
        };

        let result = manager.export_formula_result(&formula, &export_config).unwrap();
        assert!(result.content.contains("x^2 + y^2 = r^2"));
    }

    #[test]
    fn test_get_available_formats() {
        let config = create_test_config();
        let manager = ExportManager::new(config);

        let single_formats = manager.get_available_formats(&InputType::SingleFormula);
        assert!(single_formats.contains(&ExportFormat::LaTeX));
        assert!(single_formats.contains(&ExportFormat::Markdown));

        let doc_formats = manager.get_available_formats(&InputType::Document);
        assert!(doc_formats.contains(&ExportFormat::LaTeX));
        assert!(doc_formats.contains(&ExportFormat::HTML));
    }
}