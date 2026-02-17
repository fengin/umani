use crate::db::Database;
use crate::models::article::DiffRecord;
use crate::services::llm_service::{self, ChatMessage, LlmConfig};
use crate::prompts;
use similar::{ChangeTag, TextDiff};
use tauri::State;

/// 计算两段文本的 Diff
#[tauri::command]
pub fn compute_diff(original: String, modified: String) -> Result<Vec<DiffChunk>, String> {
    let diff = TextDiff::from_lines(&original, &modified);
    let mut chunks = Vec::new();

    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Equal => "equal",
            ChangeTag::Delete => "delete",
            ChangeTag::Insert => "insert",
        };
        chunks.push(DiffChunk {
            tag: tag.to_string(),
            value: change.value().to_string(),
        });
    }

    Ok(chunks)
}

/// Diff 块
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffChunk {
    pub tag: String,   // "equal" | "delete" | "insert"
    pub value: String, // 文本内容
}

/// 分析 Diff 并提取规则（调用 LLM）
#[tauri::command]
pub async fn analyze_diff(
    db: State<'_, Database>,
    article_id: i64,
    original: String,
    modified: String,
) -> Result<DiffRecord, String> {
    // 1. 生成 diff 摘要
    let diff = TextDiff::from_lines(&original, &modified);
    let mut diff_summary = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Equal => " ",
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
        };
        diff_summary.push_str(&format!("{} {}", sign, change.value()));
    }

    // 2. 获取当前 Skill 内容和 LLM 配置
    let (current_skill, config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let skill_id: Option<i64> = conn
            .query_row(
                "SELECT skill_id FROM article WHERE id = ?1",
                rusqlite::params![article_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("获取文章失败: {}", e))?;

        let current_skill = match skill_id {
            Some(sid) => conn
                .query_row(
                    "SELECT sv.content_markdown FROM skill s
                     JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                     WHERE s.id = ?1",
                    rusqlite::params![sid],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or_default(),
            None => String::new(),
        };

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

        (current_skill, config)
    };

    // 3. 调用 LLM 分析 diff
    let prompt = prompts::diff_analyze::build_diff_analyze_prompt(
        &original,
        &modified,
        &diff_summary,
        &current_skill,
    );
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let analysis = llm_service::chat_completion(&config, messages, 0.3).await?;

    // 4. 保存到数据库
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO diff_record (article_id, diff_data, llm_analysis, extracted_rules)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![article_id, diff_summary, analysis, analysis],
    )
    .map_err(|e| e.to_string())?;

    let record_id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, article_id, diff_data, llm_analysis, extracted_rules, applied_to_skill, created_at
         FROM diff_record WHERE id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok(DiffRecord {
                id: row.get(0)?,
                article_id: row.get(1)?,
                diff_data: row.get(2)?,
                llm_analysis: row.get(3)?,
                extracted_rules: row.get(4)?,
                applied_to_skill: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("查询 Diff 记录失败: {}", e))
}

/// 将 Diff 分析结果应用到 Skill（创建新版本）
#[tauri::command]
pub fn evolve_skill(
    db: State<'_, Database>,
    skill_id: i64,
    new_content_markdown: String,
    new_content_json: String,
    change_summary: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 获取当前版本号
    let current_version: i64 = conn
        .query_row(
            "SELECT current_version FROM skill WHERE id = ?1",
            rusqlite::params![skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("获取 Skill 失败: {}", e))?;

    let new_version = current_version + 1;

    // 创建新版本
    conn.execute(
        "INSERT INTO skill_version (skill_id, version_number, content_markdown, content_json, change_summary)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![skill_id, new_version, new_content_markdown, new_content_json, change_summary],
    )
    .map_err(|e| e.to_string())?;

    // 更新 Skill 当前版本号
    conn.execute(
        "UPDATE skill SET current_version = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_version, skill_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
