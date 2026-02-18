import { useState, useRef, useCallback, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import Editor from '@monaco-editor/react';
import { skillApi } from '../services/skillApi';
import { articleApi } from '../services/articleApi';
import type { Skill, Article } from '../types';
import './Editor.css';

export default function EditorPage() {
    const { t } = useTranslation();
    const navigate = useNavigate();

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

    const [searchParams, setSearchParams] = useSearchParams();

    // 加载 Skills
    const loadSkills = useCallback(async () => {
        try {
            const list = await skillApi.list();
            setSkills(list);
            return list;
        } catch (e) {
            console.error('加载 Skills 失败:', e);
            return [];
        }
    }, []);

    // 首次加载 + URL 参数处理
    useEffect(() => {
        const init = async () => {
            await loadSkills();
            // 检查 URL 中是否有 articleId 参数
            const urlArticleId = searchParams.get('articleId');
            if (urlArticleId) {
                try {
                    const art = await articleApi.get(Number(urlArticleId));
                    setArticle(art);
                    setAiContent(art.ai_generated_content);
                    setUserContent(art.user_refined_content || art.ai_generated_content);
                    setTopic(art.title);
                    // 自动选中对应 Skill
                    if (art.skill_id) {
                        setSelectedSkillId(art.skill_id);
                    }
                } catch (e) {
                    console.error('加载文章失败:', e);
                }
                // 消费掉参数
                setSearchParams({}, { replace: true });
            }
        };
        init();
    }, []); // eslint-disable-line react-hooks/exhaustive-deps

    // AI 生成文章
    const handleGenerate = async () => {
        if (!selectedSkillId || !topic.trim()) {
            setStatusMsg(t('editor.selectSkillAndTopic'));
            return;
        }
        setLoading(true);
        setStatusMsg(t('editor.generating'));
        try {
            const result = await articleApi.generate(selectedSkillId, topic);
            setArticle(result);
            setAiContent(result.ai_generated_content);
            setUserContent(result.ai_generated_content);
            setStatusMsg(t('editor.generateDone'));
        } catch (e) {
            setStatusMsg(`${t('editor.generateFailed')}: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    // 保存修改
    const handleSave = async () => {
        if (!article) return;
        try {
            await articleApi.save(article.id, userContent);
            setStatusMsg(t('editor.saved'));
        } catch (e) {
            setStatusMsg(`${t('editor.saveFailed')}: ${e}`);
        }
    };

    // 进化 Skill
    const handleEvolve = async () => {
        if (!article || !selectedSkillId) return;
        if (aiContent === userContent) {
            setStatusMsg(t('editor.noChanges'));
            return;
        }
        setLoading(true);
        setStatusMsg(t('editor.evolving'));
        try {
            await articleApi.analyzeDiff(article.id, aiContent, userContent);
            setStatusMsg(t('editor.evolveSuccess'));
        } catch (e) {
            setStatusMsg(`${t('editor.evolveFailed')}: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    // 无 Skill 时的空状态
    if (skills.length === 0) {
        return (
            <div className="editor-page">
                <div className="empty-state">
                    <div className="icon">🎯</div>
                    <h1>{t('editor.title')}</h1>
                    <p>{t('editor.noSkillHint')}</p>
                    <button className="btn btn-primary" onClick={() => navigate('/skills')}>
                        {t('workspace.newSkill')}
                    </button>
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
                        <option value="">{t('editor.selectSkill')}</option>
                        {skills.map((s) => (
                            <option key={s.id} value={s.id}>
                                {s.name} (v{s.current_version})
                            </option>
                        ))}
                    </select>

                    <input
                        className="topic-input"
                        type="text"
                        placeholder={t('editor.topicPlaceholder')}
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
                        {loading ? t('editor.generating') : t('editor.generateBtn')}
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
                            <span className="panel-label">{t('editor.aiDraft')}</span>
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
                            <span className="panel-label">{t('editor.yourEdit')}</span>
                            {aiContent !== userContent && (
                                <span className="badge changed">{t('editor.modified')}</span>
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
                    <div className="training-guide">
                        <h2>{t('editor.guideTitle')}</h2>
                        <p className="guide-subtitle">{t('editor.guideSubtitle')}</p>
                        <div className="guide-steps">
                            <div className="guide-step">
                                <div className="guide-step-number">1</div>
                                <div className="guide-step-text">
                                    <strong>{t('editor.step1Title')}</strong>
                                    <span>{t('editor.step1Desc')}</span>
                                </div>
                            </div>
                            <div className="guide-arrow">→</div>
                            <div className="guide-step">
                                <div className="guide-step-number">2</div>
                                <div className="guide-step-text">
                                    <strong>{t('editor.step2Title')}</strong>
                                    <span>{t('editor.step2Desc')}</span>
                                </div>
                            </div>
                            <div className="guide-arrow">→</div>
                            <div className="guide-step">
                                <div className="guide-step-number">3</div>
                                <div className="guide-step-text">
                                    <strong>{t('editor.step3Title')}</strong>
                                    <span>{t('editor.step3Desc')}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
