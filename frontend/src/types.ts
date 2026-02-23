export type UserRole = 'Root' | 'Adult' | 'Child' | 'Senior';

export interface User {
    role: UserRole;
    username: string;
    token?: string; // In MVP, token is global, but maybe role-scoped later
}

export interface SystemStatus {
    online: boolean;
    version: string;
    pairing_enabled: boolean;
    voice_echo_enabled: boolean;
    adblock_enabled: boolean;
    adblock_count: number;
}

export interface IdentityStatus {
    provider: string;
    id: string;
    trust_level: number;
}

export interface IdentityProvider {
    id: string;
    name: string;
    icon_url?: string;
    trust_level: number;
}

export interface WsMessage {
    type: 'text' | 'audio' | 'control' | 'error' | 'thought';
    payload: {
        content?: string;
        data?: string;
        format?: string;
        event?: string;
        sender?: string;
        code?: string;
        message?: string;
    };
}

export interface Skill {
    name: string;
    description: string;
    version: string;
    tools: string[];
}

export interface Integration {
    name: string;
    description: string;
    category: string;
    status: 'available' | 'active' | 'coming_soon';
}
