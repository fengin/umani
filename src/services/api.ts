import { invoke } from '@tauri-apps/api/core';

/**
 * 统一的 Tauri IPC 调用封装
 * 自动处理错误并提供类型安全
 */
export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
        return await invoke<T>(command, args);
    } catch (error) {
        const message = typeof error === 'string' ? error : String(error);
        console.error(`[IPC] ${command} 失败:`, message);
        throw new Error(message);
    }
}
