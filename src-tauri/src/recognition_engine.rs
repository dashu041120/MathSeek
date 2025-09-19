use crate::{
    MathSeekError, MathSeekResult, AppConfig, FormulaResult, InputType, 
    ApiClient, ImageProcessor, ResultContent, DocumentContent, DocumentSection
};
use serde::{Deserialize, Serialize};

/// Configuration for the recognition engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionConfig {
    pub confidence_threshold: f32,
    pub preprocessing_enabled: bool,
    pub auto_type_detection: bool,
    pub validation_enabled: bool,
}

impl Default for RecognitionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.5,
            preprocessing_enabled: true,
            auto_type_detection: true,
            validation_enabled: true,
        }
    }
}

/// Core recognition engine that orchestrates the formula recognition process
pub struct RecognitionEngine {
    api_client: ApiClient,
    config: RecognitionConfig,
}

impl RecognitionEngine {
    /// Create a new recognition engine with the given configuration
    pub fn new(app_config: &AppConfig) -> MathSeekResult<Self> {
        let api_client = ApiClient::from_app_config(app_config)?;
        let config = RecognitionConfig::default();
        
        Ok(Self {
            api_client,
            config,
        })
    }

    /// Create recognition engine with custom configuration
    pub fn with_config(app_config: &AppConfig, recognition_config: RecognitionConfig) -> MathSeekResult<Self> {
        let api_client = ApiClient::from_app_config(app_config)?;
        
        Ok(Self {
            api_client,
            config: recognition_config,
        })
    }

    /// Recognize mathematical content from image data
    pub async fn recognize_content(&self, image_data: Vec<u8>, input_type: Option<InputType>) -> MathSeekResult<FormulaResult> {
        // Step 1: Validate image data
        if !ImageProcessor::validate_image(&image_data) {
            return Err(MathSeekError::ImageError("Invalid image data provided".to_string()));
        }

        // Step 2: Check if image is suitable for processing
        if !ImageProcessor::is_image_suitable_for_processing(&image_data)? {
            return Err(MathSeekError::ImageError("Image is not suitable for processing (too small, too large, or poor quality)".to_string()));
        }

        // Step 3: Preprocess image if enabled
        let processed_image = if self.config.preprocessing_enabled {
            ImageProcessor::preprocess_image(&image_data)?
        } else {
            image_data
        };

        // Step 4: Detect input type if not provided
        let detected_type = match input_type {
            Some(t) => t,
            None => {
                if self.config.auto_type_detection {
                    ImageProcessor::detect_input_type(&processed_image)?
                } else {
                    InputType::SingleFormula // Default fallback
                }
            }
        };

        // Step 5: Call API for recognition based on input type
        let mut result = match detected_type {
            InputType::SingleFormula => self.recognize_single_formula(&processed_image).await?,
            InputType::Document => self.recognize_document(&processed_image).await?,
        };

        // Step 6: Validate result if enabled
        if self.config.validation_enabled {
            self.validate_recognition_result(&mut result)?;
        }

        // Step 7: Check confidence threshold
        if result.confidence < self.config.confidence_threshold {
            return Err(MathSeekError::ApiError(format!(
                "Recognition confidence ({:.2}) below threshold ({:.2})",
                result.confidence, self.config.confidence_threshold
            )));
        }

        Ok(result)
    }

    /// Recognize a single mathematical formula
    async fn recognize_single_formula(&self, image_data: &[u8]) -> MathSeekResult<FormulaResult> {
        // Use API client to recognize the formula
        let result = self.api_client.recognize_image(image_data, InputType::SingleFormula).await?;
        
        // Ensure the result is properly formatted for single formula
        match result.content {
            ResultContent::SingleFormula(_) => Ok(result),
            ResultContent::Document(doc) => {
                // Convert document to single formula if it contains only one formula
                let latex = self.extract_single_formula_from_document(&doc)?;
                Ok(FormulaResult::new_single_formula(latex, result.confidence))
            }
        }
    }

    /// Recognize a document containing multiple formulas and text
    async fn recognize_document(&self, image_data: &[u8]) -> MathSeekResult<FormulaResult> {
        // First, analyze the image layout to understand structure
        let layout = ImageProcessor::analyze_image_layout(image_data)?;
        
        // Use API client to recognize the document
        let mut result = self.api_client.recognize_image(image_data, InputType::Document).await?;
        
        // Enhance the result with layout information
        if let ResultContent::Document(ref mut doc) = result.content {
            self.enhance_document_with_layout(doc, &layout)?;
        }
        
        Ok(result)
    }

    /// Extract a single formula from a document structure
    fn extract_single_formula_from_document(&self, doc: &DocumentContent) -> MathSeekResult<String> {
        // Look for the first formula in the document
        for section in &doc.sections {
            if !section.formulas.is_empty() {
                return Ok(section.formulas[0].latex.clone());
            }
            
            // If no explicit formulas, check if the text itself is a formula
            if section.text.trim().starts_with('$') || section.text.trim().starts_with("\\[") {
                return Ok(section.text.clone());
            }
        }
        
        Err(MathSeekError::ApiError("No formula found in document".to_string()))
    }

