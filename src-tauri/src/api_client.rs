use crate::{MathSeekError, MathSeekResult, AppConfig, FormulaResult, AnalysisResult, InputType, ResultContent, DocumentContent};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use base64::prelude::*;

/// Configuration for API client
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub endpoint: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: String::new(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl From<&AppConfig> for ApiConfig {
    fn from(app_config: &AppConfig) -> Self {
        Self {
            endpoint: app_config.api_endpoint.clone(),
            api_key: app_config.api_key.clone(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Request payload for formula recognition
#[derive(Debug, Serialize)]
struct RecognitionRequest {
    image_data: String,
    input_type: String,
    options: RecognitionOptions,
}

#[derive(Debug, Serialize)]
struct RecognitionOptions {
    output_format: String,
    confidence_threshold: f32,
}

/// Response from formula recognition API
#[derive(Debug, Serialize, Deserialize)]
struct RecognitionResponse {
    success: bool,
    latex: Option<String>,
    confidence: Option<f32>,
    content: Option<serde_json::Value>,
    error: Option<String>,
}

/// Request payload for formula analysis
#[derive(Debug, Serialize)]
struct AnalysisRequest {
    formula: String,
    analysis_type: String,
}

/// Response from formula analysis API
#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResponse {
    success: bool,
    formula_type: Option<String>,
    description: Option<String>,
    usage: Option<String>,
    examples: Option<Vec<String>>,
    error: Option<String>,
}

/// HTTP client for interacting with large language model APIs
pub struct ApiClient {
    client: Client,
    config: ApiConfig,
}

impl ApiClient {
    /// Create a new API client with the given configuration
    pub fn new(config: ApiConfig) -> MathSeekResult<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("MathSeek/1.0")
            .build()
            .map_err(|e| MathSeekError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Create API client from app configuration
    pub fn from_app_config(app_config: &AppConfig) -> MathSeekResult<Self> {
        let api_config = ApiConfig::from(app_config);
        Self::new(api_config)
    }

    /// Test API connection and authentication
    pub async fn test_connection(&self) -> MathSeekResult<bool> {
        if self.config.endpoint.is_empty() || self.config.api_key.is_empty() {
            return Err(MathSeekError::ConfigError("API endpoint or key not configured".to_string()));
        }

        let test_url = format!("{}/health", self.config.endpoint);
        
        let response = self.client
            .get(&test_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Recognize mathematical formulas from image data
    pub async fn recognize_image(&self, image_data: &[u8], input_type: InputType) -> MathSeekResult<FormulaResult> {
        let base64_image = base64::prelude::BASE64_STANDARD.encode(image_data);
        
        let request = RecognitionRequest {
            image_data: base64_image,
            input_type: input_type.clone().into(),
            options: RecognitionOptions {
                output_format: "latex".to_string(),
                confidence_threshold: 0.5,
            },
        };

        let response = self.make_request_with_retry("/recognize", &request).await?;
        self.parse_recognition_response(response, input_type).await
    }

    /// Analyze a mathematical formula to get type and description
    pub async fn analyze_formula(&self, formula: &str) -> MathSeekResult<AnalysisResult> {
        let request = AnalysisRequest {
            formula: formula.to_string(),
            analysis_type: "comprehensive".to_string(),
        };

        let response = self.make_request_with_retry("/analyze", &request).await?;
        self.parse_analysis_response(response).await
    }

    /// Make HTTP request with retry logic
    async fn make_request_with_retry<T: Serialize>(&self, endpoint: &str, payload: &T) -> MathSeekResult<RecognitionResponse> {
        let url = format!("{}{}", self.config.endpoint, endpoint);
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                sleep(Duration::from_millis(self.config.retry_delay_ms * attempt as u64)).await;
            }

            match self.make_single_request(&url, payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't retry on authentication or client errors
                    if let Some(MathSeekError::ApiError(ref msg)) = last_error {
                        if msg.contains("401") || msg.contains("403") || msg.contains("400") {
                            break;
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| MathSeekError::NetworkError("Max retries exceeded".to_string())))
    }

    /// Make a single HTTP request
    async fn make_single_request<T: Serialize>(&self, url: &str, payload: &T) -> MathSeekResult<RecognitionResponse> {
        let request_future = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(payload)
            .send();

        let response = timeout(Duration::from_secs(self.config.timeout_seconds), request_future)
            .await
            .map_err(|_| MathSeekError::NetworkError("Request timeout".to_string()))?
            .map_err(|e| MathSeekError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(MathSeekError::ApiError(format!(
                "API request failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let api_response: RecognitionResponse = response
            .json()
            .await
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to parse response: {}", e)))?;

        if !api_response.success {
            return Err(MathSeekError::ApiError(
                api_response.error.unwrap_or_else(|| "Unknown API error".to_string())
            ));
        }

        Ok(api_response)
    }

    /// Parse recognition API response into FormulaResult
    async fn parse_recognition_response(&self, response: RecognitionResponse, input_type: InputType) -> MathSeekResult<FormulaResult> {
        let latex = response.latex.ok_or_else(|| {
            MathSeekError::ApiError("No LaTeX content in response".to_string())
        })?;

        let confidence = response.confidence.unwrap_or(0.0);

        let content = match input_type {
            InputType::SingleFormula => ResultContent::SingleFormula(latex.clone()),
            InputType::Document => {
                // Try to parse document content from response
                if let Some(content_value) = response.content {
                    match serde_json::from_value::<DocumentContent>(content_value) {
                        Ok(doc_content) => ResultContent::Document(doc_content),
                        Err(_) => {
                            // Fallback: create simple document with single section
                            let mut doc = DocumentContent::new(None);
                            doc.add_section(crate::DocumentSection::new(None, latex.clone()));
                            ResultContent::Document(doc)
                        }
                    }
                } else {
                    // Create simple document structure
                    let mut doc = DocumentContent::new(None);
                    doc.add_section(crate::DocumentSection::new(None, latex.clone()));
                    ResultContent::Document(doc)
                }
            }
        };

        let result = FormulaResult {
            latex,
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            input_type,
            content,
        };

        result.validate()?;
        Ok(result)
    }

    /// Parse analysis API response into AnalysisResult
    async fn parse_analysis_response(&self, response: RecognitionResponse) -> MathSeekResult<AnalysisResult> {
        // For analysis, we need to parse the response differently
        // This is a simplified implementation - in practice, you'd have a separate AnalysisResponse type
        let analysis_response: AnalysisResponse = serde_json::from_str(&serde_json::to_string(&response)?)
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to parse analysis response: {}", e)))?;

        if !analysis_response.success {
            return Err(MathSeekError::ApiError(
                analysis_response.error.unwrap_or_else(|| "Analysis failed".to_string())
            ));
        }

        Ok(AnalysisResult {
            formula_type: analysis_response.formula_type.unwrap_or_else(|| "Unknown".to_string()),
            description: analysis_response.description.unwrap_or_else(|| "No description available".to_string()),
            usage: analysis_response.usage.unwrap_or_else(|| "No usage information available".to_string()),
            examples: analysis_response.examples.unwrap_or_default(),
        })
    }

    /// Update API configuration
    pub fn update_config(&mut self, config: ApiConfig) -> MathSeekResult<()> {
        // Validate new configuration
        if config.endpoint.is_empty() {
            return Err(MathSeekError::ConfigError("API endpoint cannot be empty".to_string()));
        }
        
        if config.api_key.is_empty() {
            return Err(MathSeekError::ConfigError("API key cannot be empty".to_string()));
        }

        // Create new client with updated timeout
        self.client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("MathSeek/1.0")
            .build()
            .map_err(|e| MathSeekError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        self.config = config;
        Ok(())
    }

    /// Get current API configuration (without sensitive data)
    pub fn get_config_info(&self) -> serde_json::Value {
        serde_json::json!({
            "endpoint": self.config.endpoint,
            "timeout_seconds": self.config.timeout_seconds,
            "max_retries": self.config.max_retries,
            "retry_delay_ms": self.config.retry_delay_ms,
            "has_api_key": !self.config.api_key.is_empty()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_creation() {
        let config = ApiConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
    }

    #[test]
    fn test_api_config_from_app_config() {
        let mut app_config = AppConfig::default();
        app_config.api_endpoint = "https://api.example.com".to_string();
        app_config.api_key = "test-key".to_string();

        let api_config = ApiConfig::from(&app_config);
        assert_eq!(api_config.endpoint, "https://api.example.com");
        assert_eq!(api_config.api_key, "test-key");
    }

    #[tokio::test]
    async fn test_api_client_creation() {
        let config = ApiConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let client = ApiClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_recognition_request_serialization() {
        let request = RecognitionRequest {
            image_data: "base64data".to_string(),
            input_type: "SingleFormula".to_string(),
            options: RecognitionOptions {
                output_format: "latex".to_string(),
                confidence_threshold: 0.5,
            },
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }

    #[test]
    fn test_analysis_request_serialization() {
        let request = AnalysisRequest {
            formula: "x^2 + y^2 = r^2".to_string(),
            analysis_type: "comprehensive".to_string(),
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }
}