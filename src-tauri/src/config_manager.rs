use crate::{MathSeekError, MathSeekResult, AppConfig, ApiClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use base64::prelude::*;

/// Configuration manager for secure storage and loading of app settings
pub struct ConfigManager {
    config_path: PathBuf,
}

/// Configuration validation result
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub api_connection_ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> MathSeekResult<Self> {
        let config_dir = Self::get_config_directory()?;
        
        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| MathSeekError::IoError(format!("Failed to create config directory: {}", e)))?;
        }

        let config_path = config_dir.join("config.json");

        Ok(Self { config_path })
    }

    /// Get the application configuration directory
    fn get_config_directory() -> MathSeekResult<PathBuf> {
        // Use a platform-specific config directory
        let config_dir = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("MathSeek")
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("Library")
                .join("Application Support")
                .join("MathSeek")
        } else {
            // Linux and other Unix-like systems
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    std::env::var("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(".config")
                })
                .join("mathseek")
        };
        
        Ok(config_dir)
    }

    /// Save configuration to secure storage
    pub async fn save_config(&self, config: &AppConfig) -> MathSeekResult<()> {
        // Validate configuration before saving
        config.validate()?;

        // Encrypt sensitive data (API key)
        let encrypted_config = self.encrypt_sensitive_data(config)?;

        // Serialize and save
        let config_json = serde_json::to_string_pretty(&encrypted_config)
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&self.config_path, config_json)
            .map_err(|e| MathSeekError::IoError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Load configuration from storage
    pub async fn load_config(&self) -> MathSeekResult<Option<AppConfig>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let config_content = fs::read_to_string(&self.config_path)
            .map_err(|e| MathSeekError::IoError(format!("Failed to read config file: {}", e)))?;

        let encrypted_config: EncryptedAppConfig = serde_json::from_str(&config_content)
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to parse config: {}", e)))?;

        // Decrypt sensitive data
        let config = self.decrypt_sensitive_data(&encrypted_config)?;
        
        // Validate loaded configuration
        config.validate()?;

        Ok(Some(config))
    }

    /// Validate configuration and test API connection
    pub async fn validate_config(&self, config: &AppConfig) -> MathSeekResult<ConfigValidation> {
        let mut validation = ConfigValidation {
            is_valid: true,
            api_connection_ok: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Basic validation
        match config.validate() {
            Ok(_) => {},
            Err(e) => {
                validation.is_valid = false;
                validation.errors.push(e.to_string());
            }
        }

        // Test API connection if basic validation passes
        if validation.is_valid {
            match ApiClient::from_app_config(config) {
                Ok(client) => {
                    match client.test_connection().await {
                        Ok(true) => {
                            validation.api_connection_ok = true;
                        },
                        Ok(false) => {
                            validation.warnings.push("API connection test failed".to_string());
                        },
                        Err(e) => {
                            validation.warnings.push(format!("API connection error: {}", e));
                        }
                    }
                },
                Err(e) => {
                    validation.errors.push(format!("Failed to create API client: {}", e));
                    validation.is_valid = false;
                }
            }
        }

        Ok(validation)
    }

    /// Reset configuration to default values
    pub async fn reset_config(&self) -> MathSeekResult<AppConfig> {
        let default_config = AppConfig::default();
        self.save_config(&default_config).await?;
        Ok(default_config)
    }

    /// Export configuration (without sensitive data) for backup
    pub async fn export_config(&self, include_sensitive: bool) -> MathSeekResult<String> {
        let config = self.load_config().await?
            .ok_or_else(|| MathSeekError::ConfigError("No configuration found".to_string()))?;

        let export_config = if include_sensitive {
            config
        } else {
            AppConfig {
                api_key: "***REDACTED***".to_string(),
                ..config
            }
        };

        serde_json::to_string_pretty(&export_config)
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to export config: {}", e)))
    }

    /// Import configuration from JSON string
    pub async fn import_config(&self, config_json: &str) -> MathSeekResult<AppConfig> {
        let config: AppConfig = serde_json::from_str(config_json)
            .map_err(|e| MathSeekError::SerializationError(format!("Failed to parse imported config: {}", e)))?;

        // Validate imported configuration
        config.validate()?;

        // Save the imported configuration
        self.save_config(&config).await?;

        Ok(config)
    }

    /// Get configuration file path for debugging
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }

    /// Delete configuration file
    pub async fn delete_config(&self) -> MathSeekResult<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| MathSeekError::IoError(format!("Failed to delete config file: {}", e)))?;
        }
        Ok(())
    }

    /// Simple encryption for API key (in production, use proper encryption)
    fn encrypt_sensitive_data(&self, config: &AppConfig) -> MathSeekResult<EncryptedAppConfig> {
        // For now, we'll use base64 encoding as a simple obfuscation
        // In production, you should use proper encryption like AES
        let encrypted_api_key = BASE64_STANDARD.encode(&config.api_key);

        Ok(EncryptedAppConfig {
            api_endpoint: config.api_endpoint.clone(),
            encrypted_api_key,
            default_export_format: config.default_export_format.clone(),
            render_engine: config.render_engine.clone(),
            markdown_formula_format: config.markdown_formula_format.clone(),
        })
    }

    /// Simple decryption for API key
    fn decrypt_sensitive_data(&self, encrypted_config: &EncryptedAppConfig) -> MathSeekResult<AppConfig> {
        let api_key = String::from_utf8(
            BASE64_STANDARD.decode(&encrypted_config.encrypted_api_key)
                .map_err(|e| MathSeekError::ConfigError(format!("Failed to decrypt API key: {}", e)))?
        ).map_err(|e| MathSeekError::ConfigError(format!("Invalid API key format: {}", e)))?;

        Ok(AppConfig {
            api_endpoint: encrypted_config.api_endpoint.clone(),
            api_key,
            default_export_format: encrypted_config.default_export_format.clone(),
            render_engine: encrypted_config.render_engine.clone(),
            markdown_formula_format: encrypted_config.markdown_formula_format.clone(),
        })
    }
}

