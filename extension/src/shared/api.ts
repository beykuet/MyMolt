// MyMolt API client for the extension — communicates with the local/remote MyMolt backend

import type { MyMoltConfig, AskRequest, AskResponse, VaultCredential, DnsRule } from './types';

const CONFIG_KEY = 'mymolt_config';

export async function getConfig(): Promise<MyMoltConfig> {
    const result = await chrome.storage.local.get(CONFIG_KEY);
    return result[CONFIG_KEY] || {
        host: 'http://localhost:3000',
        token: '',
        role: 'Adult',
        userName: '',
        connected: false,
    };
}

export async function saveConfig(config: Partial<MyMoltConfig>): Promise<MyMoltConfig> {
    const current = await getConfig();
    const updated = { ...current, ...config };
    await chrome.storage.local.set({ [CONFIG_KEY]: updated });
    return updated;
}

async function apiFetch(path: string, options?: RequestInit): Promise<Response> {
    const config = await getConfig();
    const url = `${config.host}/api${path}`;
    return fetch(url, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${config.token}`,
            ...options?.headers,
        },
    });
}

// Test connection to MyMolt backend
export async function testConnection(): Promise<boolean> {
    try {
        const res = await apiFetch('/status');
        const connected = res.ok;
        await saveConfig({ connected });
        return connected;
    } catch {
        await saveConfig({ connected: false });
        return false;
    }
}

// Ask agent about current page
export async function askAgent(req: AskRequest): Promise<AskResponse> {
    const res = await apiFetch('/browse/ask', {
        method: 'POST',
        body: JSON.stringify(req),
    });
    if (!res.ok) throw new Error(`Agent error: ${res.status}`);
    return res.json();
}

// Get vault credentials matching a URL
export async function matchVaultCredentials(url: string): Promise<VaultCredential[]> {
    const res = await apiFetch(`/vault/match?url=${encodeURIComponent(url)}`);
    if (!res.ok) return [];
    return res.json();
}

// Log an autofill event for audit
export async function logAutofill(url: string, username: string): Promise<void> {
    await apiFetch('/vault/autofill-log', {
        method: 'POST',
        body: JSON.stringify({ url, username, timestamp: new Date().toISOString() }),
    });
}

// Retrieve a credential's password (one-time, not cached)
export async function getCredentialPassword(credentialId: string): Promise<string> {
    const res = await apiFetch(`/vault/credential/${encodeURIComponent(credentialId)}/password`);
    if (!res.ok) throw new Error('Failed to retrieve password');
    const data = await res.json();
    return data.password;
}

// Get DNS block rules for current role
export async function getDnsRules(): Promise<DnsRule[]> {
    const config = await getConfig();
    const res = await apiFetch(`/dns/rules?role=${config.role}`);
    if (!res.ok) return [];
    return res.json();
}
