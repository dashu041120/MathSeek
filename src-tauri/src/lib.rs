use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod error;
pub use error::{MathSeekError, MathSeekResult};

pub mod image_processor;
pub use image_processor::ImageProcessor;

pub mod api_client;
pub use api_client::{ApiClient, ApiConfig};

pub mod config_manager;
pub use config_manager::{ConfigManager, ConfigValidation};

pub mod recognition_engine;
pub use recognition_engine::{RecognitionEngine, RecognitionConfig, RecognitionStats};

pub mod export_manager;
pub use export_manager::{ExportManager, ExportConfig, ExportResult, ExportMetadata};

#[cfg(test)]
mod models_test;

pub mod examples;

// Core data models for MathSeek

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum InputType {
    SingleFormula,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportFormat {
    LaTeX,
    LaTeXInline,
    LaTeXBlock,
    Markdown,
    MarkdownInline,
    MarkdownBlock,
    DOCX,
    HTML,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenderEngine {
    MathJax,
    KaTeX,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InlineFormat {
    Dollar,      // $...$
    Parentheses, // \(...\)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockFormat {
    DoubleDollar, // $$...$$
    Brackets,     // \[...\]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownFormulaFormat {
    pub inline: InlineFormat,
    pub block: BlockFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub default_export_format: HashMap<InputType, ExportFormat>,
    pub render_engine: RenderEngine,
    pub markdown_formula_format: MarkdownFormulaFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaResult {
    pub latex: String,
    pub confidence: f32,
    pub timestamp: u64,
    pub input_type: InputType,
    pub content: ResultContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultContent {
    SingleFormula(String),
    Document(DocumentContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub title: Option<String>,
    pub sections: Vec<DocumentSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub heading: Option<String>,
    pub text: String,
    pub formulas: Vec<FormulaBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaBlock {
    pub latex: String,
    pub position: usize,
    pub is_inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub formula_type: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayout {
    pub has_multiple_formulas: bool,
    pub has_text_content: bool,
    pub formula_regions: Vec<Region>,
    pub text_regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub clipboard_available: bool,
    pub screenshot_available: bool,
    pub api_configured: bool,
    pub render_engine_ready: bool,
}

// Tauri commands for MathSeek
#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// Image input commands
#[tauri::command]
async fn capture_screenshot() -> Result<String, String> {
    match ImageProcessor::capture_screenshot().await {
        Ok(image_data) => {
            ImageProcessor::image_to_base64(&image_data)
                .map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn get_clipboard_image() -> Result<Option<String>, String> {
    match ImageProcessor::get_clipboard_image().await {
        Ok(Some(image_data)) => {
            match ImageProcessor::image_to_base64(&image_data) {
                Ok(base64_str) => Ok(Some(base64_str)),
                Err(e) => Err(e.to_string())
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn validate_image_data(base64_data: String) -> Result<bool, String> {
    match ImageProcessor::base64_to_image(&base64_data) {
        Ok(image_data) => Ok(ImageProcessor::validate_image(&image_data)),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn preprocess_image(base64_data: String) -> Result<String, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let processed_data = ImageProcessor::preprocess_image(&image_data)
        .map_err(|e| e.to_string())?;
    
    ImageProcessor::image_to_base64(&processed_data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_image_info(base64_data: String) -> Result<serde_json::Value, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let (width, height) = ImageProcessor::get_image_dimensions(&image_data)
        .map_err(|e| e.to_string())?;
    
    let is_suitable = ImageProcessor::is_image_suitable_for_processing(&image_data)
        .map_err(|e| e.to_string())?;
    
    let info = serde_json::json!({
        "width": width,
        "height": height,
        "size": image_data.len(),
        "is_suitable": is_suitable
    });
    
    Ok(info)
}

// Input type detection commands
#[tauri::command]
async fn detect_input_type(base64_data: String) -> Result<String, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let input_type = ImageProcessor::detect_input_type(&image_data)
        .map_err(|e| e.to_string())?;
    
    Ok(input_type.into())
}

#[tauri::command]
async fn analyze_image_layout(base64_data: String) -> Result<ImageLayout, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    ImageProcessor::analyze_image_layout(&image_data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_detection_confidence(base64_data: String) -> Result<f32, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let layout = ImageProcessor::analyze_image_layout(&image_data)
        .map_err(|e| e.to_string())?;
    
    // Calculate confidence based on layout analysis
    let confidence = calculate_detection_confidence(&layout);
    Ok(confidence)
}

// Helper function to calculate detection confidence
fn calculate_detection_confidence(layout: &ImageLayout) -> f32 {
    let mut confidence: f32 = 0.5; // Base confidence
    
    // Increase confidence if we have clear indicators
    if layout.has_multiple_formulas {
        confidence += 0.3; // Strong indicator of document
    }
    
    if layout.has_text_content {
        confidence += 0.2; // Moderate indicator of document
    }
    
    // Adjust based on number of regions
    let total_regions = layout.formula_regions.len() + layout.text_regions.len();
    if total_regions > 3 {
        confidence += 0.1; // More regions = higher confidence in detection
    }
    
    // Ensure confidence is within valid range
    confidence.min(1.0).max(0.0)
}

#[tauri::command]
async fn check_system_status() -> Result<SystemStatus, String> {
    let status = SystemStatus {
        clipboard_available: true,
        screenshot_available: true,
        api_configured: false,
        render_engine_ready: true,
    };
    Ok(status)
}

#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.save_config(&config).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_config() -> Result<Option<AppConfig>, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.load_config().await
        .map_err(|e| e.to_string())
}

// API client commands
#[tauri::command]
async fn test_api_connection(config: AppConfig) -> Result<bool, String> {
    let api_client = ApiClient::from_app_config(&config)
        .map_err(|e| e.to_string())?;
    
    api_client.test_connection().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn recognize_formula(base64_data: String, input_type: String, config: AppConfig) -> Result<FormulaResult, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let input_type_enum = InputType::try_from(input_type)
        .map_err(|e| e.to_string())?;
    
    let recognition_engine = RecognitionEngine::new(&config)
        .map_err(|e| e.to_string())?;
    
    recognition_engine.recognize_content(image_data, Some(input_type_enum)).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn recognize_content_auto(base64_data: String, config: AppConfig) -> Result<FormulaResult, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let recognition_engine = RecognitionEngine::new(&config)
        .map_err(|e| e.to_string())?;
    
    recognition_engine.recognize_content(image_data, None).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn re_recognize_with_type(base64_data: String, forced_type: String, config: AppConfig) -> Result<FormulaResult, String> {
    let image_data = ImageProcessor::base64_to_image(&base64_data)
        .map_err(|e| e.to_string())?;
    
    let input_type_enum = InputType::try_from(forced_type)
        .map_err(|e| e.to_string())?;
    
    let recognition_engine = RecognitionEngine::new(&config)
        .map_err(|e| e.to_string())?;
    
    recognition_engine.re_recognize_with_type(image_data, input_type_enum).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recognition_stats(config: AppConfig) -> Result<RecognitionStats, String> {
    let recognition_engine = RecognitionEngine::new(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(recognition_engine.get_recognition_stats())
}

#[tauri::command]
async fn analyze_formula(formula: String, config: AppConfig) -> Result<AnalysisResult, String> {
    let api_client = ApiClient::from_app_config(&config)
        .map_err(|e| e.to_string())?;
    
    api_client.analyze_formula(&formula).await
        .map_err(|e| e.to_string())
}

// Configuration management commands
#[tauri::command]
async fn validate_config(config: AppConfig) -> Result<ConfigValidation, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.validate_config(&config).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn reset_config() -> Result<AppConfig, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.reset_config().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_config(include_sensitive: bool) -> Result<String, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.export_config(include_sensitive).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_config(config_json: String) -> Result<AppConfig, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.import_config(&config_json).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn config_exists() -> Result<bool, String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    Ok(config_manager.config_exists())
}

#[tauri::command]
async fn delete_config() -> Result<(), String> {
    let config_manager = ConfigManager::new()
        .map_err(|e| e.to_string())?;
    
    config_manager.delete_config().await
        .map_err(|e| e.to_string())
}

// Export management commands
#[tauri::command]
async fn export_formula_result(result: FormulaResult, export_config: ExportConfig, app_config: AppConfig) -> Result<ExportResult, String> {
    let export_manager = ExportManager::new(app_config);
    
    export_manager.export_formula_result(&result, &export_config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_available_export_formats(input_type: String, app_config: AppConfig) -> Result<Vec<String>, String> {
    let input_type_enum = InputType::try_from(input_type)
        .map_err(|e| e.to_string())?;
    
    let export_manager = ExportManager::new(app_config);
    let formats = export_manager.get_available_formats(&input_type_enum);
    
    Ok(formats.into_iter().map(|f| f.into()).collect())
}

#[tauri::command]
async fn get_default_export_format(input_type: String, app_config: AppConfig) -> Result<String, String> {
    let input_type_enum = InputType::try_from(input_type)
        .map_err(|e| e.to_string())?;
    
    let export_manager = ExportManager::new(app_config);
    let format = export_manager.get_default_format(&input_type_enum);
    
    Ok(format.into())
}

#[tauri::command]
async fn export_to_file(result: FormulaResult, export_config: ExportConfig, app_config: AppConfig, file_path: String) -> Result<(), String> {
    let export_manager = ExportManager::new(app_config);
    
    let export_result = export_manager.export_formula_result(&result, &export_config)
        .map_err(|e| e.to_string())?;
    
    std::fs::write(&file_path, export_result.content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

// Helper implementations
impl Default for AppConfig {
    fn default() -> Self {
        let mut default_formats = HashMap::new();
        default_formats.insert(InputType::SingleFormula, ExportFormat::LaTeX);
        default_formats.insert(InputType::Document, ExportFormat::Markdown);
        
        Self {
            api_endpoint: String::new(),
            api_key: String::new(),
            default_export_format: default_formats,
            render_engine: RenderEngine::MathJax,
            markdown_formula_format: MarkdownFormulaFormat::default(),
        }
    }
}

impl Default for MarkdownFormulaFormat {
    fn default() -> Self {
        Self {
            inline: InlineFormat::Dollar,
            block: BlockFormat::DoubleDollar,
        }
    }
}

impl FormulaResult {
    pub fn new_single_formula(latex: String, confidence: f32) -> Self {
        Self {
            latex: latex.clone(),
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            input_type: InputType::SingleFormula,
            content: ResultContent::SingleFormula(latex),
        }
    }
    
    pub fn new_document(latex: String, confidence: f32, document: DocumentContent) -> Self {
        Self {
            latex,
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            input_type: InputType::Document,
            content: ResultContent::Document(document),
        }
    }
}

impl DocumentContent {
    pub fn new(title: Option<String>) -> Self {
        Self {
            title,
            sections: Vec::new(),
        }
    }
    
    pub fn add_section(&mut self, section: DocumentSection) {
        self.sections.push(section);
    }
}

impl DocumentSection {
    pub fn new(heading: Option<String>, text: String) -> Self {
        Self {
            heading,
            text,
            formulas: Vec::new(),
        }
    }
    
    pub fn add_formula(&mut self, formula: FormulaBlock) {
        self.formulas.push(formula);
    }
}

impl FormulaBlock {
    pub fn new(latex: String, position: usize, is_inline: bool) -> Self {
        Self {
            latex,
            position,
            is_inline,
        }
    }
}

// Validation methods
impl AppConfig {
    pub fn validate(&self) -> MathSeekResult<()> {
        if self.api_endpoint.is_empty() {
            return Err(MathSeekError::ConfigError("API endpoint cannot be empty".to_string()));
        }
        
        if self.api_key.is_empty() {
            return Err(MathSeekError::ConfigError("API key cannot be empty".to_string()));
        }
        
        // Validate URL format
        if !self.api_endpoint.starts_with("http://") && !self.api_endpoint.starts_with("https://") {
            return Err(MathSeekError::ConfigError("API endpoint must be a valid URL".to_string()));
        }
        
        Ok(())
    }
}

impl FormulaResult {
    pub fn validate(&self) -> MathSeekResult<()> {
        if self.latex.is_empty() {
            return Err(MathSeekError::ApiError("LaTeX content cannot be empty".to_string()));
        }
        
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(MathSeekError::ApiError("Confidence must be between 0.0 and 1.0".to_string()));
        }
        
        Ok(())
    }
}

impl DocumentContent {
    pub fn validate(&self) -> MathSeekResult<()> {
        if self.sections.is_empty() {
            return Err(MathSeekError::ApiError("Document must have at least one section".to_string()));
        }
        
        for section in &self.sections {
            section.validate()?;
        }
        
        Ok(())
    }
}

impl DocumentSection {
    pub fn validate(&self) -> MathSeekResult<()> {
        if self.text.is_empty() && self.formulas.is_empty() {
            return Err(MathSeekError::ApiError("Section must have either text or formulas".to_string()));
        }
        
        Ok(())
    }
}

// Conversion methods for interoperability
impl From<InputType> for String {
    fn from(input_type: InputType) -> Self {
        match input_type {
            InputType::SingleFormula => "SingleFormula".to_string(),
            InputType::Document => "Document".to_string(),
        }
    }
}

impl TryFrom<String> for InputType {
    type Error = MathSeekError;
    
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "SingleFormula" => Ok(InputType::SingleFormula),
            "Document" => Ok(InputType::Document),
            _ => Err(MathSeekError::SerializationError(format!("Invalid InputType: {}", s))),
        }
    }
}

impl From<ExportFormat> for String {
    fn from(format: ExportFormat) -> Self {
        match format {
            ExportFormat::LaTeX => "LaTeX".to_string(),
            ExportFormat::LaTeXInline => "LaTeXInline".to_string(),
            ExportFormat::LaTeXBlock => "LaTeXBlock".to_string(),
            ExportFormat::Markdown => "Markdown".to_string(),
            ExportFormat::MarkdownInline => "MarkdownInline".to_string(),
            ExportFormat::MarkdownBlock => "MarkdownBlock".to_string(),
            ExportFormat::DOCX => "DOCX".to_string(),
            ExportFormat::HTML => "HTML".to_string(),
            ExportFormat::PlainText => "PlainText".to_string(),
        }
    }
}

impl TryFrom<String> for ExportFormat {
    type Error = MathSeekError;
    
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "LaTeX" => Ok(ExportFormat::LaTeX),
            "LaTeXInline" => Ok(ExportFormat::LaTeXInline),
            "LaTeXBlock" => Ok(ExportFormat::LaTeXBlock),
            "Markdown" => Ok(ExportFormat::Markdown),
            "MarkdownInline" => Ok(ExportFormat::MarkdownInline),
            "MarkdownBlock" => Ok(ExportFormat::MarkdownBlock),
            "DOCX" => Ok(ExportFormat::DOCX),
            "HTML" => Ok(ExportFormat::HTML),
            "PlainText" => Ok(ExportFormat::PlainText),
            _ => Err(MathSeekError::SerializationError(format!("Invalid ExportFormat: {}", s))),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            check_system_status,
            save_config,
            load_config,
            capture_screenshot,
            get_clipboard_image,
            validate_image_data,
            preprocess_image,
            get_image_info,
            detect_input_type,
            analyze_image_layout,
            get_detection_confidence,
            test_api_connection,
            recognize_formula,
            recognize_content_auto,
            re_recognize_with_type,
            get_recognition_stats,
            analyze_formula,
            validate_config,
            reset_config,
            export_config,
            import_config,
            config_exists,
            delete_config,
            export_formula_result,
            get_available_export_formats,
            get_default_export_format,
            export_to_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
