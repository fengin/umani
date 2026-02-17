import { useState, useEffect, useCallback } from 'react';
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
    const [statusMsg, setStatusMsg] = useState('');

    const loadSkills = useCallback(async () => {
        try {
            const list = await skillApi.list();
            setSkills(list);
        } catch (e) {
            setStatusMsg(`加载失败: ${e}`);
        }
    }, []);

    useEffect(() => {
        loadSkills();
    }, [loadSkills]);

    const handleCreate = async () => {
        if (!newName.trim()) return;
        try {
            await skillApi.create({
                name: newName,
                category: newCategory || '通用',
                description: newDesc,
            });
            setNewName('');
            setNewCategory('通用');
            setNewDesc('');
            setShowCreate(false);
            await loadSkills();
            setStatusMsg('Skill 创建成功');
        } catch (e) {
            setStatusMsg(`创建失败: ${e}`);
        }
    };

    const handleDelete = async (id: number) => {
        try {
            await skillApi.delete(id);
            if (selectedSkill?.id === id) {
                setSelectedSkill(null);
                setVersions([]);
            }
            await loadSkills();
            setStatusMsg('已删除');
        } catch (e) {
            setStatusMsg(`删除失败: ${e}`);
        }
    };

    const handleSelect = async (skill: Skill) => {
        setSelectedSkill(skill);
        try {
            const v = await skillApi.getVersions(skill.id);
            setVersions(v);
        } catch (e) {
            setStatusMsg(`加载版本失败: ${e}`);
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
                            placeholder="Skill 名称"
                            value={newName}
                            onChange={(e) => setNewName(e.target.value)}
                        />
                        <input
                            className="setting-input"
                            placeholder="分类（如：科技评论、生活笔记）"
                            value={newCategory}
                            onChange={(e) => setNewCategory(e.target.value)}
                        />
                        <input
                            className="setting-input"
                            placeholder="描述（可选）"
                            value={newDesc}
                            onChange={(e) => setNewDesc(e.target.value)}
                        />
                        <div className="form-actions">
                            <button className="btn btn-primary" onClick={handleCreate}>
                                {t('common.confirm')}
                            </button>
                            <button className="btn btn-outline" onClick={() => setShowCreate(false)}>
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
                            <button className="btn btn-outline" onClick={() => handleExport('markdown')}>
                                导出 Markdown
                            </button>
                            <button className="btn btn-outline" onClick={() => handleExport('json')}>
                                导出 JSON
                            </button>
                            <button className="btn btn-danger" onClick={() => handleDelete(selectedSkill.id)}>
                                {t('common.delete')}
                            </button>
                        </div>
                    </div>

                    <div className="version-list">
                        <h3>版本历史 ({versions.length})</h3>
                        {versions.map((v) => (
                            <div key={v.id} className="version-item">
                                <div className="version-header">
                                    <span className="badge">v{v.version_number}</span>
                                    <span className="time">{v.created_at?.slice(0, 16)}</span>
                                </div>
                                <p className="version-summary">{v.change_summary}</p>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
