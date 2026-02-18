import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { skillApi } from '../services/skillApi';
import { articleApi } from '../services/articleApi';
import { tauriInvoke } from '../services/api';
import type { Skill, Article } from '../types';
import './Workspace.css';

interface OnboardingStatus {
    llm_configured: boolean;
    has_skills: boolean;
    has_articles: boolean;
}

export default function WorkspacePage() {
    const { t } = useTranslation();
    const navigate = useNavigate();

    const [skills, setSkills] = useState<Skill[]>([]);
    const [articles, setArticles] = useState<Article[]>([]);
    const [onboarding, setOnboarding] = useState<OnboardingStatus | null>(null);
    const [loading, setLoading] = useState(true);

    const totalEvolutions = skills.reduce((sum, s) => sum + (s.current_version - 1), 0);

    // 判断引导是否全部完成
    const allCompleted = onboarding?.llm_configured && onboarding?.has_skills && onboarding?.has_articles;

    const loadData = useCallback(async () => {
        setLoading(true);
        try {
            const [skillList, articleList, status] = await Promise.all([
                skillApi.list(),
                articleApi.list(),
                tauriInvoke<OnboardingStatus>('get_onboarding_status'),
            ]);
            setSkills(skillList);
            setArticles(articleList);
            setOnboarding(status);
        } catch (e) {
            console.error('加载数据失败:', e);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadData();
    }, [loadData]);

    return (
        <div className="workspace-page">
            <div className="page-header">
                <div>
                    <h1>{t('workspace.welcome')}</h1>
                    <p className="subtitle">{t('app.tagline')}</p>
                </div>
                <div className="page-actions">
                    <button className="btn btn-primary" onClick={() => navigate('/skills')}>
                        {t('workspace.newSkill')}
                    </button>
                    <button className="btn btn-outline" onClick={() => navigate('/editor')}>
                        {t('workspace.startTraining')}
                    </button>
                </div>
            </div>

            {/* 新手引导步骤卡片 */}
            {onboarding && !allCompleted && (
                <div className="onboarding-card">
                    <h2 className="onboarding-title">{t('onboarding.title')}</h2>
                    <div className="onboarding-steps">
                        <div
                            className={`onboarding-step ${onboarding.llm_configured ? 'completed' : 'active'}`}
                            onClick={() => !onboarding.llm_configured && navigate('/settings')}
                        >
                            <div className="step-number">{onboarding.llm_configured ? '✅' : '1'}</div>
                            <div className="step-content">
                                <span className="step-label">{t('onboarding.step1')}</span>
                                <span className="step-desc">{t('onboarding.step1Desc')}</span>
                            </div>
                            {!onboarding.llm_configured && (
                                <span className="step-action">{t('onboarding.go')} →</span>
                            )}
                        </div>
                        <div
                            className={`onboarding-step ${onboarding.has_skills ? 'completed' : onboarding.llm_configured ? 'active' : 'pending'}`}
                            onClick={() => onboarding.llm_configured && !onboarding.has_skills && navigate('/skills')}
                        >
                            <div className="step-number">{onboarding.has_skills ? '✅' : '2'}</div>
                            <div className="step-content">
                                <span className="step-label">{t('onboarding.step2')}</span>
                                <span className="step-desc">{t('onboarding.step2Desc')}</span>
                            </div>
                            {onboarding.llm_configured && !onboarding.has_skills && (
                                <span className="step-action">{t('onboarding.go')} →</span>
                            )}
                        </div>
                        <div
                            className={`onboarding-step ${onboarding.has_articles ? 'completed' : onboarding.has_skills ? 'active' : 'pending'}`}
                            onClick={() => onboarding.has_skills && !onboarding.has_articles && navigate('/editor')}
                        >
                            <div className="step-number">{onboarding.has_articles ? '✅' : '3'}</div>
                            <div className="step-content">
                                <span className="step-label">{t('onboarding.step3')}</span>
                                <span className="step-desc">{t('onboarding.step3Desc')}</span>
                            </div>
                            {onboarding.has_skills && !onboarding.has_articles && (
                                <span className="step-action">{t('onboarding.go')} →</span>
                            )}
                        </div>
                    </div>
                </div>
            )}

            <div className="stats-grid stats-3">
                <div className="stat-card" onClick={() => navigate('/skills')} style={{ cursor: 'pointer' }}>
                    <span className="stat-label">{t('workspace.stats.activeSkills')}</span>
                    <span className="stat-value">{loading ? '-' : skills.length}</span>
                </div>
                <div className="stat-card">
                    <span className="stat-label">{t('workspace.stats.evolutions')}</span>
                    <span className="stat-value">{loading ? '-' : totalEvolutions}</span>
                </div>
                <div className="stat-card" onClick={() => navigate('/editor')} style={{ cursor: 'pointer' }}>
                    <span className="stat-label">{t('workspace.stats.trainings')}</span>
                    <span className="stat-value">{loading ? '-' : articles.length}</span>
                </div>
            </div>

            <div className="workspace-content">
                <div className="section">
                    <div className="section-header">
                        <h2>{t('workspace.mySkills')}</h2>
                        {skills.length > 0 && (
                            <span className="link" onClick={() => navigate('/skills')}>
                                {t('workspace.viewAll')} →
                            </span>
                        )}
                    </div>
                    {skills.length === 0 ? (
                        <div className="empty-state">
                            <div className="icon">📦</div>
                            <p>{t('skills.empty')}</p>
                            <button className="btn btn-primary" onClick={() => navigate('/skills')}>
                                {t('workspace.newSkill')}
                            </button>
                        </div>
                    ) : (
                        <div className="skill-preview-list">
                            {skills.slice(0, 5).map((skill) => (
                                <div
                                    key={skill.id}
                                    className="skill-preview-card"
                                    onClick={() => navigate(`/skills?id=${skill.id}`)}
                                >
                                    <div className="preview-header">
                                        <span className="skill-name">{skill.name}</span>
                                        <span className="badge">v{skill.current_version}</span>
                                    </div>
                                    <div className="preview-meta">
                                        <span className="category">{skill.category}</span>
                                        <span className="time">{skill.updated_at?.slice(0, 10)}</span>
                                    </div>
                                    {skill.description && (
                                        <p className="preview-desc">{skill.description}</p>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </div>

                <div className="section">
                    <div className="section-header">
                        <h2>{t('workspace.recentTrainings')}</h2>
                    </div>
                    {articles.length === 0 ? (
                        <div className="empty-state small">
                            <p>{t('common.noData')}</p>
                        </div>
                    ) : (
                        <div className="article-preview-list">
                            {articles.slice(0, 5).map((article) => {
                                const skillName = skills.find(s => s.id === article.skill_id)?.name;
                                return (
                                    <div
                                        key={article.id}
                                        className="article-preview-item clickable"
                                        onClick={() => navigate(`/editor?articleId=${article.id}`)}
                                    >
                                        <span className="article-title">
                                            {skillName && <span className="article-skill-tag">[{skillName}]</span>}
                                            {article.title}
                                        </span>
                                        <span className="time">{article.created_at?.slice(0, 10)}</span>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
