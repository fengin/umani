/// Diff 分析提示词：分析用户修改意图并提取新的风格规则
pub fn build_diff_analyze_prompt(
    original: &str,
    modified: &str,
    diff_summary: &str,
    current_skill: &str,
) -> String {
    format!(
        r#"你是一位写作风格分析专家。用户在 AI 生成的文章基础上进行了手动修改，请分析这些修改背后的写作偏好和风格规则。

## AI 原始生成内容

{}

## 用户修改后的内容

{}

## Diff 变更摘要

{}

## 当前 Writing Style Skill

{}

---

请分析用户的修改意图，并输出以下 JSON 格式（不要添加 markdown 代码块标记）：

{{
  "modification_analysis": [
    {{
      "type": "词汇替换 | 句式调整 | 结构重组 | 内容增删 | 语气变化",
      "description": "具体修改描述",
      "intent": "推测的修改意图"
    }}
  ],
  "new_rules": {{
    "add_to_style_principles": ["应新增的风格原则"],
    "add_to_blocklist_words": ["应新增的禁用词"],
    "add_to_blocklist_patterns": ["应新增的禁用句式"],
    "other_observations": ["其他观察到的风格偏好"]
  }},
  "summary": "一句话总结本次修改对 Skill 的改进方向"
}}

分析要求：
1. 关注系统性的偏好，而非一次性的内容修正
2. 区分"内容性修改"（不影响 Skill）和"风格性修改"（应纳入 Skill）
3. 新规则应具体可执行，避免笼统描述
4. 如果修改很少或无风格意义，new_rules 可以为空数组"#,
        original, modified, diff_summary, current_skill
    )
}
