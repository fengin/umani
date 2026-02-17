import { tauriInvoke } from './api';
import type { LlmConfig } from '../types';

export const llmApi = {
    getConfig: () =>
        tauriInvoke<LlmConfig>('get_llm_config'),

    saveConfig: (config: LlmConfig) =>
        tauriInvoke<void>('save_llm_config', {
            provider: config.provider,
            endpoint: config.endpoint,
            apiKey: config.api_key,
            model: config.model,
        }),

    testConnection: () =>
        tauriInvoke<string>('test_llm_connection'),
};
