/// 写作生成提示词：根据 Skill + 主题生成符合风格的文章初稿
pub fn build_generate_prompt(skill_content: &str, topic: &str) -> String {
    format!(
        r#"你是一位专业代笔作家。请严格按照以下 Writing Style Skill 的要求来写作。

## Writing Style Skill

{}

---

## 写作任务

请根据以上风格规范，围绕以下主题撰写一篇文章：

**主题：** {}

要求：
1. 严格遵循 Skill 中定义的语气、身份和风格原则
2. 绝对避免禁止清单中的词汇、句式和结构
3. 使用作者惯用的术语和表达方式
4. 保持作者的真实声音，不要有"AI味"
5. 内容要有深度和观点，不要停留在表面

直接输出文章正文，不要添加额外的说明或元信息。"#,
        skill_content, topic
    )
}
