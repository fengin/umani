use crate::db::Database;
use serde::Serialize;
use tauri::State;

/// 新手引导状态
#[derive(Debug, Serialize)]
pub struct OnboardingStatus {
    pub llm_configured: bool,
    pub has_skills: bool,
    pub has_articles: bool,
}

/// 获取引导状态（一次性返回所有检测项）
#[tauri::command]
pub fn get_onboarding_status(db: State<'_, Database>) -> Result<OnboardingStatus, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 检查 LLM 是否已配置（api_key 非空）
    let llm_configured: bool = conn
        .query_row(
            "SELECT CASE WHEN llm_api_key IS NOT NULL AND llm_api_key != '' THEN 1 ELSE 0 END FROM user_profile WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    // 检查是否有 Skill
    let skill_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM skill", [], |row| row.get(0))
        .unwrap_or(0);

    // 检查是否有文章
    let article_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM article", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(OnboardingStatus {
        llm_configured,
        has_skills: skill_count > 0,
        has_articles: article_count > 0,
    })
}