/// Encrypted version of AppConfig for secure storage
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedAppConfig {
    pub api_endpoint: String,
    pub encrypted_api_key: String,
    pub default_export_format: std::collections::HashMap<crate::InputType, crate::ExportFormat>,
    pub render_engine: crate::RenderEngine,
    pub markdown_formula_format: crate::MarkdownFormulaFormat,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> AppConfig {
        let mut default_formats = HashMap::new();
        default_formats.insert(crate::InputType::SingleFormula, crate::ExportFormat::LaTeX);
        default_formats.insert(crate::InputType::Document, crate::ExportFormat::Markdown);

        AppConfig {
            api_endpoint: "https://api.example.com".to_string(),
            api_key: "test-api-key".to_string(),
            default_export_format: default_formats,
            render_engine: crate::RenderEngine::MathJax,
            markdown_formula_format: crate::MarkdownFormulaFormat::default(),
        }
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_sensitive_data() {
        let manager = ConfigManager::new().unwrap();
        let config = create_test_config();

        let encrypted = manager.encrypt_sensitive_data(&config).unwrap();
        assert_ne!(encrypted.encrypted_api_key, config.api_key);

        let decrypted = manager.decrypt_sensitive_data(&encrypted).unwrap();
        assert_eq!(decrypted.api_key, config.api_key);
        assert_eq!(decrypted.api_endpoint, config.api_endpoint);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let manager = ConfigManager::new().unwrap();
        let config = create_test_config();

        // This will fail because we're not actually connecting to an API
        let validation = manager.validate_config(&config).await.unwrap();
        assert!(validation.errors.is_empty()); // Basic validation should pass
    }

    #[test]
    fn test_config_export_import() {
        let config = create_test_config();
        
        let exported = serde_json::to_string(&config).unwrap();
        let imported: AppConfig = serde_json::from_str(&exported).unwrap();
        
        assert_eq!(config.api_endpoint, imported.api_endpoint);
        assert_eq!(config.api_key, imported.api_key);
    }
}