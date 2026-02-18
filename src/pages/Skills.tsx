import { useState, useEffect, useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { skillApi } from '../services/skillApi';
import { tauriInvoke } from '../services/api';
import type { Skill, SkillVersion } from '../types';
import './Skills.css';

export default function SkillsPage() {
    const { t } = useTranslation();
    const [skills, setSkills] = useState<Skill[]>([]);
    const [selectedSkill, setSelectedSkill] = useState<Skill | null>(null);
    const [versions, setVersions] = useState<SkillVersion[]>([]);
    const [showCreate, setShowCreate] = useState(false);
    const [newName, setNewName] = useState('');
    const [newCategory, setNewCategory] = useState('通用');
    const [newDesc, setNewDesc] = useState('');
    const [newSamples, setNewSamples] = useState('');
    const [analyzing, setAnalyzing] = useState(false);
    const [statusMsg, setStatusMsg] = useState('');

    // 版本内容查看/编辑
    const [expandedVersionId, setExpandedVersionId] = useState<number | null>(null);
    const [editingVersionId, setEditingVersionId] = useState<number | null>(null);
    const [editContent, setEditContent] = useState('');

    // Skill 信息编辑
    const [editingInfo, setEditingInfo] = useState(false);
    const [editName, setEditName] = useState('');
    const [editCategory, setEditCategory] = useState('');
    const [editDesc, setEditDesc] = useState('');

    const [searchParams, setSearchParams] = useSearchParams();

    const loadSkills = useCallback(async () => {
        try {
            const list = await skillApi.list();
            setSkills(list);
            // 如果 URL 中有 id 参数，自动选中对应 Skill
            const urlId = searchParams.get('id');
            if (urlId) {
                const targetSkill = list.find(s => s.id === Number(urlId));
                if (targetSkill) {
                    setSelectedSkill(targetSkill);
                    const v = await skillApi.getVersions(targetSkill.id);
                    setVersions(v);
                }
                // 消费掉参数，避免重复触发
                setSearchParams({}, { replace: true });
            }
        } catch (e) {
            setStatusMsg(`加载失败: ${e}`);
        }
    }, [searchParams, setSearchParams]);

    useEffect(() => {
        loadSkills();
    }, [loadSkills]);

    const resetCreateForm = () => {
        setNewName('');
        setNewCategory('通用');
        setNewDesc('');
        setNewSamples('');
        setShowCreate(false);
    };

    const handleCreate = async () => {
        if (!newName.trim()) return;

        // 有样本时走初始建模流程
        if (newSamples.trim()) {
            setAnalyzing(true);
            setStatusMsg(t('skills.analyzing'));
            try {
                await tauriInvoke('create_skill_with_samples', {
                    name: newName,
                    category: newCategory || '通用',
                    description: newDesc,
                    samplesText: newSamples,
                });
                resetCreateForm();
                await loadSkills();
                setStatusMsg(t('skills.createWithSamplesSuccess'));
            } catch (e) {
                setStatusMsg(`${t('skills.createFailed')}: ${e}`);
            } finally {
                setAnalyzing(false);
            }
            return;
        }

        // 无样本时快速创建
        try {
            await skillApi.create({
                name: newName,
                category: newCategory || '通用',
                description: newDesc,
            });
            resetCreateForm();
            await loadSkills();
            setStatusMsg(t('skills.createSuccess'));
        } catch (e) {
            setStatusMsg(`${t('skills.createFailed')}: ${e}`);
        }
    };

    const handleDelete = async (id: number) => {
        if (!confirm(t('skills.deleteConfirm'))) return;
        try {
            await skillApi.delete(id);
            if (selectedSkill?.id === id) {
                setSelectedSkill(null);
                setVersions([]);
            }
            await loadSkills();
            setStatusMsg(t('skills.deleted'));
        } catch (e) {
            setStatusMsg(`${t('skills.deleteFailed')}: ${e}`);
        }
    };

    const handleSelect = async (skill: Skill) => {
        setSelectedSkill(skill);
        setExpandedVersionId(null);
        setEditingVersionId(null);
        setEditingInfo(false);
        try {
            const v = await skillApi.getVersions(skill.id);
            setVersions(v);
        } catch (e) {
            setStatusMsg(`加载版本失败: ${e}`);
        }
    };

    // 开始编辑 Skill 信息
    const startEditInfo = () => {
        if (!selectedSkill) return;
        setEditName(selectedSkill.name);
        setEditCategory(selectedSkill.category);
        setEditDesc(selectedSkill.description);
        setEditingInfo(true);
    };

    // 保存 Skill 信息
    const handleSaveInfo = async () => {
        if (!selectedSkill || !editName.trim()) return;
        try {
            const updated = await skillApi.update(selectedSkill.id, {
                name: editName,
                category: editCategory,
                description: editDesc,
            });
            setSelectedSkill(updated);
            setEditingInfo(false);
            await loadSkills();
            setStatusMsg(t('skills.infoSaved'));
        } catch (e) {
            setStatusMsg(`${t('skills.infoSaveFailed')}: ${e}`);
        }
    };

    const handleExport = async (format: 'markdown' | 'json') => {
        if (!selectedSkill) return;
        try {
            const content = await tauriInvoke<string>(
                format === 'markdown' ? 'export_skill_markdown' : 'export_skill_json',
                { skillId: selectedSkill.id }
            );
            await navigator.clipboard.writeText(content);
            setStatusMsg(`已复制到剪贴板（${format.toUpperCase()}）`);
        } catch (e) {
            setStatusMsg(`导出失败: ${e}`);
        }
    };

    // 展开/收起版本内容
    const toggleVersion = (versionId: number, content: string) => {
        if (expandedVersionId === versionId) {
            setExpandedVersionId(null);
            setEditingVersionId(null);
        } else {
            setExpandedVersionId(versionId);
            setEditingVersionId(null);
            setEditContent(content);
        }
    };

    // 进入编辑模式
    const startEditing = (versionId: number, content: string) => {
        setEditingVersionId(versionId);
        setEditContent(content);
    };

    // 保存编辑内容为新版本
    const handleSaveVersion = async () => {
        if (!selectedSkill || !editContent.trim()) return;
        try {
            await tauriInvoke('evolve_skill', {
                skillId: selectedSkill.id,
                newContentMarkdown: editContent,
                newContentJson: '{}',
                changeSummary: t('skills.manualEditSummary'),
            });
            setEditingVersionId(null);
            setExpandedVersionId(null);
            // 重新加载 Skill 和版本
            await loadSkills();
            const v = await skillApi.getVersions(selectedSkill.id);
            setVersions(v);
            // 更新选中的 Skill 信息
            const updatedSkill = await skillApi.get(selectedSkill.id);
            setSelectedSkill(updatedSkill);
            setStatusMsg(t('skills.versionSaved'));
        } catch (e) {
            setStatusMsg(`${t('skills.versionSaveFailed')}: ${e}`);
        }
    };

    return (
        <div className="skills-page">
            {/* 列表区 */}
            <div className="skills-list-section">
                <div className="section-header">
                    <h1>{t('skills.title')}</h1>
                    <button className="btn btn-primary" onClick={() => setShowCreate(!showCreate)}>
                        {t('workspace.newSkill')}
                    </button>
                </div>

                {statusMsg && <div className="status-bar">{statusMsg}</div>}

                {/* 创建表单 */}
                {showCreate && (
                    <div className="create-form">
                        <input
                            className="setting-input"
                            placeholder={t('skills.name')}
                            value={newName}
                            onChange={(e) => setNewName(e.target.value)}
                        />
                        <input
                            className="setting-input"
                            placeholder={t('skills.categoryPlaceholder')}
                            value={newCategory}
                            onChange={(e) => setNewCategory(e.target.value)}
                        />
                        <input
                            className="setting-input"
                            placeholder={t('skills.descPlaceholder')}
                            value={newDesc}
                            onChange={(e) => setNewDesc(e.target.value)}
                        />
                        <div className="samples-section">
                            <label className="samples-label">{t('skills.samplesLabel')}</label>
                            <textarea
                                className="samples-textarea"
                                placeholder={t('skills.samplesPlaceholder')}
                                value={newSamples}
                                onChange={(e) => setNewSamples(e.target.value)}
                                rows={6}
                            />
                        </div>
                        <div className="form-actions">
                            <button
                                className="btn btn-primary"
                                onClick={handleCreate}
                                disabled={analyzing}
                            >
                                {analyzing
                                    ? t('skills.analyzing')
                                    : newSamples.trim()
                                        ? t('skills.createWithSamples')
                                        : t('common.confirm')}
                            </button>
                            <button className="btn btn-outline" onClick={() => resetCreateForm()} disabled={analyzing}>
                                {t('common.cancel')}
                            </button>
                        </div>
                    </div>
                )}

                {/* Skill 列表 */}
                {skills.length === 0 ? (
                    <div className="empty-state">
                        <div className="icon">📦</div>
                        <p>{t('skills.empty')}</p>
                    </div>
                ) : (
                    <div className="skill-cards">
                        {skills.map((skill) => (
                            <div
                                key={skill.id}
                                className={`skill-card ${selectedSkill?.id === skill.id ? 'active' : ''}`}
                                onClick={() => handleSelect(skill)}
                            >
                                <div className="card-header">
                                    <span className="skill-name">{skill.name}</span>
                                    <span className="badge">v{skill.current_version}</span>
                                </div>
                                <div className="card-meta">
                                    <span className="category">{skill.category}</span>
                                    <span className="time">{skill.updated_at?.slice(0, 10)}</span>
                                </div>
                                {skill.description && (
                                    <p className="card-desc">{skill.description}</p>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* 详情区 */}
            {selectedSkill && (
                <div className="skill-detail-section">
                    <div className="detail-header">
                        <h2>{selectedSkill.name}</h2>
                        <div className="detail-actions">
                            {!editingInfo && (
                                <button className="btn btn-outline" onClick={startEditInfo}>
                                    {t('common.edit')}
                                </button>
                            )}
                            <button className="btn btn-outline" onClick={() => handleExport('markdown')}>
                                {t('skills.exportMd')}
                            </button>
                            <button className="btn btn-outline" onClick={() => handleExport('json')}>
                                {t('skills.exportJson')}
                            </button>
                            <button className="btn btn-danger" onClick={() => handleDelete(selectedSkill.id)}>
                                {t('common.delete')}
                            </button>
                        </div>
                    </div>

                    {/* Skill 信息编辑区 */}
                    {editingInfo ? (
                        <div className="skill-info-edit">
                            <div className="info-field">
                                <label>{t('skills.name')}</label>
                                <input className="setting-input" value={editName} onChange={e => setEditName(e.target.value)} />
                            </div>
                            <div className="info-field">
                                <label>{t('skills.category')}</label>
                                <input className="setting-input" value={editCategory} onChange={e => setEditCategory(e.target.value)} />
                            </div>
                            <div className="info-field">
                                <label>{t('skills.descPlaceholder')}</label>
                                <input className="setting-input" value={editDesc} onChange={e => setEditDesc(e.target.value)} />
                            </div>
                            <div className="form-actions">
                                <button className="btn btn-primary btn-sm" onClick={handleSaveInfo}>{t('common.save')}</button>
                                <button className="btn btn-outline btn-sm" onClick={() => setEditingInfo(false)}>{t('common.cancel')}</button>
                            </div>
                        </div>
                    ) : (
                        <div className="skill-info-display">
                            <span className="info-tag">{selectedSkill.category}</span>
                            {selectedSkill.description && <p className="info-desc">{selectedSkill.description}</p>}
                        </div>
                    )}

                    <div className="version-list">
                        <h3>{t('skills.versionHistory')} ({versions.length})</h3>
                        {versions.map((v) => (
                            <div key={v.id} className={`version-item ${expandedVersionId === v.id ? 'expanded' : ''}`}>
                                <div
                                    className="version-header clickable"
                                    onClick={() => toggleVersion(v.id, v.content_markdown)}
                                >
                                    <div className="version-header-left">
                                        <span className="badge">v{v.version_number}</span>
                                        <span className="time">{v.created_at?.slice(0, 16)}</span>
                                        {v.version_number === selectedSkill.current_version && (
                                            <span className="badge current">{t('skills.currentVersion')}</span>
                                        )}
                                    </div>
                                    <span className="version-toggle">
                                        {expandedVersionId === v.id ? '▼' : '▶'}
                                    </span>
                                </div>
                                <p className="version-summary">{v.change_summary}</p>

                                {/* 展开的版本内容 */}
                                {expandedVersionId === v.id && (
                                    <div className="version-content">
                                        {editingVersionId === v.id ? (
                                            <>
                                                <textarea
                                                    className="version-editor"
                                                    value={editContent}
                                                    onChange={(e) => setEditContent(e.target.value)}
                                                    rows={15}
                                                />
                                                <div className="version-edit-actions">
                                                    <button className="btn btn-primary btn-sm" onClick={handleSaveVersion}>
                                                        {t('skills.saveAsNewVersion')}
                                                    </button>
                                                    <button className="btn btn-outline btn-sm" onClick={() => setEditingVersionId(null)}>
                                                        {t('common.cancel')}
                                                    </button>
                                                </div>
                                            </>
                                        ) : (
                                            <>
                                                <pre className="version-preview">{v.content_markdown || t('common.noData')}</pre>
                                                <div className="version-edit-actions">
                                                    <button
                                                        className="btn btn-outline btn-sm"
                                                        onClick={() => startEditing(v.id, v.content_markdown)}
                                                    >
                                                        {t('common.edit')}
                                                    </button>
                                                </div>
                                            </>
                                        )}
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
