use crate::db::Database;
use crate::services::llm_service::{self, LlmConfig};
use tauri::State;

/// 保存 LLM 配置
#[tauri::command]
pub fn save_llm_config(
    db: State<'_, Database>,
    provider: String,
    endpoint: String,
    api_key: String,
    model: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE user_profile SET llm_provider = ?1, llm_endpoint = ?2, llm_api_key = ?3, llm_model = ?4, updated_at = datetime('now') WHERE id = 1",
        rusqlite::params![provider, endpoint, api_key, model],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取 LLM 配置
#[tauri::command]
pub fn get_llm_config(db: State<'_, Database>) -> Result<LlmConfig, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.query_row(
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
    .map_err(|e| format!("读取配置失败: {}", e))
}

/// 测试 LLM 连接
#[tauri::command]
pub async fn test_llm_connection(
    db: State<'_, Database>,
) -> Result<String, String> {
    let config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
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
        .map_err(|e| format!("读取配置失败: {}", e))?
    };

    llm_service::test_connection(&config).await
}
