use crate::db::Database;
use crate::models::skill::{CreateSkillRequest, Skill, SkillVersion, UpdateSkillRequest};
use tauri::State;

/// 创建新 Skill（同时创建 v1 版本）
#[tauri::command]
pub fn create_skill(db: State<'_, Database>, request: CreateSkillRequest) -> Result<Skill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let category = request.category.unwrap_or_else(|| "通用".to_string());
    let description = request.description.unwrap_or_default();
    let content_md = request.content_markdown.unwrap_or_default();
    let content_json = request.content_json.unwrap_or_else(|| "{}".to_string());

    // 插入 Skill
    conn.execute(
        "INSERT INTO skill (name, category, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![request.name, category, description],
    )
    .map_err(|e| e.to_string())?;

    let skill_id = conn.last_insert_rowid();

    // 创建 v1 版本
    conn.execute(
        "INSERT INTO skill_version (skill_id, version_number, content_markdown, content_json, change_summary)
         VALUES (?1, 1, ?2, ?3, '初始版本')",
        rusqlite::params![skill_id, content_md, content_json],
    )
    .map_err(|e| e.to_string())?;

    // 返回创建的 Skill
    get_skill_by_id(&conn, skill_id)
}

/// 获取单个 Skill
#[tauri::command]
pub fn get_skill(db: State<'_, Database>, id: i64) -> Result<Skill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_skill_by_id(&conn, id)
}

/// 列出所有 Skill
#[tauri::command]
pub fn list_skills(db: State<'_, Database>) -> Result<Vec<Skill>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, description, current_version, created_at, updated_at
             FROM skill ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                current_version: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

/// 更新 Skill 元信息
#[tauri::command]
pub fn update_skill(
    db: State<'_, Database>,
    id: i64,
    request: UpdateSkillRequest,
) -> Result<Skill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // 先获取当前值用于合并
    let current = get_skill_by_id(&conn, id)?;

    let name = request.name.unwrap_or(current.name);
    let category = request.category.unwrap_or(current.category);
    let description = request.description.unwrap_or(current.description);

    conn.execute(
        "UPDATE skill SET name = ?1, category = ?2, description = ?3, updated_at = datetime('now') WHERE id = ?4",
        rusqlite::params![name, category, description, id],
    )
    .map_err(|e| e.to_string())?;

    get_skill_by_id(&conn, id)
}

/// 删除 Skill（级联删除版本和样本）
#[tauri::command]
pub fn delete_skill(db: State<'_, Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取 Skill 的所有版本
#[tauri::command]
pub fn get_skill_versions(
    db: State<'_, Database>,
    skill_id: i64,
) -> Result<Vec<SkillVersion>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, skill_id, version_number, content_markdown, content_json, change_summary, created_at
             FROM skill_version WHERE skill_id = ?1 ORDER BY version_number DESC",
        )
        .map_err(|e| e.to_string())?;

    let versions = stmt
        .query_map(rusqlite::params![skill_id], |row| {
            Ok(SkillVersion {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                version_number: row.get(2)?,
                content_markdown: row.get(3)?,
                content_json: row.get(4)?,
                change_summary: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(versions)
}

/// 获取特定版本
#[tauri::command]
pub fn get_skill_version(
    db: State<'_, Database>,
    skill_id: i64,
    version_number: i64,
) -> Result<SkillVersion, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, skill_id, version_number, content_markdown, content_json, change_summary, created_at
         FROM skill_version WHERE skill_id = ?1 AND version_number = ?2",
        rusqlite::params![skill_id, version_number],
        |row| {
            Ok(SkillVersion {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                version_number: row.get(2)?,
                content_markdown: row.get(3)?,
                content_json: row.get(4)?,
                change_summary: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("版本未找到: {}", e))
}

/// 内部辅助：按 ID 查询 Skill
fn get_skill_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Skill, String> {
    conn.query_row(
        "SELECT id, name, category, description, current_version, created_at, updated_at
         FROM skill WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                current_version: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Skill 未找到: {}", e))
}
