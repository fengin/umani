// ===== Skill 相关 =====

export interface Skill {
    id: number;
    name: string;
    category: string;
    description: string;
    current_version: number;
    created_at: string;
    updated_at: string;
}

export interface SkillVersion {
    id: number;
    skill_id: number;
    version_number: number;
    content_markdown: string;
    content_json: string;
    change_summary: string;
    created_at: string;
}

export interface CreateSkillRequest {
    name: string;
    category?: string;
    description?: string;
    content_markdown?: string;
    content_json?: string;
}

export interface UpdateSkillRequest {
    name?: string;
    category?: string;
    description?: string;
}

// ===== Article 相关 =====

export interface Article {
    id: number;
    title: string;
    original_content: string;
    ai_generated_content: string;
    user_refined_content: string;
    skill_id: number | null;
    skill_version_used: number | null;
    status: 'draft' | 'editing' | 'published';
    created_at: string;
    updated_at: string;
}

export interface DiffRecord {
    id: number;
    article_id: number;
    diff_data: string;
    llm_analysis: string;
    extracted_rules: string;
    applied_to_skill: boolean;
    created_at: string;
}

// ===== LLM 配置 =====

export interface LlmConfig {
    provider: string;
    endpoint: string;
    api_key: string;
    model: string;
}

export type LlmProvider = 'openai' | 'claude' | 'deepseek' | 'ollama' | 'custom';

export const LLM_PROVIDERS: { value: LlmProvider; label: string; defaultEndpoint: string; models: string[] }[] = [
    {
        value: 'openai',
        label: 'OpenAI',
        defaultEndpoint: 'https://api.openai.com/v1',
        models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
    },
    {
        value: 'claude',
        label: 'Claude',
        defaultEndpoint: 'https://api.anthropic.com/v1',
        models: ['claude-sonnet-4-20250514', 'claude-3-5-haiku-20241022'],
    },
    {
        value: 'deepseek',
        label: 'DeepSeek',
        defaultEndpoint: 'https://api.deepseek.com/v1',
        models: ['deepseek-chat', 'deepseek-reasoner'],
    },
    {
        value: 'ollama',
        label: 'Ollama (本地)',
        defaultEndpoint: 'http://localhost:11434/v1',
        models: ['llama3', 'mistral', 'qwen2'],
    },
    {
        value: 'custom',
        label: '自定义端点',
        defaultEndpoint: '',
        models: [],
    },
];
