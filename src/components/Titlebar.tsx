import { getCurrentWindow } from '@tauri-apps/api/window';
import { useTranslation } from 'react-i18next';
import './Titlebar.css';

const appWindow = getCurrentWindow();

export default function Titlebar() {
    const { t } = useTranslation();

    return (
        <div className="titlebar" data-tauri-drag-region>
            <span className="titlebar-title">{t('app.name')}</span>
            <div className="window-controls">
                <button
                    className="win-btn"
                    onClick={() => appWindow.minimize()}
                    title="最小化"
                >
                    <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" strokeWidth="1" /></svg>
                </button>
                <button
                    className="win-btn"
                    onClick={() => appWindow.toggleMaximize()}
                    title="最大化"
                >
                    <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1" /></svg>
                </button>
                <button
                    className="win-btn win-close"
                    onClick={() => appWindow.close()}
                    title="关闭"
                >
                    <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="1" x2="11" y2="11" stroke="currentColor" strokeWidth="1" /><line x1="11" y1="1" x2="1" y2="11" stroke="currentColor" strokeWidth="1" /></svg>
                </button>
            </div>
        </div>
    );
}
