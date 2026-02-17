import { useState, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import Editor from '@monaco-editor/react';
import { skillApi } from '../services/skillApi';
import { articleApi } from '../services/articleApi';
import type { Skill, Article } from '../types';
import './Editor.css';

export default function EditorPage() {
    const { t } = useTranslation();

    // 状态
    const [skills, setSkills] = useState<Skill[]>([]);
    const [selectedSkillId, setSelectedSkillId] = useState<number | null>(null);
    const [topic, setTopic] = useState('');
    const [article, setArticle] = useState<Article | null>(null);
    const [aiContent, setAiContent] = useState('');
    const [userContent, setUserContent] = useState('');
    const [loading, setLoading] = useState(false);
    const [statusMsg, setStatusMsg] = useState('');
    const editorRef = useRef<unknown>(null);

    // 加载 Skills
    const loadSkills = useCallback(async () => {
        try {
            const list = await skillApi.list();
            setSkills(list);
        } catch (e) {
            console.error('加载 Skills 失败:', e);
        }
    }, []);

    // 首次加载
    useState(() => {
        loadSkills();
    });

    // AI 生成文章
    const handleGenerate = async () => {
        if (!selectedSkillId || !topic.trim()) {
            setStatusMsg('请选择 Skill 并输入写作主题');
            return;
        }
        setLoading(true);
        setStatusMsg('AI 正在生成文章...');
        try {
            const result = await articleApi.generate(selectedSkillId, topic);
            setArticle(result);
            setAiContent(result.ai_generated_content);
            setUserContent(result.ai_generated_content);
            setStatusMsg('生成完成');
        } catch (e) {
            setStatusMsg(`生成失败: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    // 保存修改
    const handleSave = async () => {
        if (!article) return;
        try {
            await articleApi.save(article.id, userContent);
            setStatusMsg('已保存');
        } catch (e) {
            setStatusMsg(`保存失败: ${e}`);
        }
    };

    // 进化 Skill
    const handleEvolve = async () => {
        if (!article || !selectedSkillId) return;
        if (aiContent === userContent) {
            setStatusMsg('内容未修改，无需进化');
            return;
        }
        setLoading(true);
        setStatusMsg('正在分析修改差异...');
        try {
            await articleApi.analyzeDiff(article.id, aiContent, userContent);
            setStatusMsg('Diff 分析完成，Skill 进化中需在 Skill 管理页查看');
        } catch (e) {
            setStatusMsg(`进化失败: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    // 无 Skill 时的空状态
    if (skills.length === 0) {
        return (
            <div className="editor-page">
                <div className="empty-state">
                    <div className="icon">✏️</div>
                    <h1>{t('editor.title')}</h1>
                    <p>请先在「Skill 管理」中创建 Skill，才能开始写作</p>
                </div>
            </div>
        );
    }

    return (
        <div className="editor-page">
            {/* 顶部工具栏 */}
            <div className="editor-toolbar">
                <div className="toolbar-left">
                    <select
                        className="skill-select"
                        value={selectedSkillId || ''}
                        onChange={(e) => setSelectedSkillId(Number(e.target.value) || null)}
                    >
                        <option value="">选择 Skill...</option>
                        {skills.map((s) => (
                            <option key={s.id} value={s.id}>
                                {s.name} (v{s.current_version})
                            </option>
                        ))}
                    </select>

                    <input
                        className="topic-input"
                        type="text"
                        placeholder="输入写作主题..."
                        value={topic}
                        onChange={(e) => setTopic(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleGenerate()}
                    />
                </div>

                <div className="toolbar-right">
                    <button
                        className="btn btn-primary"
                        onClick={handleGenerate}
                        disabled={loading}
                    >
                        {loading ? '生成中...' : t('workspace.startWriting')}
                    </button>
                    {article && (
                        <>
                            <button className="btn btn-outline" onClick={handleSave}>
                                {t('common.save')}
                            </button>
                            <button className="btn btn-outline" onClick={handleEvolve} disabled={loading}>
                                {t('editor.evolveSkill')}
                            </button>
                        </>
                    )}
                </div>
            </div>

            {/* 状态栏 */}
            {statusMsg && <div className="status-bar">{statusMsg}</div>}

            {/* 双栏编辑器 */}
            {article ? (
                <div className="editor-panels">
                    <div className="editor-panel">
                        <div className="panel-header">
                            <span className="panel-label">AI 生成原文（只读）</span>
                            <span className="badge">v{article.skill_version_used}</span>
                        </div>
                        <Editor
                            height="100%"
                            language="markdown"
                            value={aiContent}
                            options={{
                                readOnly: true,
                                minimap: { enabled: false },
                                fontSize: 14,
                                lineNumbers: 'off',
                                wordWrap: 'on',
                                scrollBeyondLastLine: false,
                                renderWhitespace: 'none',
                                padding: { top: 12 },
                            }}
                            theme="vs-light"
                        />
                    </div>

                    <div className="editor-divider" />

                    <div className="editor-panel">
                        <div className="panel-header">
                            <span className="panel-label">编辑区</span>
                            {aiContent !== userContent && (
                                <span className="badge changed">已修改</span>
                            )}
                        </div>
                        <Editor
                            height="100%"
                            language="markdown"
                            value={userContent}
                            onChange={(val) => setUserContent(val || '')}
                            onMount={(editor) => {
                                editorRef.current = editor;
                            }}
                            options={{
                                minimap: { enabled: false },
                                fontSize: 14,
                                lineNumbers: 'off',
                                wordWrap: 'on',
                                scrollBeyondLastLine: false,
                                renderWhitespace: 'none',
                                padding: { top: 12 },
                            }}
                            theme="vs-light"
                        />
                    </div>
                </div>
            ) : (
                <div className="editor-placeholder">
                    <div className="empty-state">
                        <div className="icon">📝</div>
                        <p>选择 Skill 并输入主题，点击「开始写作」生成 AI 初稿</p>
                        <p style={{ fontSize: '12px', color: 'var(--text-tertiary)' }}>
                            生成后，在右侧编辑区修改文章，然后点击「进化 Skill」让 AI 学习你的风格
                        </p>
                    </div>
                </div>
            )}
        </div>
    );
}
