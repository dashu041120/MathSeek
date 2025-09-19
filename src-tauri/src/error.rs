use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types for MathSeek application
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum MathSeekError {
    #[error("API调用失败: {0}")]
    ApiError(String),
    
    #[error("图像处理错误: {0}")]
    ImageError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("导出错误: {0}")]
    ExportError(String),
    
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("IO错误: {0}")]
    IoError(String),
    
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for MathSeekError {
    fn from(err: reqwest::Error) -> Self {
        MathSeekError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for MathSeekError {
    fn from(err: std::io::Error) -> Self {
        MathSeekError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for MathSeekError {
    fn from(err: serde_json::Error) -> Self {
        MathSeekError::SerializationError(err.to_string())
    }
}

impl From<image::ImageError> for MathSeekError {
    fn from(err: image::ImageError) -> Self {
        MathSeekError::ImageError(err.to_string())
    }
}

/// Result type alias for MathSeek operations
pub type MathSeekResult<T> = Result<T, MathSeekError>;