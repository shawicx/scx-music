use serde::Serialize;
use thiserror::Error;

/// 统一后端错误类型。
///
/// 通过 `#[serde(tag = "type", content = "message")]` 序列化为
/// `{"type": "Database", "message": "..."}` 传到前端，前端可按 `type`
/// 区分错误类别（如 `InvalidArgument` → 用户输入错误，`Database` → 系统错误）。
///
/// 字段统一为 `String`（而非原始错误类型），因为 `std::io::Error` /
/// `rusqlite::Error` 等不实现 `Serialize`，转成字符串即可。
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("文件操作错误: {0}")]
    FileOperation(String),

    #[error("数据库错误: {0}")]
    Database(String),

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

// 保留常用类型的自动转换（让 ? 操作符直接工作）

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileOperation(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::OperationFailed(format!("JSON错误: {}", e))
    }
}

/// 让旧代码中残留的 `Result<T, String>` 通过 `?` 自动转换为 `AppResult<T>`，
/// String 包装为 `OperationFailed`。便于分批迁移期间编译通过。
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::OperationFailed(s)
    }
}

/// 将 AppError 转换为 String 以便向后兼容（旧前端代码读字符串）。
/// 注意：启用 serde::Serialize 后，Tauri IPC 会优先走 Serialize 路径
/// 把 AppError 作为结构化对象传给前端，此 impl 仅为内部兜底。
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converts_to_file_operation() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::FileOperation(_)));
    }

    #[test]
    fn invalid_argument_carries_message() {
        let err = AppError::InvalidArgument("bad key".to_string());
        assert!(err.to_string().contains("无效的参数"));
        assert!(err.to_string().contains("bad key"));
    }

    #[test]
    fn serializes_to_tagged_json() {
        let err = AppError::Database("lock failed".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"Database\""));
        assert!(json.contains("\"message\":\"lock failed\""));
    }

    #[test]
    fn converts_to_string_for_back_compat() {
        let err = AppError::InvalidArgument("test".to_string());
        let s: String = err.into();
        assert!(s.contains("无效的参数"));
    }

    #[test]
    fn string_converts_to_operation_failed() {
        let err: AppError = "something went wrong".to_string().into();
        assert!(matches!(err, AppError::OperationFailed(_)));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"type\":\"OperationFailed\""));
    }
}
