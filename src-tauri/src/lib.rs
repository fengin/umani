mod commands;
mod db;
mod models;
mod prompts;
mod services;

#[cfg(test)]
mod tests;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 获取应用数据目录，初始化数据库
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");

            let database = Database::new(&app_data_dir).expect("数据库初始化失败");

            // 将 Database 注册为 Tauri 全局状态
            app.manage(database);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Skill CRUD
            commands::skill::create_skill,
            commands::skill::get_skill,
            commands::skill::list_skills,
            commands::skill::update_skill,
            commands::skill::delete_skill,
            commands::skill::get_skill_versions,
            commands::skill::get_skill_version,
            // LLM
            commands::llm::save_llm_config,
            commands::llm::get_llm_config,
            commands::llm::test_llm_connection,
            // Article
            commands::article::generate_article,
            commands::article::save_article,
            commands::article::get_article,
            commands::article::list_articles,
            // Diff
            commands::diff::compute_diff,
            commands::diff::analyze_diff,
            commands::diff::evolve_skill,
            // Export
            commands::export::export_skill_markdown,
            commands::export::export_skill_json,
            // Onboarding
            commands::onboarding::get_onboarding_status,
            commands::skill::create_skill_with_samples,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
