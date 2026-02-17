import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { llmApi } from '../services/llmApi';
import { LLM_PROVIDERS } from '../types';
import type { LlmConfig, LlmProvider } from '../types';
import './Settings.css';

export default function SettingsPage() {
    const { t, i18n } = useTranslation();

    // LLM 配置状态
    const [config, setConfig] = useState<LlmConfig>({
        provider: 'openai',
        endpoint: 'https://api.openai.com/v1',
        api_key: '',
        model: 'gpt-4o',
    });
    const [saving, setSaving] = useState(false);
    const [testing, setTesting] = useState(false);
    const [statusMsg, setStatusMsg] = useState('');

    // 加载配置
    const loadConfig = useCallback(async () => {
        try {
            const c = await llmApi.getConfig();
            setConfig(c);
        } catch (e) {
            console.error('加载 LLM 配置失败:', e);
        }
    }, []);

    useEffect(() => {
        loadConfig();
    }, [loadConfig]);

    // Provider 切换
    const handleProviderChange = (provider: LlmProvider) => {
        const p = LLM_PROVIDERS.find((pp) => pp.value === provider);
        setConfig({
            ...config,
            provider,
            endpoint: p?.defaultEndpoint || '',
            model: p?.models[0] || '',
        });
    };

    // 保存
    const handleSave = async () => {
        setSaving(true);
        try {
            await llmApi.saveConfig(config);
            setStatusMsg('配置已保存');
        } catch (e) {
            setStatusMsg(`保存失败: ${e}`);
        } finally {
            setSaving(false);
        }
    };

    // 测试连接
    const handleTest = async () => {
        setTesting(true);
        setStatusMsg('正在测试连接...');
        try {
            await handleSave(); // 先保存
            const reply = await llmApi.testConnection();
            setStatusMsg(`连接成功！LLM 回复: ${reply.slice(0, 100)}`);
        } catch (e) {
            setStatusMsg(`连接失败: ${e}`);
        } finally {
            setTesting(false);
        }
    };

    const currentProvider = LLM_PROVIDERS.find((p) => p.value === config.provider);

    return (
        <div className="settings-page">
            <h1>{t('settings.title')}</h1>

            {statusMsg && <div className="status-bar">{statusMsg}</div>}

            {/* LLM 配置 */}
            <section className="setting-section">
                <h2>{t('settings.llmConfig')}</h2>

                <div className="setting-row">
                    <label>{t('settings.provider')}</label>
                    <select
                        className="setting-input"
                        value={config.provider}
                        onChange={(e) => handleProviderChange(e.target.value as LlmProvider)}
                    >
                        {LLM_PROVIDERS.map((p) => (
                            <option key={p.value} value={p.value}>
                                {p.label}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="setting-row">
                    <label>API Endpoint</label>
                    <input
                        className="setting-input"
                        type="text"
                        value={config.endpoint}
                        onChange={(e) => setConfig({ ...config, endpoint: e.target.value })}
                        placeholder="https://api.openai.com/v1"
                    />
                </div>

                <div className="setting-row">
                    <label>{t('settings.apiKey')}</label>
                    <input
                        className="setting-input"
                        type="password"
                        value={config.api_key}
                        onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                        placeholder="sk-..."
                    />
                </div>

                <div className="setting-row">
                    <label>{t('settings.model')}</label>
                    {currentProvider && currentProvider.models.length > 0 ? (
                        <select
                            className="setting-input"
                            value={config.model}
                            onChange={(e) => setConfig({ ...config, model: e.target.value })}
                        >
                            {currentProvider.models.map((m) => (
                                <option key={m} value={m}>{m}</option>
                            ))}
                        </select>
                    ) : (
                        <input
                            className="setting-input"
                            type="text"
                            value={config.model}
                            onChange={(e) => setConfig({ ...config, model: e.target.value })}
                            placeholder="输入模型名称"
                        />
                    )}
                </div>

                <div className="setting-actions">
                    <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
                        {saving ? '保存中...' : t('common.save')}
                    </button>
                    <button className="btn btn-outline" onClick={handleTest} disabled={testing}>
                        {testing ? '测试中...' : '测试连接'}
                    </button>
                </div>
            </section>

            {/* 语言 */}
            <section className="setting-section">
                <h2>{t('settings.language')}</h2>
                <div className="setting-row">
                    <label>{t('settings.language')}</label>
                    <div className="language-switcher">
                        <button
                            className={`lang-btn ${i18n.language === 'zh-CN' ? 'active' : ''}`}
                            onClick={() => i18n.changeLanguage('zh-CN')}
                        >
                            中文
                        </button>
                        <button
                            className={`lang-btn ${i18n.language === 'en-US' ? 'active' : ''}`}
                            onClick={() => i18n.changeLanguage('en-US')}
                        >
                            English
                        </button>
                    </div>
                </div>
            </section>
        </div>
    );
}
