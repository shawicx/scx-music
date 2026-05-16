use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("文件操作错误: {0}")]
    FileOperation(#[from] std::io::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("音频解析错误: {0}")]
    AudioParse(String),

    #[error("音频播放错误: {0}")]
    AudioPlayback(String),

    #[error("设备未找到: {0}")]
    DeviceNotFound(String),

    #[error("不支持的音频格式: {0}")]
    UnsupportedFormat(String),

    #[error("无效的参数: {0}")]
    InvalidArgument(String),

    #[error("操作失败: {0}")]
    OperationFailed(String),
}

// 将 AppError 转换为 String 以便 Tauri 传递
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

// 为 serde_json::Error 实现 From
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::OperationFailed(format!("JSON错误: {}", error))
    }
}

// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;