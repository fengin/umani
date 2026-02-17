use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS user_profile (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            display_name    TEXT NOT NULL DEFAULT '默认用户',
            llm_provider    TEXT NOT NULL DEFAULT 'openai',
            llm_endpoint    TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
            llm_api_key     TEXT NOT NULL DEFAULT '',
            llm_model       TEXT NOT NULL DEFAULT 'gpt-4o',
            language        TEXT NOT NULL DEFAULT 'zh-CN',
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS skill (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL,
            category        TEXT NOT NULL DEFAULT '通用',
            description     TEXT NOT NULL DEFAULT '',
            current_version INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS skill_version (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            skill_id        INTEGER NOT NULL,
            version_number  INTEGER NOT NULL,
            content_markdown TEXT NOT NULL DEFAULT '',
            content_json    TEXT NOT NULL DEFAULT '{}',
            change_summary  TEXT NOT NULL DEFAULT '初始版本',
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (skill_id) REFERENCES skill(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS article (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            title               TEXT NOT NULL DEFAULT '未命名文章',
            original_content    TEXT NOT NULL DEFAULT '',
            ai_generated_content TEXT NOT NULL DEFAULT '',
            user_refined_content TEXT NOT NULL DEFAULT '',
            skill_id            INTEGER,
            skill_version_used  INTEGER,
            status              TEXT NOT NULL DEFAULT 'draft',
            created_at          TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (skill_id) REFERENCES skill(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS diff_record (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            article_id      INTEGER NOT NULL,
            diff_data       TEXT NOT NULL DEFAULT '',
            llm_analysis    TEXT NOT NULL DEFAULT '',
            extracted_rules TEXT NOT NULL DEFAULT '',
            applied_to_skill INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (article_id) REFERENCES article(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS original_sample (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            title       TEXT NOT NULL DEFAULT '未命名样本',
            content     TEXT NOT NULL DEFAULT '',
            skill_id    INTEGER NOT NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (skill_id) REFERENCES skill(id) ON DELETE CASCADE
        );

        -- 确保至少有一条用户配置记录
        INSERT OR IGNORE INTO user_profile (id, display_name) VALUES (1, '默认用户');
        ",
    )?;
    Ok(())
}
