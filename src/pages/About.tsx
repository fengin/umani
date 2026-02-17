import { useTranslation } from 'react-i18next';
import './About.css';

export default function AboutPage() {
    const { t } = useTranslation();

    return (
        <div className="about-page">
            <h1>{t('settings.about')}</h1>

            <div className="about-card">
                <div className="about-header">
                    <span className="about-logo">🍃</span>
                    <div>
                        <div className="about-name">
                            {t('about.productName')}
                            <span className="badge">v0.1.0</span>
                        </div>
                    </div>
                </div>

                <p className="about-description">{t('about.description')}</p>

                <div className="about-meta">
                    <div className="about-meta-row">
                        <span className="about-label">{t('about.authorLabel')}</span>
                        <span className="about-value">{t('about.author')}</span>
                    </div>
                    <div className="about-meta-row">
                        <span className="about-label">{t('about.websiteLabel')}</span>
                        <a className="about-link" href="https://aibook.ren" target="_blank" rel="noopener noreferrer">
                            {t('about.website')}
                        </a>
                    </div>
                    <div className="about-meta-row">
                        <span className="about-label">{t('about.theoryLabel')}</span>
                        <a className="about-link" href="https://aibook.ren/archives/how-to-write-without-ai-taste" target="_blank" rel="noopener noreferrer">
                            {t('about.theory')}
                        </a>
                    </div>
                </div>

                <p className="about-copyright">MIT License © 2026 fengin</p>
            </div>
        </div>
    );
}
