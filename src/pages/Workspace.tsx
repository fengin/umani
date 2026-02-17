import { useTranslation } from 'react-i18next';
import './Workspace.css';

export default function WorkspacePage() {
    const { t } = useTranslation();

    return (
        <div className="workspace-page">
            <div className="page-header">
                <div>
                    <h1>{t('workspace.welcome')}</h1>
                    <p className="subtitle">{t('app.tagline')}</p>
                </div>
                <div className="page-actions">
                    <button className="btn btn-primary">{t('workspace.newSkill')}</button>
                    <button className="btn btn-outline">{t('workspace.startWriting')}</button>
                </div>
            </div>

            <div className="stats-grid">
                <div className="stat-card">
                    <span className="stat-label">{t('workspace.stats.activeSkills')}</span>
                    <span className="stat-value">0</span>
                </div>
                <div className="stat-card">
                    <span className="stat-label">{t('workspace.stats.evolutions')}</span>
                    <span className="stat-value">0</span>
                </div>
                <div className="stat-card">
                    <span className="stat-label">{t('workspace.stats.articles')}</span>
                    <span className="stat-value">0</span>
                </div>
                <div className="stat-card">
                    <span className="stat-label">{t('workspace.stats.blocklist')}</span>
                    <span className="stat-value">0</span>
                </div>
            </div>

            <div className="workspace-content">
                <div className="section">
                    <div className="section-header">
                        <h2>{t('workspace.mySkills')}</h2>
                        <span className="link">{t('workspace.viewAll')} →</span>
                    </div>
                    <div className="empty-state">
                        <div className="icon">📦</div>
                        <p>{t('skills.empty')}</p>
                        <button className="btn btn-primary">{t('workspace.newSkill')}</button>
                    </div>
                </div>

                <div className="section">
                    <h2>{t('workspace.recentArticles')}</h2>
                    <div className="empty-state small">
                        <p>{t('common.noData')}</p>
                    </div>
                </div>
            </div>
        </div>
    );
}
