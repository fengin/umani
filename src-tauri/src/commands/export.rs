use crate::db::Database;
use tauri::State;

/// 导出 Skill 为 Markdown 格式
#[tauri::command]
pub fn export_skill_markdown(db: State<'_, Database>, skill_id: i64) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (name, category, description): (String, String, String) = conn
        .query_row(
            "SELECT name, category, description FROM skill WHERE id = ?1",
            rusqlite::params![skill_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Skill 未找到: {}", e))?;

    let content: String = conn
        .query_row(
            "SELECT sv.content_markdown FROM skill s
             JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
             WHERE s.id = ?1",
            rusqlite::params![skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("获取版本内容失败: {}", e))?;

    let version: i64 = conn
        .query_row(
            "SELECT current_version FROM skill WHERE id = ?1",
            rusqlite::params![skill_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let markdown = format!(
        "# {} — Writing Style Skill\n\n**分类**: {} | **版本**: v{}\n\n{}\n\n---\n\n{}\n\n---\n\n> 由 Savor (余香) 导出 | 可直接作为 System Prompt 使用\n",
        name, category, version, description, content
    );

    Ok(markdown)
}

/// 导出 Skill 为 JSON 格式
#[tauri::command]
pub fn export_skill_json(db: State<'_, Database>, skill_id: i64) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (name, category, description, version): (String, String, String, i64) = conn
        .query_row(
            "SELECT name, category, description, current_version FROM skill WHERE id = ?1",
            rusqlite::params![skill_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("Skill 未找到: {}", e))?;

    let content_json: String = conn
        .query_row(
            "SELECT sv.content_json FROM skill s
             JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
             WHERE s.id = ?1",
            rusqlite::params![skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("获取版本内容失败: {}", e))?;

    let export = serde_json::json!({
        "name": name,
        "category": category,
        "description": description,
        "version": version,
        "skill": serde_json::from_str::<serde_json::Value>(&content_json).unwrap_or(serde_json::Value::Null),
        "exported_by": "Savor (余香)"
    });

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}
