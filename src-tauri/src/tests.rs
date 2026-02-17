//! Savor 后端单元测试
//!
//! 使用内存 SQLite 数据库，覆盖以下功能：
//! - 数据库初始化与 Schema 创建
//! - Skill CRUD（创建/读取/更新/删除）
//! - 版本管理（创建版本、进化版本、版本查询）
//! - 导出功能（Markdown / JSON）
//! - Diff 计算（文本差异计算）

#[cfg(test)]
mod tests {
    use crate::db::schema;
    use rusqlite::Connection;

    /// 创建内存数据库并初始化 Schema
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        schema::create_tables(&conn).expect("创建表失败");
        conn
    }

    // ========== 数据库初始化测试 ==========

    #[test]
    fn test_schema_creation() {
        let conn = setup_db();

        // 验证所有 6 张表都已创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(
            tables.contains(&"user_profile".to_string()),
            "缺少 user_profile 表"
        );
        assert!(tables.contains(&"skill".to_string()), "缺少 skill 表");
        assert!(
            tables.contains(&"skill_version".to_string()),
            "缺少 skill_version 表"
        );
        assert!(tables.contains(&"article".to_string()), "缺少 article 表");
        assert!(
            tables.contains(&"diff_record".to_string()),
            "缺少 diff_record 表"
        );
        assert!(
            tables.contains(&"original_sample".to_string()),
            "缺少 original_sample 表"
        );
    }

    #[test]
    fn test_default_user_profile() {
        let conn = setup_db();

        let (name, provider, model): (String, String, String) = conn
            .query_row(
                "SELECT display_name, llm_provider, llm_model FROM user_profile WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("默认用户记录应存在");

        assert_eq!(name, "默认用户");
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_schema_idempotent() {
        let conn = setup_db();
        // 重复调用不应出错
        schema::create_tables(&conn).expect("第二次创建表应成功（IF NOT EXISTS）");
    }

    // ========== Skill CRUD 测试 ==========

    /// 辅助：插入一个 Skill 并返回 ID
    fn insert_skill(conn: &Connection, name: &str, category: &str, desc: &str) -> i64 {
        conn.execute(
            "INSERT INTO skill (name, category, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![name, category, desc],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// 辅助：插入 Skill 版本
    fn insert_version(
        conn: &Connection,
        skill_id: i64,
        ver: i64,
        md: &str,
        json: &str,
        summary: &str,
    ) {
        conn.execute(
            "INSERT INTO skill_version (skill_id, version_number, content_markdown, content_json, change_summary)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![skill_id, ver, md, json, summary],
        ).unwrap();
    }

    #[test]
    fn test_create_skill() {
        let conn = setup_db();
        let id = insert_skill(&conn, "科技评论", "科技", "犀利的科技评论风格");

        let name: String = conn
            .query_row("SELECT name FROM skill WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "科技评论");
    }

    #[test]
    fn test_list_skills() {
        let conn = setup_db();
        insert_skill(&conn, "Skill A", "通用", "");
        insert_skill(&conn, "Skill B", "科技", "");
        insert_skill(&conn, "Skill C", "生活", "");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_update_skill() {
        let conn = setup_db();
        let id = insert_skill(&conn, "旧名称", "通用", "旧描述");

        conn.execute(
            "UPDATE skill SET name = ?1, description = ?2, updated_at = datetime('now') WHERE id = ?3",
            rusqlite::params!["新名称", "新描述", id],
        ).unwrap();

        let (name, desc): (String, String) = conn
            .query_row(
                "SELECT name, description FROM skill WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "新名称");
        assert_eq!(desc, "新描述");
    }

    #[test]
    fn test_delete_skill_cascade() {
        let conn = setup_db();
        let id = insert_skill(&conn, "待删除", "通用", "");
        insert_version(&conn, id, 1, "# v1", "{}", "初始版本");

        // 确认版本存在
        let ver_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_version WHERE skill_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ver_count, 1);

        // 删除 Skill
        conn.execute("DELETE FROM skill WHERE id = ?1", [id])
            .unwrap();

        // Skill 已删除
        let skill_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(skill_count, 0);

        // 级联删除：版本也应消失
        let ver_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_version WHERE skill_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ver_count_after, 0);
    }

    #[test]
    fn test_get_nonexistent_skill() {
        let conn = setup_db();
        let result = conn.query_row("SELECT id FROM skill WHERE id = 99999", [], |row| {
            row.get::<_, i64>(0)
        });
        assert!(result.is_err(), "查询不存在的 Skill 应返回错误");
    }

    // ========== 版本管理测试 ==========

    #[test]
    fn test_create_version() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "测试 Skill", "通用", "");
        insert_version(
            &conn,
            skill_id,
            1,
            "# 版本1",
            "{\"tone\": \"neutral\"}",
            "初始版本",
        );

        let (md, json): (String, String) = conn
            .query_row(
                "SELECT content_markdown, content_json FROM skill_version WHERE skill_id = ?1 AND version_number = 1",
                [skill_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(md, "# 版本1");
        assert_eq!(json, "{\"tone\": \"neutral\"}");
    }

    #[test]
    fn test_evolve_version() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "进化测试", "通用", "");
        insert_version(&conn, skill_id, 1, "# v1", "{}", "初始版本");

        // 模拟进化：创建 v2
        let new_version = 2i64;
        insert_version(
            &conn,
            skill_id,
            new_version,
            "# v2 进化后",
            "{\"rules\": [\"简洁\"]}",
            "用户修改了段落结构",
        );

        // 更新 current_version
        conn.execute(
            "UPDATE skill SET current_version = ?1 WHERE id = ?2",
            rusqlite::params![new_version, skill_id],
        )
        .unwrap();

        let current: i64 = conn
            .query_row(
                "SELECT current_version FROM skill WHERE id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 2);

        // 确认有 2 个版本
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_version WHERE skill_id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_specific_version() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "版本查询", "通用", "");
        insert_version(&conn, skill_id, 1, "# v1", "{}", "初始");
        insert_version(&conn, skill_id, 2, "# v2", "{}", "进化");

        let summary: String = conn
            .query_row(
                "SELECT change_summary FROM skill_version WHERE skill_id = ?1 AND version_number = 2",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(summary, "进化");
    }

    #[test]
    fn test_versions_ordered_desc() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "排序测试", "通用", "");
        insert_version(&conn, skill_id, 1, "", "{}", "v1");
        insert_version(&conn, skill_id, 2, "", "{}", "v2");
        insert_version(&conn, skill_id, 3, "", "{}", "v3");

        let mut stmt = conn
            .prepare("SELECT version_number FROM skill_version WHERE skill_id = ?1 ORDER BY version_number DESC")
            .unwrap();
        let versions: Vec<i64> = stmt
            .query_map([skill_id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(versions, vec![3, 2, 1]);
    }

    // ========== 导出功能测试 ==========

    #[test]
    fn test_export_markdown() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "我的风格", "科技", "犀利科技评论");
        insert_version(
            &conn,
            skill_id,
            1,
            "## 核心规则\n- 简洁有力\n- 避免陈词滥调",
            "{}",
            "初始",
        );

        // 模拟导出逻辑 (与 commands/export.rs 一致)
        let (name, category, description): (String, String, String) = conn
            .query_row(
                "SELECT name, category, description FROM skill WHERE id = ?1",
                [skill_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        let content: String = conn
            .query_row(
                "SELECT sv.content_markdown FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();

        let markdown = format!(
            "# {} — Writing Style Skill\n\n**分类**: {} | **版本**: v{}\n\n{}\n\n---\n\n{}\n\n---\n\n> 由 Savor (余香) 导出 | 可直接作为 System Prompt 使用\n",
            name, category, 1, description, content
        );

        assert!(markdown.contains("我的风格 — Writing Style Skill"));
        assert!(markdown.contains("**分类**: 科技"));
        assert!(markdown.contains("简洁有力"));
        assert!(markdown.contains("Savor (余香) 导出"));
    }

    #[test]
    fn test_export_json() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "JSON测试", "通用", "测试用");
        insert_version(&conn, skill_id, 1, "", "{\"tone\":\"formal\"}", "初始");

        let content_json: String = conn
            .query_row(
                "SELECT sv.content_json FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();

        let export = serde_json::json!({
            "name": "JSON测试",
            "category": "通用",
            "version": 1,
            "skill": serde_json::from_str::<serde_json::Value>(&content_json).unwrap_or(serde_json::Value::Null),
            "exported_by": "Savor (余香)"
        });

        let json_str = serde_json::to_string_pretty(&export).unwrap();
        assert!(json_str.contains("\"name\": \"JSON测试\""));
        assert!(json_str.contains("\"tone\": \"formal\""));
        assert!(json_str.contains("Savor (余香)"));
    }

    // ========== Diff 计算测试 ==========

    #[test]
    fn test_diff_identical() {
        use similar::{ChangeTag, TextDiff};

        let text = "Hello\nWorld\n";
        let diff = TextDiff::from_lines(text, text);
        let changes: Vec<_> = diff.iter_all_changes().collect();

        for change in &changes {
            assert_eq!(change.tag(), ChangeTag::Equal);
        }
    }

    #[test]
    fn test_diff_insert() {
        use similar::{ChangeTag, TextDiff};

        let original = "Hello\nWorld\n";
        let modified = "Hello\nBeautiful\nWorld\n";
        let diff = TextDiff::from_lines(original, modified);

        let tags: Vec<ChangeTag> = diff.iter_all_changes().map(|c| c.tag()).collect();
        assert!(tags.contains(&ChangeTag::Insert), "应检测到插入操作");
    }

    #[test]
    fn test_diff_delete() {
        use similar::{ChangeTag, TextDiff};

        let original = "Hello\nWorld\nFoo\n";
        let modified = "Hello\nFoo\n";
        let diff = TextDiff::from_lines(original, modified);

        let tags: Vec<ChangeTag> = diff.iter_all_changes().map(|c| c.tag()).collect();
        assert!(tags.contains(&ChangeTag::Delete), "应检测到删除操作");
    }

    #[test]
    fn test_diff_replace() {
        use similar::{ChangeTag, TextDiff};

        let original = "旧的段落内容\n";
        let modified = "新的段落内容，修改后更简洁\n";
        let diff = TextDiff::from_lines(original, modified);

        let tags: Vec<ChangeTag> = diff.iter_all_changes().map(|c| c.tag()).collect();
        assert!(tags.contains(&ChangeTag::Delete), "替换应包含删除");
        assert!(tags.contains(&ChangeTag::Insert), "替换应包含插入");
    }

    #[test]
    fn test_diff_empty() {
        use similar::TextDiff;

        let diff = TextDiff::from_lines("", "");
        let changes: Vec<_> = diff.iter_all_changes().collect();
        assert!(changes.is_empty(), "两个空字符串的 diff 应为空");
    }

    // ========== 文章数据库测试 ==========

    #[test]
    fn test_article_crud() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "测试 Skill", "通用", "");

        // 创建文章
        conn.execute(
            "INSERT INTO article (skill_id, skill_version_used, title, ai_generated_content, user_refined_content)
             VALUES (?1, 1, '人工智能趋势', '# AI 趋势\n\nAI 正在...', '# AI 趋势\n\nAI 正在...')",
            [skill_id],
        ).unwrap();

        let article_id = conn.last_insert_rowid();

        // 修改文章
        conn.execute(
            "UPDATE article SET user_refined_content = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params!["# AI 趋势\n\n人工智能正在改变世界...", article_id],
        ).unwrap();

        let modified: String = conn
            .query_row(
                "SELECT user_refined_content FROM article WHERE id = ?1",
                [article_id],
                |row| row.get(0),
            )
            .unwrap();

        assert!(modified.contains("人工智能正在改变世界"));
    }

    #[test]
    fn test_list_articles_by_skill() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "Skill A", "通用", "");

        for i in 1..=3 {
            conn.execute(
                "INSERT INTO article (skill_id, skill_version_used, title, ai_generated_content, user_refined_content)
                 VALUES (?1, 1, ?2, '', '')",
                rusqlite::params![skill_id, format!("话题{}", i)],
            ).unwrap();
        }

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM article WHERE skill_id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    // ========== LLM 配置测试 ==========

    #[test]
    fn test_update_llm_config() {
        let conn = setup_db();

        conn.execute(
            "UPDATE user_profile SET llm_provider = ?1, llm_endpoint = ?2, llm_api_key = ?3, llm_model = ?4 WHERE id = 1",
            rusqlite::params!["deepseek", "https://api.deepseek.com/v1", "sk-test-key", "deepseek-chat"],
        ).unwrap();

        let (provider, endpoint, model): (String, String, String) = conn
            .query_row(
                "SELECT llm_provider, llm_endpoint, llm_model FROM user_profile WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(provider, "deepseek");
        assert_eq!(endpoint, "https://api.deepseek.com/v1");
        assert_eq!(model, "deepseek-chat");
    }

    // ================================================================
    // ========== 边界条件 & 异常分支测试 ==========
    // ================================================================

    // ---------- Skill 边界测试 ----------

    #[test]
    fn test_create_skill_with_special_characters() {
        let conn = setup_db();
        // SQL 注入风格的名称应安全存储
        let id = insert_skill(&conn, "Skill'; DROP TABLE skill;--", "test", "desc");
        let name: String = conn
            .query_row("SELECT name FROM skill WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "Skill'; DROP TABLE skill;--");
    }

    #[test]
    fn test_create_skill_with_unicode_emoji() {
        let conn = setup_db();
        let id = insert_skill(&conn, "🎨 创意写作 ✍️", "🌟 艺术", "包含 emoji 的描述 💡");
        let (name, cat, desc): (String, String, String) = conn
            .query_row(
                "SELECT name, category, description FROM skill WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(name, "🎨 创意写作 ✍️");
        assert_eq!(cat, "🌟 艺术");
        assert!(desc.contains("💡"));
    }

    #[test]
    fn test_create_skill_with_very_long_name() {
        let conn = setup_db();
        let long_name = "A".repeat(10000);
        let id = insert_skill(&conn, &long_name, "通用", "");
        let name: String = conn
            .query_row("SELECT name FROM skill WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name.len(), 10000);
    }

    #[test]
    fn test_create_skill_with_empty_name() {
        let conn = setup_db();
        // schema 允许空字符串 (NOT NULL 但没有 CHECK 长度)
        let id = insert_skill(&conn, "", "通用", "");
        let name: String = conn
            .query_row("SELECT name FROM skill WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "");
    }

    #[test]
    fn test_update_nonexistent_skill() {
        let conn = setup_db();
        let affected = conn
            .execute("UPDATE skill SET name = 'x' WHERE id = 99999", [])
            .unwrap();
        assert_eq!(affected, 0, "更新不存在的 Skill 应影响 0 行");
    }

    #[test]
    fn test_delete_nonexistent_skill() {
        let conn = setup_db();
        let affected = conn
            .execute("DELETE FROM skill WHERE id = 99999", [])
            .unwrap();
        assert_eq!(affected, 0, "删除不存在的 Skill 应影响 0 行");
    }

    #[test]
    fn test_create_duplicate_name_skills() {
        let conn = setup_db();
        // schema 没有 UNIQUE 约束，允许重名
        let id1 = insert_skill(&conn, "相同名称", "通用", "描述1");
        let id2 = insert_skill(&conn, "相同名称", "通用", "描述2");
        assert_ne!(id1, id2, "重名 Skill 应有不同 ID");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill WHERE name = '相同名称'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_skill_default_values() {
        let conn = setup_db();
        // 只提供 name，其余用默认值
        conn.execute("INSERT INTO skill (name) VALUES ('仅名称')", [])
            .unwrap();
        let id = conn.last_insert_rowid();

        let (cat, desc, ver): (String, String, i64) = conn
            .query_row(
                "SELECT category, description, current_version FROM skill WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(cat, "通用", "默认分类应为'通用'");
        assert_eq!(desc, "", "默认描述应为空字符串");
        assert_eq!(ver, 1, "默认版本号应为 1");
    }

    // ---------- 版本管理边界测试 ----------

    #[test]
    fn test_version_for_nonexistent_skill() {
        let conn = setup_db();
        // FK 约束应阻止为不存在的 skill 创建版本
        let result = conn.execute(
            "INSERT INTO skill_version (skill_id, version_number, content_markdown, content_json, change_summary)
             VALUES (99999, 1, '', '{}', 'test')",
            [],
        );
        assert!(
            result.is_err(),
            "为不存在的 Skill 创建版本应失败（FK 约束）"
        );
    }

    #[test]
    fn test_get_nonexistent_version() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "测试", "通用", "");
        insert_version(&conn, skill_id, 1, "", "{}", "v1");

        let result = conn.query_row(
            "SELECT id FROM skill_version WHERE skill_id = ?1 AND version_number = 99",
            [skill_id],
            |row| row.get::<_, i64>(0),
        );
        assert!(result.is_err(), "查询不存在的版本号应返回错误");
    }

    #[test]
    fn test_version_with_large_content() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "大内容", "通用", "");
        let large_md = "# 测试\n".to_string() + &"这是一段很长的文本。\n".repeat(5000);
        let large_json_content = format!(
            "{{\"rules\": [{}]}}",
            (0..1000)
                .map(|i| format!("\"rule_{}\"", i))
                .collect::<Vec<_>>()
                .join(",")
        );

        insert_version(
            &conn,
            skill_id,
            1,
            &large_md,
            &large_json_content,
            "大内容测试",
        );

        let (md, json): (String, String) = conn
            .query_row(
                "SELECT content_markdown, content_json FROM skill_version WHERE skill_id = ?1 AND version_number = 1",
                [skill_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!(md.len() > 50000, "大 Markdown 应完整存储");
        assert!(json.contains("rule_999"), "大 JSON 应完整存储");
    }

    #[test]
    fn test_multiple_rapid_evolutions() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "快速进化", "通用", "");

        // 连续创建 10 个版本
        for v in 1..=10 {
            insert_version(
                &conn,
                skill_id,
                v,
                &format!("# v{}", v),
                "{}",
                &format!("第 {} 次进化", v),
            );
            conn.execute(
                "UPDATE skill SET current_version = ?1 WHERE id = ?2",
                rusqlite::params![v, skill_id],
            )
            .unwrap();
        }

        let current: i64 = conn
            .query_row(
                "SELECT current_version FROM skill WHERE id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 10);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_version WHERE skill_id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 10);
    }

    // ---------- 导出边界测试 ----------

    #[test]
    fn test_export_nonexistent_skill() {
        let conn = setup_db();
        let result = conn.query_row("SELECT name FROM skill WHERE id = 99999", [], |row| {
            row.get::<_, String>(0)
        });
        assert!(result.is_err(), "导出不存在的 Skill 应失败");
    }

    #[test]
    fn test_export_skill_with_empty_content() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "空内容", "通用", "");
        insert_version(&conn, skill_id, 1, "", "{}", "初始");

        let content: String = conn
            .query_row(
                "SELECT sv.content_markdown FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(content, "", "空内容应正常导出为空字符串");
    }

    #[test]
    fn test_export_json_with_invalid_json_content() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "坏JSON", "通用", "");
        insert_version(&conn, skill_id, 1, "", "这不是JSON", "初始");

        let content_json: String = conn
            .query_row(
                "SELECT sv.content_json FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();

        // 模拟导出逻辑中的 fallback
        let parsed = serde_json::from_str::<serde_json::Value>(&content_json)
            .unwrap_or(serde_json::Value::Null);
        assert_eq!(
            parsed,
            serde_json::Value::Null,
            "无效 JSON 应 fallback 为 Null"
        );
    }

    #[test]
    fn test_export_after_evolve() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "进化导出", "通用", "");
        insert_version(&conn, skill_id, 1, "# v1 旧内容", "{}", "初始");
        insert_version(
            &conn,
            skill_id,
            2,
            "# v2 新内容",
            "{\"evolved\": true}",
            "进化",
        );
        conn.execute(
            "UPDATE skill SET current_version = 2 WHERE id = ?1",
            [skill_id],
        )
        .unwrap();

        // 导出应使用 current_version (v2) 的内容
        let content: String = conn
            .query_row(
                "SELECT sv.content_markdown FROM skill s
                 JOIN skill_version sv ON sv.skill_id = s.id AND sv.version_number = s.current_version
                 WHERE s.id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(content.contains("v2 新内容"), "导出应使用最新版本");
        assert!(!content.contains("v1 旧内容"), "不应包含旧版本内容");
    }

    // ---------- Diff 边界测试 ----------

    #[test]
    fn test_diff_chinese_text() {
        use similar::{ChangeTag, TextDiff};

        let original = "今天天气不错\n我们去公园散步吧\n";
        let modified = "今天天气真好\n我们去公园散步吧\n一起看日落\n";
        let diff = TextDiff::from_lines(original, modified);

        let tags: Vec<ChangeTag> = diff.iter_all_changes().map(|c| c.tag()).collect();
        assert!(tags.contains(&ChangeTag::Delete), "中文替换应检测到删除");
        assert!(tags.contains(&ChangeTag::Insert), "中文替换应检测到插入");
        assert!(tags.contains(&ChangeTag::Equal), "不变行应标记为 Equal");
    }

    #[test]
    fn test_diff_from_empty() {
        use similar::{ChangeTag, TextDiff};

        let original = "";
        let modified = "新增内容\n第二行\n";
        let diff = TextDiff::from_lines(original, modified);

        let all_insert = diff
            .iter_all_changes()
            .all(|c| c.tag() == ChangeTag::Insert);
        assert!(all_insert, "从空到有内容，所有变更应为 Insert");
    }

    #[test]
    fn test_diff_to_empty() {
        use similar::{ChangeTag, TextDiff};

        let original = "原始内容\n第二行\n";
        let modified = "";
        let diff = TextDiff::from_lines(original, modified);

        let all_delete = diff
            .iter_all_changes()
            .all(|c| c.tag() == ChangeTag::Delete);
        assert!(all_delete, "从有内容到空，所有变更应为 Delete");
    }

    #[test]
    fn test_diff_single_char_change() {
        use similar::TextDiff;

        let original = "abcdefg\n";
        let modified = "abcXefg\n";
        let diff = TextDiff::from_lines(original, modified);
        let changes: Vec<_> = diff.iter_all_changes().collect();
        // 行级 diff：整行是一个变更
        assert!(!changes.is_empty(), "单字符变更应被检测到");
    }

    #[test]
    fn test_diff_multiline_large() {
        use similar::TextDiff;

        // 100 行原文，每隔 10 行修改一行
        let mut original_lines = Vec::new();
        let mut modified_lines = Vec::new();
        for i in 0..100 {
            original_lines.push(format!("第 {} 行原始内容", i));
            if i % 10 == 5 {
                modified_lines.push(format!("第 {} 行【已修改】", i));
            } else {
                modified_lines.push(format!("第 {} 行原始内容", i));
            }
        }
        let original = original_lines.join("\n") + "\n";
        let modified = modified_lines.join("\n") + "\n";

        let diff = TextDiff::from_lines(&original, &modified);
        let change_count = diff
            .iter_all_changes()
            .filter(|c| c.tag() != similar::ChangeTag::Equal)
            .count();
        // 10 行修改 = 10 delete + 10 insert = 20
        assert_eq!(change_count, 20, "应检测到 10 处单行替换（20 个变更块）");
    }

    // ---------- 级联删除深度测试 ----------

    #[test]
    fn test_delete_skill_cascades_articles_and_samples() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "级联全覆盖", "通用", "");
        insert_version(&conn, skill_id, 1, "", "{}", "v1");

        // 创建文章
        conn.execute(
            "INSERT INTO article (skill_id, skill_version_used, title) VALUES (?1, 1, '测试文章')",
            [skill_id],
        )
        .unwrap();

        // 创建原始样本
        conn.execute(
            "INSERT INTO original_sample (skill_id, title, content) VALUES (?1, '样本1', '内容')",
            [skill_id],
        )
        .unwrap();

        // 删除 Skill
        conn.execute("DELETE FROM skill WHERE id = ?1", [skill_id])
            .unwrap();

        // 验证级联删除
        let version_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_version WHERE skill_id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();
        let sample_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM original_sample WHERE skill_id = ?1",
                [skill_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(version_count, 0, "版本应被级联删除");
        assert_eq!(sample_count, 0, "样本应被级联删除");

        // article 的 FK 是 ON DELETE SET NULL，不会被删除
        let article_skill: Option<i64> = conn
            .query_row(
                "SELECT skill_id FROM article WHERE title = '测试文章'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(article_skill, None, "文章的 skill_id 应被置为 NULL");
    }

    // ---------- 文章边界测试 ----------

    #[test]
    fn test_article_without_skill() {
        let conn = setup_db();
        // article 的 skill_id 允许 NULL
        conn.execute(
            "INSERT INTO article (title, ai_generated_content, user_refined_content) VALUES ('独立文章', '内容', '内容')",
            [],
        ).unwrap();

        let article_id = conn.last_insert_rowid();
        let skill_id: Option<i64> = conn
            .query_row(
                "SELECT skill_id FROM article WHERE id = ?1",
                [article_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(skill_id, None, "无 Skill 关联的文章 skill_id 应为 NULL");
    }

    #[test]
    fn test_article_default_status() {
        let conn = setup_db();
        conn.execute("INSERT INTO article (title) VALUES ('状态测试')", [])
            .unwrap();

        let status: String = conn
            .query_row(
                "SELECT status FROM article WHERE title = '状态测试'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "draft", "文章默认状态应为 draft");
    }

    #[test]
    fn test_article_with_multiline_content() {
        let conn = setup_db();
        let content = "# 标题\n\n## 第一章\n\n这是第一段。\n\n## 第二章\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n> 引用内容\n";
        conn.execute(
            "INSERT INTO article (title, ai_generated_content) VALUES ('多行测试', ?1)",
            [content],
        )
        .unwrap();

        let stored: String = conn
            .query_row(
                "SELECT ai_generated_content FROM article WHERE title = '多行测试'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, content, "含代码块和特殊语法的多行内容应完整存储");
    }

    // ---------- LLM 配置边界测试 ----------

    #[test]
    fn test_llm_config_empty_api_key() {
        let conn = setup_db();
        conn.execute("UPDATE user_profile SET llm_api_key = '' WHERE id = 1", [])
            .unwrap();

        let key: String = conn
            .query_row(
                "SELECT llm_api_key FROM user_profile WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(key, "", "空 API Key 应保存为空字符串");
    }

    #[test]
    fn test_llm_config_preserve_unmodified_fields() {
        let conn = setup_db();
        // 只更新 provider，其余字段不应变
        conn.execute(
            "UPDATE user_profile SET llm_provider = 'anthropic' WHERE id = 1",
            [],
        )
        .unwrap();

        let (provider, endpoint, model): (String, String, String) = conn
            .query_row(
                "SELECT llm_provider, llm_endpoint, llm_model FROM user_profile WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(provider, "anthropic", "Provider 应被更新");
        assert_eq!(
            endpoint, "https://api.openai.com/v1",
            "未修改的 endpoint 应保持默认值"
        );
        assert_eq!(model, "gpt-4o", "未修改的 model 应保持默认值");
    }

    // ---------- Diff Record 数据库测试 ----------

    #[test]
    fn test_diff_record_fk_constraint() {
        let conn = setup_db();
        // diff_record 的 article_id 是必填的 FK
        let result = conn.execute(
            "INSERT INTO diff_record (article_id, diff_data, llm_analysis) VALUES (99999, 'data', 'analysis')",
            [],
        );
        assert!(
            result.is_err(),
            "为不存在的 article 创建 diff_record 应失败（FK 约束）"
        );
    }

    #[test]
    fn test_delete_article_cascades_diff_records() {
        let conn = setup_db();
        let skill_id = insert_skill(&conn, "测试", "通用", "");
        conn.execute(
            "INSERT INTO article (skill_id, skill_version_used, title) VALUES (?1, 1, '删除测试')",
            [skill_id],
        )
        .unwrap();
        let article_id = conn.last_insert_rowid();

        // 创建 diff 记录
        conn.execute(
            "INSERT INTO diff_record (article_id, diff_data, llm_analysis) VALUES (?1, 'diff', 'analysis')",
            [article_id],
        ).unwrap();

        // 删除文章
        conn.execute("DELETE FROM article WHERE id = ?1", [article_id])
            .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM diff_record WHERE article_id = ?1",
                [article_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "删除文章应级联删除 diff_record");
    }
}
