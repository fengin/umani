import { tauriInvoke } from './api';
import type { Skill, SkillVersion, CreateSkillRequest, UpdateSkillRequest } from '../types';

export const skillApi = {
    create: (request: CreateSkillRequest) =>
        tauriInvoke<Skill>('create_skill', { request }),

    get: (id: number) =>
        tauriInvoke<Skill>('get_skill', { id }),

    list: () =>
        tauriInvoke<Skill[]>('list_skills'),

    update: (id: number, request: UpdateSkillRequest) =>
        tauriInvoke<Skill>('update_skill', { id, request }),

    delete: (id: number) =>
        tauriInvoke<void>('delete_skill', { id }),

    getVersions: (skillId: number) =>
        tauriInvoke<SkillVersion[]>('get_skill_versions', { skillId }),

    getVersion: (skillId: number, versionNumber: number) =>
        tauriInvoke<SkillVersion>('get_skill_version', { skillId, versionNumber }),
};
