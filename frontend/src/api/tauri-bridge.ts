/**
 * Tauri Bridge — provides access to native Tauri commands from React.
 *
 * When running in a browser (non-Tauri), all methods gracefully fall back
 * to using the HTTP API client, so the same React code works in both
 * the web app and the Tauri desktop app.
 */

// Check if we're running inside Tauri
const isTauri = (): boolean => {
    return typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';
};

// Dynamic import of Tauri API (only available in Tauri runtime)
const getTauriInvoke = async () => {
    if (!isTauri()) return null;
    try {
        const { invoke } = await import('@tauri-apps/api/core');
        return invoke;
    } catch {
        return null;
    }
};

export interface DaemonStatus {
    running: boolean;
    port: number;
    version: string;
}

/**
 * Get the status of the MyMolt daemon.
 * Returns null if not running in Tauri.
 */
export async function getDaemonStatus(): Promise<DaemonStatus | null> {
    const invoke = await getTauriInvoke();
    if (!invoke) return null;
    return invoke<DaemonStatus>('get_daemon_status');
}

/**
 * Get the API base URL.
 * In Tauri: returns the local daemon URL.
 * In browser: returns relative URL (empty string).
 */
export async function getApiUrl(): Promise<string> {
    const invoke = await getTauriInvoke();
    if (!invoke) return '';
    return invoke<string>('get_api_url');
}

/**
 * Ask the agent about page content via direct Rust bridge.
 * Falls back to HTTP if not in Tauri.
 */
export async function askAgent(
    question: string,
    pageUrl: string,
    pageText: string,
): Promise<string> {
    const invoke = await getTauriInvoke();
    if (!invoke) {
        // Fallback to HTTP API
        const { apiClient } = await import('./client');
        const res = await apiClient.post('/browse/ask', {
            url: pageUrl,
            page_text: pageText,
            question,
        }) as { answer: string };
        return res.answer;
    }
    return invoke<string>('ask_agent', {
        question,
        pageUrl,
        pageText,
    });
}

/**
 * Whether we're running inside Tauri (native app) or a browser.
 */
export function isNativeApp(): boolean {
    return isTauri();
}

/**
 * Platform info for display.
 */
export function getPlatformInfo(): { platform: string; native: boolean } {
    if (isTauri()) {
        return { platform: 'Desktop (Tauri)', native: true };
    }
    return { platform: 'Web', native: false };
}
