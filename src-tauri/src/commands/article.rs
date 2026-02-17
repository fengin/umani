use crate::db::Database;
use crate::models::article::Article;
use crate::services::llm_service::{self, ChatMessage, LlmConfig};
use crate::prompts;
use tauri::State;

/// 创建文章（AI 生成初稿）
#[tauri::command]
pub async fn generate_article(
    db: State<'_, Database>,
    skill_id: i64,
    topic: String,
) -> Result<Article, String> {
    // 1. 获取 Skill 当前版本内容
    let (skill_content, version_used, config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let skill_content: String = conn
            .query_row(
                "SELECT sv.content_markdown FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                rusqlite::params![skill_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("获取 Skill 失败: {}", e))?;

        let version_used: i64 = conn
            .query_row(
                "SELECT current_version FROM skill WHERE id = ?1",
                rusqlite::params![skill_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("获取版本号失败: {}", e))?;

        let config = conn
            .query_row(
                "SELECT llm_provider, llm_endpoint, llm_api_key, llm_model FROM user_profile WHERE id = 1",
                [],
                |row| {
                    Ok(LlmConfig {
                        provider: row.get(0)?,
                        endpoint: row.get(1)?,
                        api_key: row.get(2)?,
                        model: row.get(3)?,
                    })
                },
            )
            .map_err(|e| format!("读取 LLM 配置失败: {}", e))?;

        (skill_content, version_used, config)
    };

    // 2. 调用 LLM 生成文章
    let prompt = prompts::generate::build_generate_prompt(&skill_content, &topic);
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let ai_content = llm_service::chat_completion(&config, messages, 0.7).await?;

    // 3. 保存到数据库
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO article (title, ai_generated_content, skill_id, skill_version_used, status)
         VALUES (?1, ?2, ?3, ?4, 'editing')",
        rusqlite::params![topic, ai_content, skill_id, version_used],
    )
    .map_err(|e| e.to_string())?;

    let article_id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, title, original_content, ai_generated_content, user_refined_content,
                skill_id, skill_version_used, status, created_at, updated_at
         FROM article WHERE id = ?1",
        rusqlite::params![article_id],
        |row| {
            Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                original_content: row.get(2)?,
                ai_generated_content: row.get(3)?,
                user_refined_content: row.get(4)?,
                skill_id: row.get(5)?,
                skill_version_used: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| format!("查询文章失败: {}", e))
}

/// 保存用户修改后的文章内容
#[tauri::command]
pub fn save_article(
    db: State<'_, Database>,
    article_id: i64,
    content: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE article SET user_refined_content = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![content, article_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 列出文章
#[tauri::command]
pub fn list_articles(db: State<'_, Database>) -> Result<Vec<Article>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, original_content, ai_generated_content, user_refined_content,
                    skill_id, skill_version_used, status, created_at, updated_at
             FROM article ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let articles = stmt
        .query_map([], |row| {
            Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                original_content: row.get(2)?,
                ai_generated_content: row.get(3)?,
                user_refined_content: row.get(4)?,
                skill_id: row.get(5)?,
                skill_version_used: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(articles)
}
