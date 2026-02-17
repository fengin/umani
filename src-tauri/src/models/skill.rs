use serde::{Deserialize, Serialize};

/// Skill 主结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub description: String,
    pub current_version: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Skill 版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersion {
    pub id: i64,
    pub skill_id: i64,
    pub version_number: i64,
    pub content_markdown: String,
    pub content_json: String,
    pub change_summary: String,
    pub created_at: String,
}

/// 原创样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginalSample {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub skill_id: i64,
    pub created_at: String,
}

/// 创建 Skill 请求
#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub content_markdown: Option<String>,
    pub content_json: Option<String>,
}

/// 更新 Skill 请求
#[derive(Debug, Deserialize)]
pub struct UpdateSkillRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}
