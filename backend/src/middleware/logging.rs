use salvo::http::StatusCode;
use salvo::logging::Logger;
use salvo::prelude::*;
use std::time::Instant;
use tracing::{info, warn};

/*
置好 tracing_subscriber 后，只需要调用以下宏就会自动记录日志：

宏	                    级别	用途
tracing::info!("...")	INFO	一般信息
tracing::warn!("...")	WARN	警告信息
tracing::error!("...")	ERROR	错误信息
tracing::debug!("...")	DEBUG	调试信息
例如：
tracing::info!("应用启动成功");
tracing::error!("数据库连接失败: {:?}", e);
tracing::warn!("请求超时: {}", path);

*/

pub fn logger() -> Logger {
    Logger::new()
}

#[handler]
pub async fn request_logger(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("");
    let full_path = if query.is_empty() {
        path.clone()
    } else {
        format!("{}?{}", path, query)
    };

    ctrl.call_next(req, _depot, res).await;

    let duration = start.elapsed();
    let status = res.status_code.map(|c| c.as_u16()).unwrap_or(200);

    if status >= 500 {
        warn!(
            method = %method,
            path = %full_path,
            status = status,
            duration_ms = duration.as_millis(),
            "Server error"
        );
    } else if status >= 400 {
        warn!(
            method = %method,
            path = %full_path,
            status = status,
            duration_ms = duration.as_millis(),
            "Client error"
        );
    } else {
        info!(
            method = %method,
            path = %full_path,
            status = status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );
    }
}

#[handler]
pub async fn slow_request_logger(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    ctrl.call_next(req, _depot, res).await;

    let duration = start.elapsed();
    let threshold_ms = 1000;

    if duration.as_millis() > threshold_ms {
        warn!(
            method = %method,
            path = %path,
            duration_ms = duration.as_millis(),
            threshold_ms = threshold_ms,
            "Slow request detected"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let _ = logger();
    }
}
