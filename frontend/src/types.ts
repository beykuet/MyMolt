export interface SystemStatus {
    version: string;
    uptime_secs: number;
    memory_usage_mb: number;
    cpu_usage_percent: number;
    active_agents: number;
    voice_mode_active: boolean;
    pairing_enabled: boolean;
}

export interface IdentityStatus {
    provider: string;
    id: string;
    trust_level: number;
    linked_at: string;
}

export interface WidgetConfig {
    id: string;
    type_: string;
    title: string;
    icon?: string;
    action_url?: string;
}
