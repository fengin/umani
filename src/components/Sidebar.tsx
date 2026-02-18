import { NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import './Sidebar.css';

const navItems = [
    { path: '/', icon: '📋', labelKey: 'nav.workspace' },
    { path: '/editor', icon: '🎯', labelKey: 'nav.editor' },
    { path: '/skills', icon: '📦', labelKey: 'nav.skills' },
    { path: '/settings', icon: '⚙️', labelKey: 'nav.settings' },
    { path: '/about', icon: '💡', labelKey: 'nav.about' },
];

export default function Sidebar() {
    const { t } = useTranslation();

    return (
        <aside className="sidebar">
            <div className="sidebar-brand">
                <span style={{ fontSize: '18px' }}>🍃</span>
                <span>{t('app.name')}</span>
            </div>

            <nav className="sidebar-nav">
                {navItems.map((item) => (
                    <NavLink
                        key={item.path}
                        to={item.path}
                        className={({ isActive }) =>
                            `nav-item ${isActive ? 'active' : ''}`
                        }
                    >
                        <span className="icon">{item.icon}</span>
                        <span>{t(item.labelKey)}</span>
                    </NavLink>
                ))}
            </nav>

            <div className="sidebar-footer">
                <div className="nav-item version-info">
                    <span className="icon">💡</span>
                    <span>v0.1.0</span>
                </div>
            </div>
        </aside>
    );
}
