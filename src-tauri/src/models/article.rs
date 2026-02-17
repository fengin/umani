use serde::{Deserialize, Serialize};

/// 文章
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub original_content: String,
    pub ai_generated_content: String,
    pub user_refined_content: String,
    pub skill_id: Option<i64>,
    pub skill_version_used: Option<i64>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Diff 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffRecord {
    pub id: i64,
    pub article_id: i64,
    pub diff_data: String,
    pub llm_analysis: String,
    pub extracted_rules: String,
    pub applied_to_skill: bool,
    pub created_at: String,
}
