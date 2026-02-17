export interface SystemStatus {
    version: string;
    uptime_secs: number;
    memory_usage_mb: number;
    cpu_usage_percent: number;
    active_agents: number;
    voice_mode_active: boolean;
    pairing_enabled: boolean;
    voice_echo_enabled: boolean;
}

export interface IdentityStatus {
    provider: string;
    id: string;
    trust_level: number;
    linked_at: string;
}

export interface IdentityProvider {
    id: string;
    name: string;
    icon_url?: string;
}

export interface WidgetConfig {
    id: string;
    type_: string;
    title: string;
    icon?: string;
    action_url?: string;
}

export type WsMessage =
    | { type: 'text'; payload: { content: string; sender: string; is_final: boolean } }
    | { type: 'audio'; payload: { data: string; format: string } }
    | { type: 'control'; payload: { event: string } }
    | { type: 'error'; payload: { code: string; message: string } };
