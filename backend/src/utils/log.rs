use std::path::PathBuf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Registry};

// 初始化日志：控制台 + 文件双输出
pub fn init_logger() -> WorkerGuard {
    // 1. 创建按天滚动的日志文件（目录、文件名前缀）
    let log_dir = PathBuf::from("logs");
    std::fs::create_dir_all(&log_dir).expect("无法创建日志目录");
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "app.log");

    // 2. 非阻塞写入（不阻塞业务线程）
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // 3. 文件输出 Layer
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_level(true)
        .with_target(true);

    // 4. 控制台输出 Layer
    // ============================================================
    // 生产环境：仅显示 INFO 及以上级别（默认）
    // 调试时开启 DEBUG：注释掉下面的代码，取消注释后面的代码
    // ============================================================
    // 【生产环境用】只显示 INFO 及以上级别
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_level(true)
        .with_target(true)
        .with_filter(LevelFilter::INFO);

    // 【调试用】显示所有级别（包括 DEBUG）
    // 要开启 DEBUG 日志，取消注释以下代码，并注释掉上面的代码：
    // let console_layer = fmt::layer()
    //     .with_writer(std::io::stdout)
    //     .with_level(true)
    //     .with_target(true);
    // ============================================================

    // 5. 组合两层并初始化
    Registry::default()
        .with(console_layer)
        .with(file_layer)
        .init();

    // 必须返回 guard，否则日志会直接关闭
    guard
}