    /// Enhance document content with layout analysis information
    fn enhance_document_with_layout(&self, doc: &mut DocumentContent, layout: &crate::ImageLayout) -> MathSeekResult<()> {
        // Add metadata about the layout analysis
        if doc.sections.is_empty() {
            // Create a default section if none exists
            doc.sections.push(DocumentSection::new(None, "Recognized content".to_string()));
        }
        
        // Add information about detected regions
        let region_info = format!(
            "Detected {} formula regions and {} text regions",
            layout.formula_regions.len(),
            layout.text_regions.len()
        );
        
        // Add this as metadata to the first section
        if let Some(first_section) = doc.sections.first_mut() {
            if first_section.text.is_empty() {
                first_section.text = region_info;
            }
        }
        
        Ok(())
    }

    /// Validate and potentially correct recognition results
    fn validate_recognition_result(&self, result: &mut FormulaResult) -> MathSeekResult<()> {
        // Validate the result structure
        result.validate()?;
        
        // Additional validation based on input type
        match &result.content {
            ResultContent::SingleFormula(latex) => {
                self.validate_single_formula(latex)?;
            }
            ResultContent::Document(doc) => {
                self.validate_document_content(doc)?;
            }
        }
        
        // Normalize confidence to valid range
        if result.confidence > 1.0 {
            result.confidence = 1.0;
        } else if result.confidence < 0.0 {
            result.confidence = 0.0;
        }
        
        Ok(())
    }

    /// Validate single formula content
    fn validate_single_formula(&self, latex: &str) -> MathSeekResult<()> {
        if latex.trim().is_empty() {
            return Err(MathSeekError::ApiError("Empty formula content".to_string()));
        }
        
        // Basic LaTeX syntax validation
        if !self.is_valid_latex_syntax(latex) {
            return Err(MathSeekError::ApiError("Invalid LaTeX syntax detected".to_string()));
        }
        
        Ok(())
    }

    /// Validate document content structure
    fn validate_document_content(&self, doc: &DocumentContent) -> MathSeekResult<()> {
        doc.validate()?;
        
        // Additional document-specific validation
        for section in &doc.sections {
            for formula in &section.formulas {
                if !self.is_valid_latex_syntax(&formula.latex) {
                    return Err(MathSeekError::ApiError(format!(
                        "Invalid LaTeX syntax in formula: {}", formula.latex
                    )));
                }
            }
        }
        
        Ok(())
    }

    /// Basic LaTeX syntax validation
    fn is_valid_latex_syntax(&self, latex: &str) -> bool {
        let latex = latex.trim();
        
        // Check for balanced braces
        let mut brace_count = 0;
        let mut in_math_mode = false;
        
        for ch in latex.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        return false; // Unbalanced braces
                    }
                }
                '$' => in_math_mode = !in_math_mode,
                _ => {}
            }
        }
        
        // Check if braces are balanced and we're not in unclosed math mode
        brace_count == 0 && !in_math_mode
    }

    /// Re-recognize content with different parameters
    pub async fn re_recognize_with_type(&self, image_data: Vec<u8>, forced_type: InputType) -> MathSeekResult<FormulaResult> {
        self.recognize_content(image_data, Some(forced_type)).await
    }

    /// Get recognition statistics and metadata
    pub fn get_recognition_stats(&self) -> RecognitionStats {
        RecognitionStats {
            confidence_threshold: self.config.confidence_threshold,
            preprocessing_enabled: self.config.preprocessing_enabled,
            auto_type_detection: self.config.auto_type_detection,
            validation_enabled: self.config.validation_enabled,
        }
    }

    /// Update recognition configuration
    pub fn update_config(&mut self, new_config: RecognitionConfig) {
        self.config = new_config;
    }
}

/// Statistics and metadata about recognition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionStats {
    pub confidence_threshold: f32,
    pub preprocessing_enabled: bool,
    pub auto_type_detection: bool,
    pub validation_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;

    #[test]
    fn test_recognition_config_default() {
        let config = RecognitionConfig::default();
        assert_eq!(config.confidence_threshold, 0.5);
        assert!(config.preprocessing_enabled);
        assert!(config.auto_type_detection);
        assert!(config.validation_enabled);
    }

    #[test]
    fn test_latex_syntax_validation() {
        let app_config = AppConfig::default();
        let engine = RecognitionEngine::new(&app_config).unwrap();
        
        // Valid LaTeX
        assert!(engine.is_valid_latex_syntax("x^2 + y^2 = r^2"));
        assert!(engine.is_valid_latex_syntax("\\frac{a}{b}"));
        assert!(engine.is_valid_latex_syntax("$x + y$"));
        
        // Invalid LaTeX
        assert!(!engine.is_valid_latex_syntax("x^2 + y^2 = r^2}"));  // Unbalanced brace
        assert!(!engine.is_valid_latex_syntax("{x^2 + y^2 = r^2"));   // Unbalanced brace
        assert!(!engine.is_valid_latex_syntax("$x + y"));            // Unclosed math mode
    }

    #[test]
    fn test_recognition_stats() {
        let app_config = AppConfig::default();
        let engine = RecognitionEngine::new(&app_config).unwrap();
        
        let stats = engine.get_recognition_stats();
        assert_eq!(stats.confidence_threshold, 0.5);
        assert!(stats.preprocessing_enabled);
    }
}