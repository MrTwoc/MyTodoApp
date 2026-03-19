use serde::{Deserialize, Serialize};

/*
    用户个人设置结构体:
    mode: 使用模式 - 单机模式(Single)或在线模式(Online)
    theme: 主题 - 暗色主题(Dark)或亮色主题(Light)
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppMode {
    Single, // 单机模式 - 本地存储，不与服务器交互
    Online, // 在线模式 - 支持团队协作，任务、团队、个人数据存储在服务器，切换团队时需要登录
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,  // 暗色主题
    Light, // 亮色主题
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub mode: AppMode,
    pub theme: Theme,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            mode: AppMode::Single, // 默认为单机模式
            theme: Theme::Dark,    // 默认为暗色主题
        }
    }
}
