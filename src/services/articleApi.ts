import { tauriInvoke } from './api';
import type { Article } from '../types';

export interface DiffChunk {
    tag: 'equal' | 'delete' | 'insert';
    value: string;
}

export const articleApi = {
    generate: (skillId: number, topic: string) =>
        tauriInvoke<Article>('generate_article', { skillId, topic }),

    save: (articleId: number, content: string) =>
        tauriInvoke<void>('save_article', { articleId, content }),

    get: (articleId: number) =>
        tauriInvoke<Article>('get_article', { articleId }),

    list: () =>
        tauriInvoke<Article[]>('list_articles'),

    computeDiff: (original: string, modified: string) =>
        tauriInvoke<DiffChunk[]>('compute_diff', { original, modified }),

    analyzeDiff: (articleId: number, original: string, modified: string) =>
        tauriInvoke<unknown>('analyze_diff', { articleId, original, modified }),

    evolveSkill: (
        skillId: number,
        newContentMarkdown: string,
        newContentJson: string,
        changeSummary: string,
    ) =>
        tauriInvoke<void>('evolve_skill', {
            skillId,
            newContentMarkdown,
            newContentJson,
            changeSummary,
        }),
};
